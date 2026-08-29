//! # AAPL 1m bars — a realistic Newton machine
//!
//! Run: `cargo run --example aapl_1m --release`
//!
//! DuckDB extracted April 2026 1-minute OHLCV into
//! `examples/aapl_1m_2026-04.csv`. Each row is one `Msg::Bar`. Indicators
//! are **host** incremental `Ema` / `Stoch` (const packs + `*State::push`),
//! not a crate dependency. The chart never sees a lookback.
//!
//! ## Chart (AND of two XOR regions) — newton-machine
//!
//! ```text
//! Desk {
//!   quad: Warmup | Neutral | QuadOversold | QuadOverbought
//!   ema:  Warmup | Neutral | BelowTwoOrMore | Split | AboveTwoOrMore
//! }
//! ```
//!
//! This is **configuration** (what is true). Overlaps such as
//! `QuadOversold ∩ Split` are **not** a third parent. The host policy
//! layer below names and scores them.
//!
//! ## Three kinds of “previous”
//!
//! | What | Where | Lifetime | Role |
//! | --- | --- | --- | --- |
//! | `now` | `rt.machine()` after `apply` | this bar | live configuration |
//! | `prev` | host copy of `machine` *before* `apply` | one bar | edge detector for printing (“did overlap just become true?”) |
//! | sidecar | `rt.history()` | until the next opted-in *exit* | inertia: last extreme, how long we were there |
//! | journal | `rt.snapshot()` | you persist it | `{config, context, history}` after a completed step |
//!
//! `prev` / `prev_ol` are **not** the sidecar. They are host scratch so we
//! can print rising edges. The sidecar is written inside `Transitional::exit`
//! when leaving `QuadOversold` / `QuadOverbought`. `snapshot()` clones the
//! triple; it does not compute history — history was already there.
//!
//! ## File map (lib vs host)
//!
//! Sections marked **LIB** call `newton_machine::…`. Sections marked **HOST**
//! are this example: CSV, TA engines, overlap/score policy, stdout.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

// LIB — named imports so a reader can see the crate without prelude::*.
use newton_machine::cmd::Cmd;
use newton_machine::machine::{Boot, Machine};
use newton_machine::runtime::Runtime;
use newton_machine::topology::Topology;
use newton_machine::transition::{perform, Transitional};

// =============================================================================
// HOST — incremental TA (not the Newton chart, not a crate dep)
//
// Pattern stolen from finance-solution `stocks::ta`:
//   1. `const` packs (lookbacks / fast vs full) live at module scope.
//   2. One engine type per family (`Ema`, `Stoch`), parameterized, not a
//      type per period (`Ema9`, `Ema21`, …).
//   3. `push` is O(window) here (scan min/max). Live desks can swap in a
//      ring + sliding max; the *call site* (`Ta::push` → `Snap`) stays.
// =============================================================================

/// Wilder/Chart-school EMA: α = 2/(period+1), seed = SMA of the first
/// `period` closes, then recursive. Warm-up is `None` until the seed is full.
struct Ema {
    period: usize,
    alpha: f64,
    seed: VecDeque<f64>,
    value: Option<f64>,
}

impl Ema {
    fn new(period: usize) -> Self {
        assert!(period >= 1, "ema period");
        Self {
            period,
            alpha: 2.0 / (period as f64 + 1.0),
            seed: VecDeque::with_capacity(period),
            value: None,
        }
    }

    fn push(&mut self, close: f64) -> Option<f64> {
        if let Some(prev) = self.value {
            let next = self.alpha * close + (1.0 - self.alpha) * prev;
            self.value = Some(next);
            return Some(next);
        }
        self.seed.push_back(close);
        if self.seed.len() < self.period {
            return None;
        }
        let seed = self.seed.iter().sum::<f64>() / self.period as f64;
        self.seed.clear();
        self.value = Some(seed);
        Some(seed)
    }

    fn last(&self) -> Option<f64> {
        self.value
    }
}

