# newton-machine

[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/newton-machine.svg)](LICENSE-MIT)
[![CI](https://github.com/shan-alexander/newton-machine/actions/workflows/ci.yml/badge.svg)](https://github.com/shan-alexander/newton-machine/actions)

**A typed Harel configuration driven by a pure Elm step — illegal XOR cannot be constructed, effects are data, history is a sidecar.**

A Newton machine is not a new automaton. Harel already defined the *kinematics* (XOR/AND, hierarchy, history, run-to-completion). This crate names the *dynamics and conservation laws* for embedding those kinematics in a running Rust system: **Unidirectional Configuration Architecture (UCA)**.

**Status:** `0.1.0` is the engine, **not yet published** to crates.io. The API is **not** SemVer-stable. There is no proc-macro and no SCXML interpreter.

## Install

```toml
[dependencies]
newton-machine = "0.1"
```

```bash
cargo add newton-machine
```

Source: [github.com/shan-alexander/newton-machine](https://github.com/shan-alexander/newton-machine).

MSRV **1.80**. `#![no_std]` + `alloc` supported. `unsafe` is forbidden.

## Four laws

1. **The configuration is the type.** XOR is an `enum`. AND is a `struct`. Illegal simultaneous children are unrepresentable. No string ids on the hot path.
2. **The only mutation protocol is TEA.** `Msg → (config, context, Cmd)`. Entry/exit/transition actions emit data. The host executes I/O.
3. **Memory is inertial and external.** History is a sidecar of discriminants and small snapshots, not live variants and not closures. The persistable phase space is exactly `{config, context, history}`.
4. **Typestate is a façade, not the machine.** Public phase types may hide illegal methods. The interior stays a configuration tree.

## Nested ADTs

An **ADT** (algebraic data type) in Rust is an `enum` or `struct`. **Nested** means you put them inside each other so the type *is* the active configuration:

```rust
enum Conn {                      // XOR: exactly one child
    Offline,
    Connecting { attempt: u8 },
    Online(Session),             // nested AND
}

struct Session {                 // AND: all fields active
    auth: Auth,                  // XOR region
    sync: Sync,                  // XOR region, orthogonal to auth
}

enum Auth { Anonymous, SignedIn { user: u64 } }
enum Sync { Idle, Fetching, Dirty }
```

That is Harel’s XOR/AND decomposition as ordinary Rust. You cannot construct `Offline` and `Online` at once. You *can* be `SignedIn` and `Fetching` at once, because those are fields of one struct — not a cartesian product you have to name (`OnlineSignedInFetching`, …).

A Harel tool often stores the same shape as a graph of string ids (`"Connecting"`) and a runtime “current configuration set.” A Newton machine stores it as the value of `Self`. Pattern matching is dispatch. No table walk on the hot path.

## Unidirectional Configuration Architecture

**UCA** is the articulate name of the family. A UCA machine is one whose:

- **control state** is a typed configuration (nested ADTs),
- **applied forces** are messages,
- **reactions** are commands (data),
- **memory of prior configurations** is inertial — a sidecar, not live variants.

Unidirectional means there is one door: `update`. The view does not write back. Entry actions do not call the broker. History does not hide inside the enum. If you can serialize `{config, context, history}` after an event and restore it on another process, you are in UCA. If that snapshot would contain closures, sockets, or “whatever the interpreter heap was,” you are not.

Other architectures you will meet (sometimes in the same crate ecosystem):

| Architecture | Control state | Mutation | Effects | Snapshot |
| --- | --- | --- | --- | --- |
| **UCA / Newton** | nested ADTs | TEA `update` only | `Cmd` data, host executes | `{config, context, history}` |
| **Classic TEA** | one flat `Model` | same loop | same `Cmd` | model only; no chart, no history sidecar |
| **Harel tool / SCXML** | id graph + current-set | interpreter or generated switch | often I/O in entry actions | interpreter heap |
| **Typestate lattice** | each config is a type (`Machine<Online>`) | methods exist only in some types | varies | hard: `S` is in the type, not the value |
| **GoF State** | `Box<dyn State>` | virtual `handle` | easy to hide I/O in the object | trait objects / closures |
| **Actor / threaded regions** | one mailbox per region | concurrent messages | I/O anywhere | not a single step |

Classic TEA is UCA with a trivial chart. Classic Harel is the kinematics without UCA’s conservation laws. Strip hierarchy from Newton and you are back at TEA. Strip the laws and you are back at Harel-as-diagram.

## TEA runtime — and what we did not choose

[`Runtime`](https://docs.rs/newton-machine/latest/newton_machine/struct.Runtime.html) is the Elm runtime in this crate: it **owns** `{config, context, history}`, and the only way to change them from the outside is `apply(msg)` (or `step` for an owned, pure variant).

```text
Runtime::boot(flags)  →  (Runtime, entry Cmd)
Runtime::apply(msg)   →  Cmd          // host executes Cmd
Runtime::view()       →  View
Runtime::snapshot()   →  {config, context, history}
```

That is what “TEA runtime” means here. It is not Tokio, not a GUI framework, not a thread. It is a struct plus a function.

Alternatives we refused (and why):

| Alternative | What it looks like | Why not as the core |
| --- | --- | --- |
| Call `update` yourself with no owner | three loose variables | easy to persist a torn triple; `Runtime` keeps them together |
| Callbacks / observers | `on_enter(Connecting, \|\| http())` | I/O in the chart; snapshots lie |
| Interpreter step | `machine.dispatch("CONNECT")` | string ids, untyped config |
| One task per orthogonal region | `tokio::spawn` per AND field | races; Harel concurrency is virtual (one clock, document order) |
| Async `Stream` of events inside `update` | `.await` in entry | `update` is no longer a step you can replay |

The host may still be async. It awaits I/O **after** `apply` returns a `Cmd`, then feeds `Msg::Authed` back in. Live, test, and replay stay one function.

## LCA (least common ancestor)

**LCA** is a tree idea, not a Newton invention. Given two nodes, it is the deepest node that is an ancestor of both (a node is an ancestor of itself).

```text
            Root          LCA(Offline, Connecting) = Root
          /  |  \         LCA(Connecting, Online)  = Root
    Offline  Connecting  Online
                           |
                         Session
```

Harel uses LCA to decide **what to exit and what to enter** so you do not exit a parent you are staying in:

1. Exit source → LCA, inner first (do not exit the LCA).
2. Assign the destination configuration.
3. Enter LCA → target, outer first (do not re-enter the LCA).

If you go `Offline → Connecting`, both hang off `Root`, so you exit `Offline`, enter `Connecting`, and never “exit the whole machine.” If you moved between two children of `Online`, you would not exit `Online`. [`perform`](https://docs.rs/newton-machine/latest/newton_machine/fn.perform.html) is that algorithm over your [`Topology`](https://docs.rs/newton-machine/latest/newton_machine/trait.Topology.html) parent function.

A Harel *chart* specifies this. A Newton *machine* implements it on nested ADTs, then still returns `Cmd` instead of calling I/O from `enter`.

## RTC drain cap

**RTC** (run-to-completion): one external `Msg` is fully processed before the next external `Msg`. Harel allows *internal* events during that step (a transition queues another message). [`rtc`](https://docs.rs/newton-machine/latest/newton_machine/fn.rtc.html) drains those follow-ups inside `update`.

A guard that always retriggers (`A → B → A → …`) would never return. The **drain cap** (default 32) bounds how many internal messages one external step may handle. Exceed it and you get [`Storm`](https://docs.rs/newton-machine/latest/newton_machine/struct.Storm.html) — a bug in the chart, not a hang the watchdog has to guess at.

See `cargo run --example storm`.

## Newton machine vs Harel statechart

Same kinematics. Different artifact.

| | Harel / SCXML / STATEMATE | Newton machine (UCA) |
| --- | --- | --- |
| Primary artifact | a diagram / document | a running Rust value |
| Configuration | current-set of ids | nested ADTs (`Self`) |
| Effects | often code in entry/exit | `Cmd` data; host executes |
| History | often inside the live config / interpreter | sidecar; opt-in; serializable |
| Replay | tool-dependent | same `update` as live |
| Illegal XOR | runtime error | does not compile |
| When to use | spec, interchange, multi-language codegen | Rust process that must not lie about its state |

**You still “have Harel”** if you draw XOR/AND, use LCA, use history, use RTC. You **leave Harel-as-usually-implemented** when entry actions cannot call HTTP, when history cannot hold a socket, and when the snapshot is a triple of values. That is the family.

If the deliverable is a PDF of legal configurations, use a Harel tool. If the deliverable is a crash-consistent journal of a Rust service, use a Newton machine. Do not use either for a three-variant enum and a `match`.

## Other Rust crates

These are good at what they pick. UCA is the conjunction they leave implicit.

| Crate | What it is | Gap vs Newton |
| --- | --- | --- |
| [`statig`](https://crates.io/crates/statig) | compiled hierarchical FSM, `no_std`, event `handle` | not an Elm loop; orthogonality / persistable sidecar are not the product |
| [`smlang`](https://crates.io/crates/smlang) | Boost-SML-like DSL macro | string-ish DSL; no UCA snapshot law |
| [`rust-fsm`](https://crates.io/crates/rust-fsm) | small DSL + trait | flat machines; no Harel AND/history/TEA host split |
| [`sm`](https://crates.io/crates/sm) | 100% static, compile-time transitions | no Elm `Cmd`/host; no orthogonal product |
| [`state-machines`](https://crates.io/crates/state-machines) | typestate port of Ruby’s gem | typestate lattice; orthogonal × history explodes |
| [`essm`](https://crates.io/crates/essm) / [`statum`](https://crates.io/crates/statum) | typestate, illegal transitions do not compile | same lattice problem; no TEA runtime |
| SCXML crates | document + interpreter | string ids, heap snapshot, wrong API for app engineers |
| elm-statecharts (Elm) | msgs into `update` + statechart | the *shape* we steal; not Rust |

Typestate remains allowed as a **façade** (`Machine<Offline>` hiding `logout`). It is not the interior.

## Elm-shaped loop

```rust
use newton_machine::prelude::*;

struct Chart;
struct Model;

impl Machine for Chart {
    type Flags = ();
    type Model = Model;
    type Msg = ();
    type Cmd = Cmd<()>;
    type View = ();
    type History = ();
    type NodeId = &'static str;

    fn init(_: ()) -> Boot<Self> {
        Boot::new(Chart, Model, (), Cmd::none())
    }

    fn update(&mut self, _: &mut Model, _: &mut (), _: ()) -> Cmd<()> {
        Cmd::none()
    }

    fn view(&self, _: &Model) {}

    fn in_state(&self, _: &'static str) -> bool {
        true
    }
}

let (mut rt, _entry_cmds) = Runtime::<Chart>::boot(());
let _cmd = rt.apply(());
let _view = rt.view();
```

`Runtime` owns `{config, context, history}`. The host executes `Cmd`. The crate never opens a socket.

## Examples (GitHub only)

Runnable demos live in [`examples/`](examples/) in this repository. They are **not** in the crates.io crate.

```bash
cargo run --example counter      # TEA only
cargo run --example connection   # XOR + LCA + history + Cmd
cargo run --example orthogonal   # AND regions
cargo run --example replay       # snapshot restore
cargo run --example storm        # RTC drain cap
cargo run --example aapl_1m --release  # AAPL 1m April 2026; HOST ema/stoch
```

`counter.rs` and `connection.rs` are walkthroughs (module docs). Start there.

## Benchmarks (GitHub only)

Criterion harnesses under [`benches/`](benches/). They are **not** in the crates.io package. Read the file headers even if you do not run them — each file says what the number means and what it does not prove.

```bash
cargo bench --bench encodings # handwritten fields vs Runtime vs mega-enum vs HashSet ids vs Box<dyn>
cargo bench --bench apply     # Runtime::apply vs step vs view
cargo bench --bench harel     # LCA / perform / rtc drain
cargo bench --bench compose   # And<L,R> vs handwritten struct; Cmd::and; Sub::diff
```

HTML report: `target/criterion/report/index.html`.

## What this crate will never do

- Interpret SCXML documents at runtime.
- Open sockets, place orders, or perform I/O inside `update`.
- Run one thread per orthogonal region.
- Store closures in a snapshot.
- Be a risk gateway. The machine may *request* lock via `Cmd`. Only a host firewall may *admit* a command to the world.
- Load YAML as a chart, or treat a sleeve / superstate table as a Harel node. Truth is nested ADTs; policy is a host [`ChordTable`](https://docs.rs/newton-machine/latest/newton_machine/struct.ChordTable.html) (or the desk’s own `HashMap`).

## Features

| Feature | Default | Role |
| --- | --- | --- |
| `std` | yes | `std::error::Error` for `Storm`; implies `alloc` |
| `alloc` | via `std` | `Cmd` heap beyond 4 atoms, `Sub::Many`, `Tape` vec, `ChordTable`, `Fleet` |
| `serde` | no | derives on snapshot types |

## Crate layout

| Piece | Owns |
| --- | --- |
| **`newton-machine`** (this crate) | UCA laws and engine |
| `examples/` (GitHub) | runnable demos, not on crates.io |
| `benches/` (GitHub) | Criterion: apply / LCA / And / Cmd |
| `docs/` (GitHub) | rustbrain ADRs / goals / concepts |
| host application | gateway, watchdog, broker, persistence backend |

Architecture graph: [rustbrain](https://docs.rs/rustbrain) under `docs/` (`rustbrain context "why newton machine"`).

## License

MIT OR Apache-2.0
