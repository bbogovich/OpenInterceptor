# Dependency policy

Third-party dependencies are **avoided by default**. One is added only when it solves a problem
that would be prohibitively expensive to build here. Every dependency is a liability carried
forever: it changes without our requirements, may be abandoned, and pulls in its own tree.

## Before adding one, state

- **Feature complexity** — genuinely hard, or just unfamiliar?
- **Stability** — release history, maintenance, number of maintainers.
- **Its dependency tree** — the transitive weight is part of the cost.
- **How much would be used** — a large crate for one function is never justified.
- **Licence** — MIT/Apache/BSD/zlib. **Never GPL**; LGPL only with great care.

The evaluation is recorded here *before* the dependency is added (a project rule in
`CLAUDE.md`). Versions are pinned exactly (`=x.y.z`) so an upstream release cannot change the
build silently; `Cargo.lock` is committed.

## Current dependencies (`Cargo.toml`)

| Crate | Features | Why |
|---|---|---|
| `macroquad =0.4.14` | default + `audio` | Window, OpenGL context, input, textures, 2D/3D drawing, sound playback. |
| `glam =0.27.0` | default | Vectors, matrices, quaternions for `model`, `sim` and `render`. |
| `image =0.24.9` | `png` only | Decoding PNG textures in mod packs. |

### Evaluation

| Question | macroquad | glam | image (png) |
|---|---|---|---|
| Feature complexity | Cross-platform windowing, GL contexts, input and audio back-ends are genuinely hard and platform-specific. | 3D maths is not hard, but a correct, fast, well-tested quaternion/matrix library is a lot of code to own. | PNG needs DEFLATE plus filters and chunk handling: modest but not worth owning. |
| Stability | Used by ArenaVM and Looking Glass on this machine with the same pinned version; maintained since 2020. | Very widely used, stable API, active maintenance. | Long-lived, widely used. |
| Dependency tree | miniquad, fontdue, glam, image (png, tga), quad-rand; the `audio` feature adds **quad-snd** (CoreAudio on macOS, ALSA on Linux — the Linux build needs `libasound2-dev`) and its small decoders. | **None added**: already in macroquad's tree at exactly this version. | **None added**: already in macroquad's tree at this version with the `png` feature. |
| How much is used | Most of what we use from it is core (window, input, drawing, materials, audio). | Most of `Vec2/Vec3/Vec4/Quat/Mat4`. | One function: `load_from_memory_with_format`. Justified only because it costs nothing extra. |
| Licence | MIT / Apache-2.0 | MIT / Apache-2.0 | MIT / Apache-2.0 |

