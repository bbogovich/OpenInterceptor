# Scripting (planned)

Missions (Stage 3) and the campaign (Stage 4) need logic that data alone cannot express:
"when the freighter reaches the gate, spawn the second wave"; "when research X completes, open
system Y". That logic will be **scripts** in packs.

## Decision pending: the language

| Option | For | Against |
|---|---|---|
| **Lua 5.4 via `mlua`** (proposed) | The modding standard; modders already know it; fast; small; MIT | A C library (vendored and compiled by cargo); `mlua` plus its build dependencies; unsafe FFI boundary |
| Rhai | Pure Rust; easy embedding; MIT/Apache | Slower; fewer modders know it; heavier dependency tree than it looks |
| Own small language | No dependency | Months of work, poor tooling, everyone must learn it |

Under the [dependency policy](../best-practices/dependencies.md), a scripting language is the
kind of problem that is prohibitively expensive to build in-house, so a dependency is justified;
the choice is made at the start of Stage 3 after evaluating each candidate's stability,
dependency tree and licence. The script API will be designed so that the INI rules stay the
primary interface and scripts only add behaviour.
