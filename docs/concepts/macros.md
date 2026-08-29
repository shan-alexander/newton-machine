---
tags: [macros, agents]
node_type: concept
---
# Macros (feature `macros`)

The public guide lives in rustdoc so crates.io / docs.rs / agents share one page: symbol:macros (crate module `newton_machine::macros`).

Git repo = Cargo **workspace** (`newton-machine` + `newton-machine-macros`). Users `cargo add newton-machine --features macros`. Never add the macros crate.

See [[docs/adr/0024-macros-feature-hidden-proc-macro]].
