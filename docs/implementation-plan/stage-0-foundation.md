# Stage 0 — Foundation and analysis

**Status: done** (2026-11-10).

## Delivered

- Repository `OpenInterceptor` (MIT) with the SpecItOut manifest (`implementation`,
  `specVersion: null`), labels, `.gitignore` safety net, README and `scripts/check.sh`.
- This site on the `docs` branch, with `docs.sh`.
- Analysis of the CD: [layout](../original-game/cd-layout.md) and [formats](../original-game/formats/index.md).
- Prototype decoders (outside the repository) that parsed all meshes and rendered models with
  their textures, used to confirm the formats before engine code was written.
- Research into the original's gameplay and controls.
- The architecture, the dependency decision (macroquad only) and this plan.

## Decisions

| Decision | Why |
|---|---|
| Rust, one crate, macroquad 0.4.14 | Proven in ArenaVM and Looking Glass on this machine; OpenGL 3 on macOS and Linux; small. |
| All format readers in-crate | Each is a few hundred lines; no crate exists for these formats anyway. |
| INI for rules | The original uses INI, so original sections import directly; no parser dependency. |
| OBJ + PNG/TGA for mod art | Universal export targets; OBJ is trivial to parse. |
| Gamepad without a crate | macOS GameController shim (pattern from Looking Glass) and Linux joystick API with std. |