/// Unvalidated `Copy` pack. Fast = raw %K; Full = SMA(raw %K, k_smooth).
/// `%D` is always SMA(%K, d_period). Same formula, extra smoothing.
#[derive(Clone, Copy, Debug)]
struct StochParams {
    k_period: usize,
    k_smooth: usize,
    d_period: usize,
}

impl StochParams {
    const fn fast(k_period: usize, d_period: usize) -> Self {
        Self {
            k_period,
            k_smooth: 1,
            d_period,
        }
    }

    #[allow(dead_code)] // pack vocabulary: Full is how you would add 14,3,3
    const fn full(k_period: usize, k_smooth: usize, d_period: usize) -> Self {
        Self {
            k_period,
            k_smooth,
            d_period,
        }
    }
}

const EMA9: usize = 9;
const EMA21: usize = 21;
const EMA50: usize = 50;
const EMA200: usize = 200;

const FAST_9_3: StochParams = StochParams::fast(9, 3);
const FAST_14_3: StochParams = StochParams::fast(14, 3);
const FAST_40_3: StochParams = StochParams::fast(40, 3);
const FAST_60_3: StochParams = StochParams::fast(60, 3);

/// Incremental stochastic. Flat HH==LL: carry previous raw %K, else 50.
/// `push` returns `Some((%K, %D))` only when **both** lines are defined
/// (same grain as a finance-solution `StochState::push`).
struct Stoch {
    params: StochParams,
    highs: VecDeque<f64>,
    lows: VecDeque<f64>,
    raw_k: VecDeque<f64>,
    smooth_k: VecDeque<f64>,
    prev_raw_k: Option<f64>,
}

impl Stoch {
    fn new(params: StochParams) -> Self {
        assert!(params.k_period >= 1 && params.k_smooth >= 1 && params.d_period >= 1);
        Self {
            params,
            highs: VecDeque::with_capacity(params.k_period),
            lows: VecDeque::with_capacity(params.k_period),
            raw_k: VecDeque::with_capacity(params.k_smooth),
            smooth_k: VecDeque::with_capacity(params.d_period),
            prev_raw_k: None,
        }
    }

    fn push(&mut self, high: f64, low: f64, close: f64) -> Option<(f64, f64)> {
        push_window(&mut self.highs, self.params.k_period, high);
        push_window(&mut self.lows, self.params.k_period, low);
        if self.highs.len() < self.params.k_period {
            return None;
        }
        let hh = self.highs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let ll = self.lows.iter().copied().fold(f64::INFINITY, f64::min);
        let range = hh - ll;
        let raw = if range == 0.0 {
            self.prev_raw_k.unwrap_or(50.0)
        } else {
            100.0 * (close - ll) / range
        };
        self.prev_raw_k = Some(raw);
        push_window(&mut self.raw_k, self.params.k_smooth, raw);
        if self.raw_k.len() < self.params.k_smooth {
            return None;
        }
        let k = mean(&self.raw_k);
        push_window(&mut self.smooth_k, self.params.d_period, k);
        if self.smooth_k.len() < self.params.d_period {
            return None;
        }
        Some((k, mean(&self.smooth_k)))
    }
}

fn push_window(buf: &mut VecDeque<f64>, cap: usize, v: f64) {
    if buf.len() == cap {
        buf.pop_front();
    }
    buf.push_back(v);
}

fn mean(buf: &VecDeque<f64>) -> f64 {
    buf.iter().sum::<f64>() / buf.len() as f64
}

// =============================================================================
// HOST — lake bars + four EMA engines + four stochastic packs
// =============================================================================

#[derive(Clone, Debug)]
struct Bar {
    ts: String,
    high: f64,
    low: f64,
    close: f64,
}

struct Ta {
    ema9: Ema,
    ema21: Ema,
    ema50: Ema,
    ema200: Ema,
    stoch9: Stoch,
    stoch14: Stoch,
    stoch40: Stoch,
    stoch60: Stoch,
}

