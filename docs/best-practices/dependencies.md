# Dependency policy

Third-party dependencies are **avoided by default**. One is added only when it solves a problem
that would be prohibitively expensive to build here. Every dependency is a liability carried
forever: it changes without our requirements, may be abandoned, and pulls in its own tree.

## Current dependencies

| Crate | Why | Evaluation |
|---|---|---|
| `macroquad =0.4.14` | Window, OpenGL context, input, textures, audio on macOS, Linux and Windows | Cross-platform windowing and GL are genuinely hard; stable and used by ArenaVM and Looking Glass on this machine; MIT/Apache; moderate tree (miniquad, glam, quad-snd, image with png); pinned exactly |

Everything else is in-crate: PE resources, XMSH, XTXR, TGA, WAV + MS-ADPCM, INI, OBJ, the
gamepad layers. Each is a few hundred lines with tests.

## Before adding one, state

- **Feature complexity** — genuinely hard, or just unfamiliar?
- **Stability** — release history, maintenance, number of maintainers.
- **Its dependency tree** — the transitive weight is part of the cost.
- **How much would be used** — a large crate for one function is never justified.
- **Licence** — MIT/Apache/BSD/zlib. **Never GPL**; LGPL only with great care.

The first candidate is the Stage 3 scripting language (see [Scripting](../modding/scripting.md)).