`glam` and `image` are used **directly** (rather than through macroquad's re-exports) so that
the `content` and `sim` layers do not depend on the windowing library — they compile and are
tested without it in mind — at zero added build weight.

## Built in-crate instead

PE resources, XMSH, XTXR, TGA, WAV (PCM, MS ADPCM, IMA ADPCM), INI, OBJ/MTL, command-line
parsing, the gamepad layers (Objective-C shim compiled by `build.rs`; Linux joystick API with
`std`), random numbers for AI and effects, the starfield, and since Stage 2 the **YAML** reader
and writer, the **schema** validator and JSON Schema exporter, and the **condition language**
(lexer, parser, type checker, evaluator). Each is a few hundred lines with tests.

**YAML, evaluated (Stage 2).** Mod files moved to YAML. The usual crate, `serde_yaml`, is
unmaintained (archived by its author), and alternatives would bring `serde` and a YAML engine
into the tree for a small part of what they do. Full YAML also behaves in ways that surprise
modders (implicit typing such as `no` → false, anchors and aliases). The project's files need
mappings, lists, scalars, block text and comments — about 750 lines in `formats::yaml`, with
every value kept as text until the schema types it, so errors can name the file, line and key.
Decision: build in-crate ([DEV-030](../../implementation/deviations/index.md)).

**Console networking, evaluated (C1, app 0.4.0).** The tablet console needs an HTTP server, the
WebSocket protocol, SHA-1 and base64 for its handshake, and JSON. The usual crates (`tungstenite`
or an async web framework, `sha1`, `base64`, `serde_json`) would bring in `serde`, often an
async runtime, and their trees — for a server that serves three embedded files and exchanges
small JSON messages with one or two tablets. Only a fraction would be used: no TLS, no
keep-alive, no chunking, no request bodies, no fragmented messages. Written in-crate:
`remote::http` (~120 lines), `remote::ws` with SHA-1 and base64 (~260, tested against the FIPS
and RFC vectors), `remote::json` (~360) and threads from `std`. Decision: build in-crate; no
dependency added. The console page itself (`web/console/`) is plain custom elements and CSS
with no framework or build step.

## External tools

Programs the user may have installed, which the project calls but never links, bundles or
requires, are **not dependencies**.

**Blender (planned, MD10).** A mod may carry `.blend` files as the source of its models. When
Blender is installed, the Mod Designer runs it headless on save
(`blender -b file.blend --python-expr …`) to export OBJ/MTL or `.glb`; without it, the exported
files already in the pack are used. The engine never parses `.blend`, whose binary format changes
between Blender versions. Blender is GPL, but it runs as a separate program on the user's own
files and nothing of it enters the codebase. See
[R-MOD-054](../../specification/requirements/modding.md#r-mod-054-blender-sources).

**`.glb` (planned, MD10)** is read in-crate: the glTF JSON chunk with `remote::json` and the
binary buffers; no crate is added
([R-MOD-053](../../specification/requirements/modding.md#r-mod-053-single-file-models)).

## Tier 2 scripting, evaluated (Stage 3.10) — awaiting the user's decision

!!! warning "Awaiting the user's decision"
    This is the evaluation [R-MOD-015](../../specification/requirements/modding.md#r-mod-015-tier-2-scripting-decision)
    asks for **before** a second-tier script language is added. **No decision has been made.**
    The recommendation below is the evaluator's; the user decides. Nothing is added to
    `Cargo.toml` until then. GitHub issue #16.

**The question.** Tier 1 — behaviours as state machines, missions as definitions, and the
checked condition and action language (`script::*`) — carries both shipped campaigns. Tier 2 is
for what Tier 1 cannot express: logic with memory and branching, such as a boss with phases, a
custom ending, or an event chain that counts and remembers
([Format: behaviours and scripting › Tier 2](../../specification/design/format-behaviour.md#tier-2-scripts)).
Whatever the language, the [design](../../specification/design/format-behaviour.md#tier-2-scripts)
already fixes its contract: scripts are called through hooks, see **only** the documented query
and action API, run on the 60 Hz simulation tick, are deterministic (no clock, files or network;
random numbers only from the game's generator) and keep state only through the API, so a save
never serialises a script VM.

Three options:

- **(a) `mlua`** — Rust bindings to Lua 5.4 (or LuaJIT, or Luau), building the Lua C sources
  that it bundles (the `vendored` feature).
- **(b) `rhai`** — an embedded scripting language written in Rust.
- **(c) Grow `script::*`** — extend the in-crate condition and action language into a small
  statement language: local variables, `if` / `else`, bounded loops, functions, and calls into
  the same checked queries and actions.

**What could not be verified.** This evaluation was written without network access. Release
history, maintainer counts and dependency trees below are from the evaluator's knowledge of the
crates up to 2025, marked *(unverified)*; before adding either crate, check them on crates.io
and the repositories, and record the exact version and `cargo tree` output here. The lines-of-code
figures for (c) are counted from main `90653b9`; the estimate of its growth is *derived*.

| Question | (a) `mlua` | (b) `rhai` | (c) grow `script::*` |
|---|---|---|---|
| **Feature complexity** | A full language VM — garbage collector, coroutines, tables, closures, standard library — is genuinely hard; so is a *safe* binding over Lua's C API (errors that `longjmp`, userdata lifetimes, panics across the FFI boundary). Not something to build here. | The same scope in pure Rust: parser, AST optimiser, closures, modules, a dynamic value type. Genuinely hard to build, but more than Tier 2 needs. | Modest, and the hard parts exist: the lexer, recursive-descent parser, type checker and evaluator are ~540 lines in `script/mod.rs`, with the battle and object contexts (~160 lines each) and actions (~150). Statements, scopes, a call stack and an operation budget add an estimated **800 – 1 200 lines** with tests *(derived)*. The expensive part is design and documentation, not code. |
| **Stability** | Lua 5.4 itself: decades of slow, careful releases from a three-person team at PUC-Rio. `mlua`: a 2019 fork of `rlua`, still **0.x** — its minor releases (0.9 → 0.10 → 0.11) have changed the API *(unverified)*. One principal maintainer: **bus factor ≈ 1** *(unverified)*. LuaJIT is effectively one maintainer and stops at Lua 5.1 semantics. | 1.x since 2021 with frequent releases that have kept 1.x compatible *(unverified)*. Started by one author, maintained for several years mainly by one other in the `rhaiscript` organisation: **bus factor ≈ 1** *(unverified)*. | Ours. It changes only when this project changes it; no upstream can break the build. The cost is that we carry it forever. |
| **Transitive tree** | `mlua-sys` plus `lua-src` (or `luajit-src`) for the bundled C sources; at build time **`cc`** and **`pkg-config`**, and a **C compiler** on every platform (macOS already needs one for the gamepad shim; Linux and Windows builders would too). A handful of small Rust crates besides (for example `bstr`, `num-traits`, `rustc-hash`) *(unverified)*. C code in the process; cross-compiling and a WebAssembly build (macroquad can target the web) become harder. | Pure Rust, but **several crates**: for example `ahash` (with `getrandom`, `once_cell`, `zerocopy`), `smartstring`, `thin-vec`, `num-traits`, `bitflags`, a time crate, and the procedural macro crate `rhai_codegen` with `syn`, `quote`, `proc-macro2` — roughly 15 – 20 crates and noticeably longer clean builds *(unverified)*. | **None.** |
| **How much would be used** | A small fraction: one VM state, a dozen registered functions, a few userdata types, limits. Not async, serde, modules, the C module loader or most of the standard library. | A small fraction: `Engine`, compile to AST, registered functions, limits. Not modules, serde, metadata, the optimiser's levels or the codegen macros beyond registration. | All of it, by construction: only what modders need is built. |
| **Licence** | `mlua` MIT; Lua 5.4 MIT; LuaJIT MIT; Luau MIT. | MIT OR Apache-2.0. | The project's own (MIT). |
| **Sandboxing** | Possible but **opt-in**: load only safe libraries (no `io`, `os`, `package`, `debug`, `load` of bytecode), a memory limit, an instruction-count hook to stop runaway scripts. The VM is C: a bug there or in the binding is memory-unsafe in our process. **Determinism needs care**: Lua uses `f64` where the simulation uses `f32`, and `pairs()` order over string-keyed tables depends on a per-state hash seed that Lua 5.4 randomises by default *(derived)* — replays could diverge unless that is controlled. | Good: no file, network or clock access unless registered; limits on operations, call depth, string, array and map sizes, and a progress callback to abort. Memory-safe Rust. Maps are ordered, so iteration is deterministic *(unverified)*. Errors such as calling an unknown function surface **at run time**, not at load. | **By construction**: the only effects are the actions in the checked table, the only inputs the documented queries, values are `f32` like the simulation, and there is no I/O to remove. Every name, argument and type is **checked when the files load**, as conditions are today; an operation budget per hook call guarantees that a loop finishes. |
| **Modder experience** | Lua is the most widely known game-modding language; editors, tutorials and debuggers exist. | Rust-like syntax; few modders know it; good error messages. | One language from condition to script: a modder who writes `hull < 25%` today writes `if hull < 25% { message(@msg.retreat) }` tomorrow, and `--check` catches mistakes before play. No outside tutorials or debugger; the docs site must teach it. |
| **Performance** | Fast (LuaJIT very fast). | A tree-walking interpreter, several times slower than Lua *(unverified)*; fine for hooks, poor for per-tick logic on many ships. | Like today's conditions: a tree walk over pre-resolved query indices with no allocation. Fast enough for hooks; not a general-purpose fast path. |
| **Teaching value (objective 2)** | Shows FFI and embedding, which `platform` already shows. | Shows embedding a Rust library. | Shows how an interpreter grows from expressions to statements, scopes and a call stack — a classic exercise, in code already documented for beginners. |

**What (c) would be.** A *small* statement language over the same checked API, not a
general-purpose one:

```text
# a hook body (illustrative syntax, not a decided design)
let shots = 0
if hull < 25% and not system_damaged(engines) {
    message(@msg.guardian_enraged)
    for each member of guards { order(member, attack_player) }
} else {
    shots = shots + 1
}
def enraged() { return hull < 25% }
```

- Local variables (numbers, true/false, names) live for one hook call; anything that must last
  goes through actions (`set_flag`, counters, object variables), as the design requires.
- Loops are bounded: over a role's members or a counted range, plus an operation budget per call.
- Functions are defined in the pack and checked for their arguments; no recursion (or a fixed
  depth).
- Strings, tables, closures and libraries are deliberately out of scope.

**Its ceiling.** (c) cannot host what needs general-purpose data structures or algorithms: a
procedural generator, a custom AI planner, string processing, a mod's own data tables. If mods
need that, a real language is needed, and the choice between (a) and (b) comes back.

### Recommendation (the evaluator's; awaiting the user)

1. **Default: (c), grow `script::*`.** Under the dependency policy a dependency is added only
   when building the feature here would be prohibitively expensive. It would not be: the lexer,
   parser, checker and evaluator exist; mods already use the language; it is checked at load,
   deterministic and sandboxed by construction; and it adds nothing to the tree. Both shipped
   campaigns need only Tier 1, so the extension can be built when a mod first needs it.
2. **Fallback: (a), `mlua` with Lua 5.4 (vendored)**, if mods turn out to need a full
   general-purpose language. Lua is what modders know, it is fast, and its C core is among the
   most stable code there is; the costs — a C build, an FFI boundary, opt-in sandboxing and the
   determinism care above — are known and containable behind the same hook API. It would need
   the version, `cargo tree` and the sandbox and determinism settings recorded here first.
3. **(b) `rhai`** is not recommended now: it costs more crates than (a) for a language few
   modders know, with run-time rather than load-time errors. It is the option to reconsider if
   a pure-Rust or WebAssembly build becomes a requirement, where (a)'s C code is the obstacle.

Whatever is chosen, the hook API is the same, so a Tier 1 definition that calls a hook does not
change if the language behind it does.

## Next candidates

None pending. The Tier 2 language above awaits the user's decision.