impl Ta {
    fn new() -> Ta {
        Ta {
            ema9: Ema::new(EMA9),
            ema21: Ema::new(EMA21),
            ema50: Ema::new(EMA50),
            ema200: Ema::new(EMA200),
            stoch9: Stoch::new(FAST_9_3),
            stoch14: Stoch::new(FAST_14_3),
            stoch40: Stoch::new(FAST_40_3),
            stoch60: Stoch::new(FAST_60_3),
        }
    }

    fn push(&mut self, bar: &Bar) -> Snap {
        let _ = self.ema9.push(bar.close);
        let _ = self.ema21.push(bar.close);
        let _ = self.ema50.push(bar.close);
        let _ = self.ema200.push(bar.close);
        Snap {
            ts: bar.ts.clone(),
            close: bar.close,
            ema9: self.ema9.last(),
            ema21: self.ema21.last(),
            ema50: self.ema50.last(),
            ema200: self.ema200.last(),
            k9: self
                .stoch9
                .push(bar.high, bar.low, bar.close)
                .map(|kd| kd.0),
            k14: self
                .stoch14
                .push(bar.high, bar.low, bar.close)
                .map(|kd| kd.0),
            k40: self
                .stoch40
                .push(bar.high, bar.low, bar.close)
                .map(|kd| kd.0),
            k60: self
                .stoch60
                .push(bar.high, bar.low, bar.close)
                .map(|kd| kd.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Snap {
    ts: String,
    close: f64,
    ema9: Option<f64>,
    ema21: Option<f64>,
    ema50: Option<f64>,
    ema200: Option<f64>,
    k9: Option<f64>,
    k14: Option<f64>,
    k40: Option<f64>,
    k60: Option<f64>,
}

impl Snap {
    fn emas(&self) -> [Option<f64>; 4] {
        [self.ema9, self.ema21, self.ema50, self.ema200]
    }

    fn stoch_k(&self) -> [Option<f64>; 4] {
        [self.k9, self.k14, self.k40, self.k60]
    }

    fn above_below(&self) -> (u8, u8) {
        let mut above = 0u8;
        let mut below = 0u8;
        for e in self.emas() {
            match e {
                Some(v) if self.close > v => above += 1,
                Some(v) if self.close < v => below += 1,
                _ => {}
            }
        }
        (above, below)
    }
}

// =============================================================================
// HOST knobs — classify the world. Changing these does not change Desk's type.
// =============================================================================

/// %K at-or-below this ⇒ that stochastic is “oversold.”
const STOCH_OVERSOLD: f64 = 27.0;
/// %K at-or-above this ⇒ that stochastic is “overbought.”
const STOCH_OVERBOUGHT: f64 = 79.0;

// =============================================================================
// LIB shape — nested ADTs. Desk is the Newton configuration (what is true).
// =============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum QuadStoch {
    Warmup,
    Neutral,
    QuadOversold,
    QuadOverbought,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EmaStance {
    Warmup,
    Neutral,
    BelowTwoOrMore,
    /// 2 EMAs above and 2 below (or any ≥2 / ≥2). Mixed tape, not a trend bet.
    Split,
    AboveTwoOrMore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Desk {
    quad: QuadStoch,
    ema: EmaStance,
}

impl Default for Desk {
    fn default() -> Self {
        Self {
            quad: QuadStoch::Warmup,
            ema: EmaStance::Warmup,
        }
    }
}

fn classify_quad(s: &Snap) -> QuadStoch {
    let ks = s.stoch_k();
    if ks.iter().any(Option::is_none) {
        return QuadStoch::Warmup;
    }
    let ks = [
        ks[0].unwrap(),
        ks[1].unwrap(),
        ks[2].unwrap(),
        ks[3].unwrap(),
    ];
    if ks.iter().all(|k| *k < STOCH_OVERSOLD) {
        QuadStoch::QuadOversold
    } else if ks.iter().all(|k| *k > STOCH_OVERBOUGHT) {
        QuadStoch::QuadOverbought
    } else {
        QuadStoch::Neutral
    }
}

fn classify_ema(s: &Snap) -> EmaStance {
    if s.emas().iter().any(Option::is_none) {
        return EmaStance::Warmup;
    }
    let (above, below) = s.above_below();
    match (above >= 2, below >= 2) {
        (true, true) => EmaStance::Split,
        (true, false) => EmaStance::AboveTwoOrMore,
        (false, true) => EmaStance::BelowTwoOrMore,
        (false, false) => EmaStance::Neutral,
    }
}

// =============================================================================
// HOST policy — overlaps + score. Not a region. Not newton-machine.
// =============================================================================

/// Named cell in the `quad × ema` product. Adding a variant here does **not**
/// change `Desk` and does **not** require a new LCA parent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Overlap {
    None,
    /// Oversold while EMAs are split (2–2): bounce possible, trend not confirmed.
    OversoldLift,
    /// Oversold while price is above ≥2 EMAs: bounce-with-trend sketch.
    OversoldSuperLift,
    /// Overbought while price is below ≥2 EMAs: fade-with-trend sketch.
    OverboughtFade,
}

fn overlap(d: &Desk) -> Overlap {
    match (d.quad, d.ema) {
        (QuadStoch::QuadOversold, EmaStance::Split) => Overlap::OversoldLift,
        (QuadStoch::QuadOversold, EmaStance::AboveTwoOrMore) => Overlap::OversoldSuperLift,
        (QuadStoch::QuadOverbought, EmaStance::BelowTwoOrMore) => Overlap::OverboughtFade,
        _ => Overlap::None,
    }
}

/// Additive score from **independent regions**, then a *named-combo* term.
///
/// ```text
/// score = quad_weight + ema_weight + overlap_bonus
/// ```
///
/// Region weights are linear: oversold = +2, overbought = −2, above EMAs = +1,
/// below EMAs = −1, **Split = 0** (2–2 is not a directional EMA bet).
///
/// Overlap bonus is the non-linear bit: SuperLift is worth more than
/// oversold+above added twice (we do **not** double-count the region
/// weights — bonus is extra conviction on the *named cell*).
///
/// This is not a fill. A later policy engine would threshold `score` and
/// emit intent; a gateway would still admit/refuse.
#[derive(Clone, Copy)]
struct ScoreParts {
    quad: i8,
    ema: i8,
    combo: i8,
}

impl ScoreParts {
    fn total(self) -> i8 {
        self.quad + self.ema + self.combo
    }
}

fn score_parts(d: &Desk) -> ScoreParts {
    let quad = match d.quad {
        QuadStoch::QuadOversold => 2,
        QuadStoch::QuadOverbought => -2,
        QuadStoch::Warmup | QuadStoch::Neutral => 0,
    };
    let ema = match d.ema {
        EmaStance::AboveTwoOrMore => 1,
        EmaStance::BelowTwoOrMore => -1,
        // Split: mixed tape. Zero, not “a little bullish.”
        EmaStance::Split | EmaStance::Warmup | EmaStance::Neutral => 0,
    };
    let combo = match overlap(d) {
        Overlap::OversoldSuperLift => 2,
        Overlap::OversoldLift => 1,
        Overlap::OverboughtFade => -2,
        Overlap::None => 0,
    };
    ScoreParts { quad, ema, combo }
}

fn score(d: &Desk) -> i8 {
    score_parts(d).total()
}

// =============================================================================
// LIB — Topology + Transitional + Machine (newton-machine)
// =============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Node {
    Root,
    Quad,
    Ema,
    QWarmup,
    QNeutral,
    QOversold,
    QOverbought,
    EWarmup,
    ENeutral,
    EBelow,
    ESplit,
    EAbove,
}

impl QuadStoch {
    fn node(self) -> Node {
        match self {
            QuadStoch::Warmup => Node::QWarmup,
            QuadStoch::Neutral => Node::QNeutral,
            QuadStoch::QuadOversold => Node::QOversold,
            QuadStoch::QuadOverbought => Node::QOverbought,
        }
    }
}

impl EmaStance {
    fn node(self) -> Node {
        match self {
            EmaStance::Warmup => Node::EWarmup,
            EmaStance::Neutral => Node::ENeutral,
            EmaStance::BelowTwoOrMore => Node::EBelow,
            EmaStance::Split => Node::ESplit,
            EmaStance::AboveTwoOrMore => Node::EAbove,
        }
    }
}

impl Topology for Desk {
    type Node = Node;
    fn parent(node: Node) -> Option<Node> {
        match node {
            Node::Root => None,
            Node::Quad | Node::Ema => Some(Node::Root),
            Node::QWarmup | Node::QNeutral | Node::QOversold | Node::QOverbought => {
                Some(Node::Quad)
            }
            Node::EWarmup | Node::ENeutral | Node::EBelow | Node::ESplit | Node::EAbove => {
                Some(Node::Ema)
            }
        }
    }
}

/// LIB sidecar. Written on *exit* of opted-in composites, not every bar.
/// This is the auditable inertia: “where was quad last time we left an extreme?”
#[derive(Clone, Debug, Default)]
struct History {
    last_quad: Option<QuadStoch>,
    last_ema: Option<EmaStance>,
    bars_in_last_quad: u32,
    last_extreme_ts: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostCmd {
    AlertOversold,
    AlertOverbought,
    Persist,
}

impl Transitional for Desk {
    type Ctx = Model;
    type Hist = History;
    type Cmd = Cmd<HostCmd>;

    fn exit(&mut self, node: Node, ctx: &mut Model, hist: &mut History) -> Cmd<HostCmd> {
        match node {
            Node::QOversold | Node::QOverbought => {
                // Sidecar write — newton-machine calls this from `perform`.
                hist.last_quad = Some(self.quad);
                hist.bars_in_last_quad = ctx.bars_in_quad;
                hist.last_extreme_ts = Some(ctx.ts.clone());
                Cmd::single(HostCmd::Persist)
            }
            Node::EAbove | Node::EBelow | Node::ESplit => {
                hist.last_ema = Some(self.ema);
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }

    fn enter(&mut self, node: Node, ctx: &mut Model, _: &mut History) -> Cmd<HostCmd> {
        match node {
            Node::QOversold => {
                ctx.bars_in_quad = 0;
                Cmd::single(HostCmd::AlertOversold)
            }
            Node::QOverbought => {
                ctx.bars_in_quad = 0;
                Cmd::single(HostCmd::AlertOverbought)
            }
            Node::QNeutral | Node::QWarmup => {
                ctx.bars_in_quad = 0;
                Cmd::none()
            }
            _ => Cmd::none(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Model {
    ts: String,
    close: f64,
    snap: Snap,
    bars_in_quad: u32,
    bar_i: u32,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            ts: String::new(),
            close: 0.0,
            snap: Snap {
                ts: String::new(),
                close: 0.0,
                ema9: None,
                ema21: None,
                ema50: None,
                ema200: None,
                k9: None,
                k14: None,
                k40: None,
                k60: None,
            },
            bars_in_quad: 0,
            bar_i: 0,
        }
    }
}

struct Msg {
    snap: Snap,
}

impl Machine for Desk {
    type Flags = ();
    type Model = Model;
    type Msg = Msg;
    type Cmd = Cmd<HostCmd>;
    type View = String;
    type History = History;
    type NodeId = Node;

    fn init(_: ()) -> Boot<Self> {
        Boot::new(
            Desk::default(),
            Model::default(),
            History::default(),
            Cmd::none(),
        )
    }

    fn update(&mut self, model: &mut Model, hist: &mut History, msg: Msg) -> Cmd<HostCmd> {
        model.ts = msg.snap.ts.clone();
        model.close = msg.snap.close;
        model.snap = msg.snap.clone();
        model.bar_i += 1;
        model.bars_in_quad = model.bars_in_quad.saturating_add(1);

        let want = Desk {
            quad: classify_quad(&msg.snap),
            ema: classify_ema(&msg.snap),
        };

        let mut cmd = Cmd::none();
        if want.quad != self.quad {
            let from = self.quad.node();
            let to = want.quad.node();
            let dest = Desk {
                quad: want.quad,
                ema: self.ema,
            };
            // LIB: LCA transition. Sidecar writes happen inside `exit`.
            cmd = cmd.and(perform(self, from, dest, to, model, hist));
        }
        if want.ema != self.ema {
            let from = self.ema.node();
            let to = want.ema.node();
            let dest = Desk {
                quad: self.quad,
                ema: want.ema,
            };
            cmd = cmd.and(perform(self, from, dest, to, model, hist));
        }
        cmd
    }

    fn view(&self, model: &Model) -> String {
        // HOST policy functions called from view — display only.
        format!(
            "{}  ${:.2}  quad={:?}  ema={:?}  overlap={:?}  score={:+}",
            model.ts,
            model.close,
            self.quad,
            self.ema,
            overlap(self),
            score(self)
        )
    }

    fn in_state(&self, id: Node) -> bool {
        id == Node::Root
            || id == Node::Quad
            || id == Node::Ema
            || self.quad.node() == id
            || self.ema.node() == id
    }
}

// =============================================================================
// HOST — CSV + pretty print
// =============================================================================

fn csv_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/aapl_1m_2026-04.csv")
}

fn load_bars() -> Vec<Bar> {
    let f = File::open(csv_path()).unwrap_or_else(|e| {
        panic!(
            "missing {}: {e} (GitHub tree; DuckDB-extracted April 2026 AAPL 1m)",
            csv_path().display()
        )
    });
    let mut bars = Vec::new();
    for (i, line) in BufReader::new(f).lines().enumerate() {
        let line = line.expect("read");
        if i == 0 {
            continue;
        }
        let c: Vec<&str> = line.split(',').collect();
        if c.len() < 7 {
            continue;
        }
        bars.push(Bar {
            ts: c[1].to_string(),
            high: c[3].parse().expect("high"),
            low: c[4].parse().expect("low"),
            close: c[5].parse().expect("close"),
        });
    }
    bars
}

fn fmt_opt(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("{x:7.2}"),
        None => "    n/a".into(),
    }
}

fn tick(s: Option<f64>, close: f64) -> char {
    match s {
        Some(e) if close > e => '↑',
        Some(e) if close < e => '↓',
        Some(_) => '=',
        None => '·',
    }
}

fn print_change(prev: &Desk, rt: &Runtime<Desk>, cmd: &Cmd<HostCmd>, prev_ol: Overlap) {
    // LIB reads: now = machine, sidecar = history, numbers = model.
    let m = rt.model();
    let now = rt.machine();
    let sidecar = rt.history();
    let (above, below) = m.snap.above_below();
    let ol = overlap(now);
    println!("── {}  close={:.2}  bar#{} ──", m.ts, m.close, m.bar_i);
    if prev.quad != now.quad {
        println!(
            "  stoch  {:?} → {:?}    K9={} K14={} K40={} K60={}",
            prev.quad,
            now.quad,
            fmt_opt(m.snap.k9),
            fmt_opt(m.snap.k14),
            fmt_opt(m.snap.k40),
            fmt_opt(m.snap.k60)
        );
        println!(
            "         sidecar last_quad={:?}  bars_in_last_extreme={}  last_extreme_ts={}",
            sidecar.last_quad,
            sidecar.bars_in_last_quad,
            sidecar.last_extreme_ts.as_deref().unwrap_or("—")
        );
    }
    if prev.ema != now.ema {
        let s = &m.snap;
        println!(
            "  ema    {:?} → {:?}    above={above} below={below}  9{} 21{} 50{} 200{}",
            prev.ema,
            now.ema,
            tick(s.ema9, s.close),
            tick(s.ema21, s.close),
            tick(s.ema50, s.close),
            tick(s.ema200, s.close)
        );
        println!(
            "         ema9={} ema21={} ema50={} ema200={}  last_ema={:?}",
            fmt_opt(s.ema9),
            fmt_opt(s.ema21),
            fmt_opt(s.ema50),
            fmt_opt(s.ema200),
            sidecar.last_ema
        );
    }
    let sp = score_parts(now);
    println!(
        "  policy overlap={:?}{}  score={:+}  (quad{:+} + ema{:+} + combo{:+})",
        ol,
        if ol != prev_ol { "  ← edge" } else { "" },
        sp.total(),
        sp.quad,
        sp.ema,
        sp.combo
    );
    if !cmd.is_none() {
        print!("  cmd    ");
        let mut first = true;
        for c in cmd.iter() {
            if !first {
                print!(", ");
            }
            print!("{c:?}");
            first = false;
        }
        println!();
    }
    println!();
}

fn main() {
    println!("# AAPL 1m  April 2026  newton-machine  (HOST ema/stoch, not a TA crate)\n");
    println!("chart = quad × ema   policy = overlap/score (host)   sidecar = rt.history()\n");

    let bars = load_bars();
    let mut ta = Ta::new();

    // LIB: owns {config, model, sidecar}.
    let (mut rt, _) = Runtime::<Desk>::boot(());

    let mut changes = 0u32;
    let mut oversold_entries = 0u32;
    let mut overbought_entries = 0u32;
    let mut oversold_lift = 0u32;
    let mut oversold_super = 0u32;
    let mut overbought_fade = 0u32;

    for bar in &bars {
        let snap = ta.push(bar);

        // HOST scratch: one-bar "was". Not the sidecar.
        let prev = *rt.machine();
        let prev_ol = overlap(&prev);

        // LIB: one RTC step. Sidecar may update inside perform → exit.
        let cmd = rt.apply(Msg { snap });

        // LIB: live configuration after the step.
        let now = *rt.machine();

        if now != prev {
            changes += 1;
        }
        if prev.quad != QuadStoch::QuadOversold && now.quad == QuadStoch::QuadOversold {
            oversold_entries += 1;
        }
        if prev.quad != QuadStoch::QuadOverbought && now.quad == QuadStoch::QuadOverbought {
            overbought_entries += 1;
        }
        let ol = overlap(&now);
        if prev_ol != Overlap::OversoldLift && ol == Overlap::OversoldLift {
            oversold_lift += 1;
        }
        if prev_ol != Overlap::OversoldSuperLift && ol == Overlap::OversoldSuperLift {
            oversold_super += 1;
        }
        if prev_ol != Overlap::OverboughtFade && ol == Overlap::OverboughtFade {
            overbought_fade += 1;
        }
        if prev.quad != now.quad || prev.ema != now.ema {
            print_change(&prev, &rt, &cmd, prev_ol);
        }
    }

    // LIB: journal point — clone of {config: now, context: model, history: sidecar}.
    let journal = rt.snapshot();
    println!("── footer ──");
    println!("  bars            {}", bars.len());
    println!("  config changes  {changes}");
    println!("  quad oversold entries      {oversold_entries}");
    println!("  quad overbought entries    {overbought_entries}");
    println!("  policy OversoldLift        {oversold_lift}  (oversold ∩ ema Split)");
    println!("  policy OversoldSuperLift   {oversold_super}  (oversold ∩ ema above 2+)");
    println!("  policy OverboughtFade      {overbought_fade}  (overbought ∩ ema below 2+)");
    println!("  snapshot.config   {:?}", journal.config);
    println!(
        "  snapshot.history  last_quad={:?} last_ema={:?} last_extreme_ts={}",
        journal.history.last_quad,
        journal.history.last_ema,
        journal.history.last_extreme_ts.as_deref().unwrap_or("—")
    );
}
