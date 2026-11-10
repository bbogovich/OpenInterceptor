# Best practices

A secondary goal of OpenInterceptor is to be a **worked example of Rust practice for a 3D
application or game engine**. These pages state the practices the code follows, and why. Code
review checks them; where the code falls short, that is a bug.

| Page | Covers |
|---|---|
| [Architectural design](architecture.md) | Layering, data-driven design, sim/render separation, fixed timestep |
| [Coding patterns](patterns.md) | Newtype ids, plain-data structs, explicit GPU ownership, no global mutable state, error types per layer |
| [Data model and performance](data-model.md) | SoA vs AoS, index handles, baked meshes, LOD, no per-frame allocation, precision |
| [Dependency policy](dependencies.md) | Why one dependency, and how a second would be justified |
| [Commenting standard](comments.md) | `//!`, `///`, and comments on non-obvious lines |
| [Testing policy](testing-policy.md) | Unit tests for all code; manual cases for playtest-only behaviour |
