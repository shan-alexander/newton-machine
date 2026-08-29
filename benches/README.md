# Benches (GitHub only)

Criterion wall-clock harnesses. **Not** in the crates.io package.

```bash
cargo bench                         # all four
cargo bench --bench encodings       # nested ADT vs alternatives
cargo bench --bench apply           # Runtime::apply vs step vs view
cargo bench --bench harel           # LCA / perform / rtc drain
cargo bench --bench compose         # And vs handwritten; Cmd::and; Sub::diff
```

HTML: `target/criterion/report/index.html`.

## What Criterion is measuring

Criterion 0.5’s default measurement is **`WallTime`**: `std::time::Instant`
around the iteration, with warmup, outlier detection, and a 95% confidence
interval. `Throughput::Elements` converts ns/iter into transitions/sec.

That *is* wall-clock. We do not add a second `Instant` loop on top (that is
how you lie to yourself about noise). We do not report CPU cycles: `perf` /
cachegrind are Linux-only and not wired here.

## How to read the numbers

| File | Question it answers | Not a proof of |
| --- | --- | --- |
| `encodings` | Is Newton’s nested ADT on the handwritten floor? Do HashSet-id / `Box<dyn>` pay extra? | “faster than XState” |
| `apply` | What does `Runtime::apply` cost vs owned `step` on a real `perform`? | Absolute latency of *your* `enter` body |
| `harel` | What does LCA / `perform` / RTC drain cost in isolation? | I/O, serde, GUI |
| `compose` | `And<L,R>` vs a handwritten AND struct; stack `Cmd` vs heap spill; `Sub::diff` | Network subscribe/unsubscribe |

The crate’s performance claim is **zero-cost encoding**: `newton_runtime`
should sit next to `handwritten_fields`. Safety (illegal XOR does not
compile, snapshot is a triple) is the product; speed is “you did not pay
for an interpreter.”

If a **batch of 1024** lands in the picosecond band, LLVM folded the loop
(`n += 1024`). The batch harness sinks the counter **per tick**. A single
tick in the 0.3–0.7 ns band is a few instructions (plausible at ~4 GHz);
`string_id_set` / `gof_box_dyn` should be tens of nanoseconds.

`encoding_tick` includes Criterion’s per-iteration wrapper. Divide
`encoding_batch_1024` by 1024 for the tight-loop cost (LLVM inlines
`Runtime::apply` down to the field write).
