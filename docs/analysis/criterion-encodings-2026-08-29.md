---
tags: [analysis, criterion, encodings]
node_type: analysis
---
# criterion encodings 2026-08-29

## Question / scope

Are the Criterion benches actually measuring wall-clock work, and does a Newton nested ADT sit on the handwritten floor versus the encodings we refused (mega-enum, HashSet string ids, `Box<dyn State>`)?

Not in scope: a cross-language shoot-out with XState / SCXML runtimes.

## When

2026-08-29, Windows, `cargo bench` / Criterion 0.5 `WallTime`. Optimized (`--release`). Gnuplot absent; plotters HTML under `target/criterion/`.

## Findings

Criterion **is** wall-clock (`std::time::Instant` + warmup + 95% CI). `Throughput::Elements` is transitions/sec. A raw `Instant` loop is not used. CPU-cycle/`perf` counters are not wired (Linux-only).

First encodings run was **dishonest**: `encoding_batch_1024/newton` landed in hundreds of picoseconds for 1024 ticks (LLVM folded `n += 1024`). `gof_box_dyn` boxed ZSTs, so `Box` did not allocate. Fixes: observable `u64` per encoding, `black_box` on outputs, per-tick sink in the batch loop, non-ZST GoF payload.

### encoding_tick (one transition per Criterion iteration)

Median wall-clock, this machine:

| Encoding | Time | Throughput | vs handwritten |
| --- | --- | --- | --- |
| `handwritten_fields` | 0.35 ns | 2.8 G/s | 1.0× |
| `mega_enum` | 0.55 ns | 1.8 G/s | 1.6× |
| `newton_runtime` | 0.64 ns | 1.6 G/s | 1.8× |
| `gof_box_dyn` (allocating) | 24.1 ns | 41 M/s | ~69× |
| `string_id_set` | 33.1 ns | 30 M/s | ~94× |

### encoding_batch_1024 / 1024 (tight loop)

LLVM inlines `Runtime::apply`. Per-tick cost ≈ handwritten. HashSet / GoF stay at ~31 ns / ~24 ns per tick — the extra work is real.

### apply / harel / compose (not encodings)

| Bench | Time | Read as |
| --- | --- | --- |
| `apply/runtime_apply_toggle` | 25.3 ns | Off↔On + `perform` + stack `Cmd` |
| `apply/step_owned_toggle` | 37.4 ns | ~1.5× apply (owned triple) |
| `apply/runtime_view` | 0.23 ns | query floor |
| `and/handwritten_struct_tick` | 0.23 ns | same band as `and_combinator_tick` |
| `and/and_combinator_tick` | 0.23 ns | `And<L,R>` is not the cost |
| `cmd/and_2_stack` | 9.2 ns | common `perform` concat |
| `cmd/and_4_stack` | 26.5 ns | stack cap |
| `cmd/and_5_heap` | 69.1 ns | ~2.6× stack-4 |
| `sub_diff/none_to_one` | 10.5 ns | host start one listener |
| `sub_diff/swap_two` | 18.9 ns | start+stop |
| `harel/lca_sibling` | 0.22 ns | parent `match` |
| `harel/lca_deep` | 0.78 ns | one extra ancestor |
| `harel/perform_sibling` | 41.2 ns | exit+enter+`Cmd` |
| `harel/perform_descend` | 60.3 ns | deeper enter path |
| `rtc/drain_1` | 10.2 ns | no follow-ups |
| `rtc/drain_8` | 242 ns | ~30 ns/internal msg |
| `rtc/drain_31` | 1.53 µs | just under cap 32 |

## Artifacts

- `cargo bench --bench encodings --bench apply --bench harel --bench compose`
- `benches/README.md`, `benches/encodings.rs`
- HTML: `target/criterion/report/index.html`

## Recommendations

- Quote **encoding_tick** (and HashSet / GoF ratios) in README/talks. Do not quote picosecond batch numbers from before the sink-per-tick fix.
- Pitch is **zero-cost encoding**, not “faster than a `match`.” `handwritten_fields` *is* that match. Newton is within ~2× per message and matches it in a tight loop.
- Mega-enum is as fast at 2×2 and is refused because 3×4 regions explode the variant count, not because it is slow.
- Keep `INLINE_CAP = 4`: heap `Cmd` is the first real allocator tax on the `perform` path.
- Do not add an XState bench.

## Open questions / edge cases

- `And<L,R>` with a fat `Msg: Clone` (e.g. `String`) will show the clone; this run used `Copy`.
- GoF of empty states is much cheaper (`Box<ZST>` does not allocate). The 24 ns figure is the honest “state holds data” case.

## Related

- [[docs/adr/0002-xor-enums-and-and-structs]]
- [[docs/adr/0021-and-combinator]]
- [[docs/adr/0022-sub-diff]]
- [[docs/adr/0018-cmd-inline-then-heap]]
- symbol:Runtime::apply
- symbol:Sub::diff
- symbol:perform
