# newton-machine

[![crates.io](https://img.shields.io/crates/v/newton-machine.svg)](https://crates.io/crates/newton-machine)
[![docs.rs](https://docs.rs/newton-machine/badge.svg)](https://docs.rs/newton-machine)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/newton-machine.svg)](LICENSE-MIT)

**Newtonian state machines** for Rust: Harel configuration as nested ADTs, the Elm Architecture as the only mutation protocol, and history as an inertial sidecar.

A Newton machine is not a new automaton. Harel already defined the kinematics (XOR/AND, hierarchy, history, run-to-completion). This crate names the **dynamics and conservation laws** for embedding those kinematics in a running Rust system: Unidirectional Configuration Architecture (UCA).

> A Newton machine may lock itself. Only the gateway may lock the wire. Only the venue can save you when both are dead.

**Status:** `0.1.0` is the engine (TEA `Runtime`, Harel LCA, RTC drain cap, history sidecar). The API is **not** SemVer-stable. There is no proc-macro and no SCXML interpreter.

## Install

```toml
[dependencies]
newton-machine = "0.1"
```

```bash
cargo add newton-machine
```

MSRV **1.80**. `#![no_std]` + `alloc` supported. `unsafe` is forbidden.

## Four laws

1. **The configuration is the type.** XOR is an `enum`. AND is a `struct`. Illegal simultaneous children are unrepresentable. No string ids on the hot path.
2. **The only mutation protocol is TEA.** `Msg → (config, context, Cmd)`. Entry/exit/transition actions emit data. The host executes I/O.
3. **Memory is inertial and external.** History is a sidecar of discriminants and small snapshots, not live variants and not closures. The persistable phase space is exactly `{config, context, history}`.
4. **Typestate is a façade, not the machine.** Public phase types may hide illegal methods. The interior stays a configuration tree.

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

`Runtime` owns `{config, context, history}`. `apply` is the hot path. `step` is the Elm-pure owned variant. The host executes `Cmd`; the crate never opens a socket.

## Configuration encoding

```rust
enum Conn {
    Offline,
    Connecting { attempt: u8 },
    Online(Session),
}

struct Session {
    auth: Auth, // XOR region
    sync: Sync, // XOR region; orthogonal to auth
}

enum Auth { Anonymous, SignedIn { user: u64 } }
enum Sync { Idle, Fetching, Dirty }
```

Orthogonal regions are fields, not threads. That is Harel virtual concurrency: one event, several regions, defined document order, one run-to-completion clock.

## Harel kinematics (handwritten)

Implement [`Topology`](https://docs.rs/newton-machine/latest/newton_machine/trait.Topology.html) (parent of each node id) and [`Transitional`](https://docs.rs/newton-machine/latest/newton_machine/trait.Transitional.html) (exit/enter). [`perform`](https://docs.rs/newton-machine/latest/newton_machine/fn.perform.html) exits inner-first, assigns the destination, enters outer-first.

[`rtc`](https://docs.rs/newton-machine/latest/newton_machine/fn.rtc.html) drains internal follow-ups with a cap of 32 (`Storm` if a loop would hang the host).

## Examples (GitHub only)

Runnable demos live in [`examples/`](examples/) in this repository. They are **not** included in the crates.io crate (docs.rs is the library).

```bash
cargo run --example counter      # TEA only
cargo run --example connection   # XOR + history
cargo run --example orthogonal   # AND regions
cargo run --example replay       # snapshot restore
cargo run --example storm        # RTC drain cap
```

See [`examples/README.md`](examples/README.md).

## When to use a Newton machine

Use a **Harel chart** when the primary artifact is a specification (diagrams, SCXML interchange, multi-language codegen).

Use a **Newton machine** when the primary artifact is a running Rust system that must not lie about its state: live, replay, and test share one `update`; snapshots contain no closures; invalid XOR children fail to compile.

Do not use either when a three-variant enum and a `match` will do.

## What this crate will never do

- Interpret SCXML documents at runtime.
- Open sockets, place orders, or perform I/O inside `update`.
- Run one thread per orthogonal region.
- Store closures in a snapshot.
- Be a risk gateway. A Newton machine may *request* lock. Only a host firewall may *admit* a command to the wire.

## Features

| Feature | Default | Role |
| --- | --- | --- |
| `std` | yes | `std::error::Error` for `Storm`; implies `alloc` |
| `alloc` | via `std` | `Cmd::Batch`, `Sub::Many`, `Tape` vec |
| `serde` | no | derives on snapshot types |

## Crate layout

| Piece | Owns |
| --- | --- |
| **`newton-machine`** (this crate) | UCA laws and engine |
| `examples/` (GitHub) | runnable demos, not on crates.io |
| `docs/` (GitHub) | rustbrain ADRs / goals / concepts |
| host application | gateway, watchdog, broker, persistence backend |

## Architecture notes (GitHub)

The in-repo second brain is [rustbrain](https://docs.rs/rustbrain) under `docs/`:

```bash
rustbrain context "why newton machine"
rustbrain query "history sidecar" --type adr,concept --scores
```

## License

MIT OR Apache-2.0
