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
```

| Example | What it shows |
| --- | --- |
| `counter` | Elm loop only: `Machine` + `Runtime`, no hierarchy |
| `connection` | XOR tree, `perform` (LCA), history sidecar, `Cmd` as intent |
| `orthogonal` | AND node: two regions, one `Msg`, one RTC step |
| `replay` | Snapshot `{config, context, history}` → `MemoryStore` → restore |
| `storm` | `rtc` drain cap: a looping follow-up becomes `Storm` |

None of them open sockets. The host prints commands instead of executing I/O.
That is the family law: `update` returns data.
