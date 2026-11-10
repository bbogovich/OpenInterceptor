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
`std`), random numbers for AI and effects, the starfield. Each is a few hundred lines with
tests.

The next candidate is the Stage 3 scripting language (see [Scripting](../modding/scripting.md)).
