# Examples

These programs live on GitHub so people can read and run them. They are **not**
part of the crates.io package (`newton-machine` on docs.rs is the library only).

Requires the default `std` feature (printing).

```bash
cargo run --example counter
cargo run --example connection
cargo run --example orthogonal
cargo run --example replay
cargo run --example storm
cargo run --example aapl_1m --release
```

The first two files are walkthroughs (module docs + inline comments). Start
there. They also spell out **Newton vs Harel**: same XOR/AND/LCA/RTC geometry,
different laws (typed config, `Cmd` as data, history sidecar).

**LCA** = least common ancestor in the node tree: what `perform` exits and
enters so a parent you are staying in is not torn down. **TEA runtime** =
`Runtime` owning `{config, context, history}` with `apply` as the only door
(not Tokio). **RTC drain cap** = max internal follow-ups per external `Msg`
(`storm` example). **Nested ADTs** = `enum`/`struct` inside each other so the
type *is* the configuration.

| Example | What it shows |
| --- | --- |
| `counter` | Elm loop only: `Machine` + `Runtime`, no hierarchy. Read this first. |
| `connection` | XOR tree, `perform` (LCA), history sidecar, `Cmd` as intent |
| `orthogonal` | `And<Auth, Sync>`: first-class Harel AND, one `Msg`, one RTC clock |
| `replay` | Snapshot `{config, context, history}` → `MemoryStore` → restore |
| `storm` | RTC drain cap: a looping follow-up becomes `Storm` |
| `aapl_1m` | April 2026 AAPL 1m: HOST EMA 9/21/50/200 + fast stoch 9/14/40/60 (const packs, no TA crate). Overlaps/scores are **host policy**. Needs the CSV. |

None of them open sockets. The host prints commands instead of executing I/O.
That is the family law: `update` returns data.
