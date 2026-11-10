//! # Rendering
//!
//! Draws one frame of the 3D scene with macroquad. Everything here *reads* the
//! simulation; nothing writes to it.
//!
//! ## Frame structure
//!
//! ```text
//! clear ─▶ sky pass (stars, sun; no depth) ─▶ solid pass (ships, stations, debris;
//!          lit shader, depth test) ─▶ glow pass (engines, bolts, missiles, explosions;
//!          additive blending, depth test but no depth write)
//! ```
//!
//! The HUD is drawn afterwards in 2D by [`crate::hud`].
//!
//! ## Materials (shaders)
//!
//! A *material* is a shader program plus GPU state (blending, depth). We use three:
//!
//! - **lit** — textured, vertex-coloured, with one directional light (the sun) plus
//!   ambient. Normals come in through macroquad's spare `normal` vertex attribute.
//! - **sky** — unlit, ignores and does not write depth, so stars sit behind everything.
//! - **glow** — additive (`src + dst`): overlapping glows brighten like real light.
//!
//! ## Allocation discipline
//!
//! The glow and sky geometry is rebuilt every frame into `Vec`s owned by the renderer
//! and cleared with `clear()` — which keeps their capacity — so after the first frame
//! no memory is allocated for them.

pub mod camera;
pub mod effects;
pub mod gpu;
pub mod starfield;

use crate::content::Content;
use crate::sim::{ShipId, World};
use camera::SpaceCamera;
use effects::{explosion_curve, Effects};
use glam::{Mat4, Quat, Vec2, Vec3};
use gpu::{GpuModel, ModelCache};
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Comparison, Equation, PipelineParams, UniformType};
use macroquad::prelude::*;
use std::rc::Rc;

const LIT_VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
attribute vec4 normal;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
uniform vec4 SunDirection;
uniform vec4 Tint;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    // w = 0 transforms a direction (rotation only, no translation).
    vec3 n = normalize((Model * vec4(normal.xyz, 0.0)).xyz);
    float diffuse = max(dot(n, SunDirection.xyz), 0.0);
    float light = 0.30 + 0.85 * diffuse;
    color = vec4(color0.rgb / 255.0 * light + Tint.rgb, color0.a / 255.0);
    uv = texcoord;
}"#;

const LIT_FRAGMENT: &str = r#"#version 100
varying lowp vec2 uv;
varying lowp vec4 color;
uniform sampler2D Texture;
void main() {
    gl_FragColor = color * texture2D(Texture, uv);
}"#;

const PLAIN_VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    color = color0 / 255.0;
    uv = texcoord;
}"#;

const PLAIN_FRAGMENT: &str = r#"#version 100
varying lowp vec2 uv;
varying lowp vec4 color;
uniform sampler2D Texture;
void main() {
    gl_FragColor = color * texture2D(Texture, uv);
}"#;

/// Distance at which sky objects are drawn (inside the far plane).
const SKY_DISTANCE: f32 = 150_000.0;

/// Owns GPU resources and draws scenes.
pub struct Renderer {
    lit: Material,
    sky: Material,
    glow: Material,
    /// A soft round spot, generated at start-up, used for every glow.
    glow_texture: Texture2D,
    pub models: ModelCache,
    stars: Vec<starfield::Star>,
    star_seed: u64,
    // Reused per-frame buffers (see the module docs).
    glow_vertices: Vec<Vertex>,
    glow_indices: Vec<u16>,
    sky_vertices: Vec<Vertex>,
    sky_indices: Vec<u16>,
}

/// Creates a radial-gradient texture: bright centre fading to transparent edge.
fn make_glow_texture() -> Texture2D {
    const N: u16 = 64;
    let mut rgba = Vec::with_capacity(N as usize * N as usize * 4);
    for y in 0..N {
        for x in 0..N {
            let dx = (x as f32 + 0.5) / N as f32 * 2.0 - 1.0;
            let dy = (y as f32 + 0.5) / N as f32 * 2.0 - 1.0;
            let d = (dx * dx + dy * dy).sqrt();
            let a = ((1.0 - d).max(0.0)).powf(1.8);
            let v = (a * 255.0) as u8;
            rgba.extend_from_slice(&[v, v, v, v]);
        }
    }
    let t = Texture2D::from_rgba8(N, N, &rgba);
    t.set_filter(FilterMode::Linear);
    t
}

/// Pushes a camera-facing square into vertex/index buffers. `center` is
/// camera-relative; `right`/`up` are the camera's axes.
pub fn push_billboard(
    verts: &mut Vec<Vertex>,
    idx: &mut Vec<u16>,
    center: Vec3,
    size: f32,
    right: Vec3,
    up: Vec3,
    color: [u8; 4],
) {
    if verts.len() + 4 > u16::MAX as usize {
        return; // buffer full this frame; skip rather than corrupt indices
    }
    let base = verts.len() as u16;
    let (r, u) = (right * size, up * size);
    for (p, uv) in [
        (center - r - u, Vec2::new(0.0, 1.0)),
        (center + r - u, Vec2::new(1.0, 1.0)),
        (center + r + u, Vec2::new(1.0, 0.0)),
        (center - r + u, Vec2::new(0.0, 0.0)),
    ] {
        verts.push(Vertex { position: p, uv, color, normal: Vec4::ZERO });
    }
    idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// Pushes a beam segment from `a` to `b` (camera-relative) as a quad that faces the
/// camera: its width direction is perpendicular to both the beam and the line of sight.
pub fn push_streak(verts: &mut Vec<Vertex>, idx: &mut Vec<u16>, a: Vec3, b: Vec3, width: f32, color: [u8; 4]) {
    if verts.len() + 4 > u16::MAX as usize {
        return;
    }
    let along = b - a;
    let side = along.cross((a + b) * 0.5).normalize_or_zero() * width;
    let base = verts.len() as u16;
    for (p, uv) in [
        (a - side, Vec2::new(0.0, 0.5)),
        (a + side, Vec2::new(1.0, 0.5)),
        (b + side, Vec2::new(1.0, 0.5)),
        (b - side, Vec2::new(0.0, 0.5)),
    ] {
        verts.push(Vertex { position: p, uv: Vec2::new(0.5, uv.y), color, normal: Vec4::ZERO });
    }
    idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// Converts an RGB colour (0..1) and brightness to vertex bytes.
pub fn rgb8(c: [f32; 3], k: f32) -> [u8; 4] {
    let f = |x: f32| (x * k * 255.0).clamp(0.0, 255.0) as u8;
    [f(c[0]), f(c[1]), f(c[2]), 255]
}

impl Renderer {
    /// Compiles shaders and creates GPU state. Fails only if the GPU rejects a shader.
    pub fn new() -> Result<Renderer, String> {
        let lit = load_material(
            ShaderSource::Glsl { vertex: LIT_VERTEX, fragment: LIT_FRAGMENT },
            MaterialParams {
                pipeline_params: PipelineParams {
                    depth_test: Comparison::LessOrEqual,
                    depth_write: true,
                    ..Default::default()
                },
                uniforms: vec![
                    UniformDesc::new("SunDirection", UniformType::Float4),
                    UniformDesc::new("Tint", UniformType::Float4),
                ],
                ..Default::default()
            },
        )
        .map_err(|e| format!("lit shader: {e:?}"))?;
        let additive = BlendState::new(Equation::Add, BlendFactor::One, BlendFactor::One);
        let sky = load_material(
            ShaderSource::Glsl { vertex: PLAIN_VERTEX, fragment: PLAIN_FRAGMENT },
            MaterialParams {
                pipeline_params: PipelineParams {
                    depth_test: Comparison::Always,
                    depth_write: false,
                    color_blend: Some(additive),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .map_err(|e| format!("sky shader: {e:?}"))?;
        let glow = load_material(
            ShaderSource::Glsl { vertex: PLAIN_VERTEX, fragment: PLAIN_FRAGMENT },
            MaterialParams {
                pipeline_params: PipelineParams {
                    depth_test: Comparison::LessOrEqual,
                    depth_write: false,
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Value(BlendValue::SourceAlpha),
                        BlendFactor::One,
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .map_err(|e| format!("glow shader: {e:?}"))?;
        Ok(Renderer {
            lit,
            sky,
            glow,
            glow_texture: make_glow_texture(),
            models: ModelCache::default(),
            stars: Vec::new(),
            star_seed: u64::MAX,
            glow_vertices: Vec::new(),
            glow_indices: Vec::new(),
            sky_vertices: Vec::new(),
            sky_indices: Vec::new(),
        })
    }

    /// The GPU model for a ship class (loaded on first use).
    pub fn ship_model(
        &mut self,
        content: &Content,
        world: &World,
        class: crate::content::rules::ShipClassId,
    ) -> Option<Rc<GpuModel>> {
        let c = world.rules.ship(class);
        self.models.get(content, &c.model, c.model_pack, c.model_scale)
    }

    /// Draws a model with the lit material, choosing the LOD by distance. Call after
    /// `begin_solid`.
    pub fn draw_model(&self, cam: &SpaceCamera, model: &GpuModel, position: Vec3, rotation: Quat, tint: Vec3) {
        self.draw_model_lod(cam, model, position, rotation, tint, None);
    }

    /// Like [`Renderer::draw_model`], but `force_lod = Some(i)` draws LOD `i` whatever
    /// the distance (used by the model viewer).
    pub fn draw_model_lod(
        &self,
        cam: &SpaceCamera,
        model: &GpuModel,
        position: Vec3,
        rotation: Quat,
        tint: Vec3,
        force_lod: Option<usize>,
    ) {
        let lod = match force_lod.and_then(|i| model.lods.get(i)) {
            Some(l) => l,
            None => model.lod(position.distance(cam.position)),
        };
        self.lit.set_uniform("Tint", tint.extend(0.0));
        let gl = unsafe_gl();
        gl.push_model_matrix(cam.model_matrix(position, rotation, 1.0));
        for mesh in &lod.meshes {
            draw_mesh(mesh);
        }
        gl.pop_model_matrix();
    }

    /// Starts the solid pass: sets the camera and the lit material.
    pub fn begin_solid(&self, cam: &SpaceCamera, sun: Vec3) {
        set_camera(cam);
        gl_use_material(&self.lit);
        self.lit.set_uniform("SunDirection", sun.normalize_or(Vec3::Y).extend(0.0));
    }

    /// Draws the stars and the sun.
    pub fn draw_sky(&mut self, cam: &SpaceCamera, seed: u64, sun: Vec3) {
        if seed != self.star_seed {
            self.stars = starfield::generate(seed, 2500);
            self.star_seed = seed;
        }
        set_camera(cam);
        self.sky_vertices.clear();
        self.sky_indices.clear();
        let right = cam.rotation * Vec3::X;
        let up = cam.rotation * Vec3::Y;
        // Star size scales with distance and field of view so a star covers ~1–2 pixels.
        let pixel = SKY_DISTANCE * (cam.fov_y * 0.5).tan() * 2.0 / screen_height().max(1.0);
        for s in &self.stars {
            push_billboard(
                &mut self.sky_vertices,
                &mut self.sky_indices,
                s.direction * SKY_DISTANCE,
                s.size * pixel,
                right,
                up,
                s.color,
            );
        }
        let sun_dir = sun.normalize_or(Vec3::Y);
        push_billboard(
            &mut self.sky_vertices,
            &mut self.sky_indices,
            sun_dir * SKY_DISTANCE,
            SKY_DISTANCE * 0.06,
            right,
            up,
            [255, 240, 200, 255],
        );
        push_billboard(
            &mut self.sky_vertices,
            &mut self.sky_indices,
            sun_dir * SKY_DISTANCE,
            SKY_DISTANCE * 0.2,
            right,
            up,
            [90, 70, 40, 255],
        );
        gl_use_material(&self.sky);
        // `Mesh` owns its Vecs, so we *move* our buffers into it with `mem::take`
        // (leaving empty Vecs behind), draw — `draw_mesh` copies the data into
        // macroquad's batch — and then move them back, keeping their capacity.
        let mesh = Mesh {
            vertices: std::mem::take(&mut self.sky_vertices),
            indices: std::mem::take(&mut self.sky_indices),
            texture: Some(self.glow_texture.clone()),
        };
        draw_mesh(&mesh);
        self.sky_vertices = mesh.vertices;
        self.sky_indices = mesh.indices;
        gl_use_default_material();
    }

    /// Draws every live ship (except `hide`, e.g. the player in cockpit view), debris.
    pub fn draw_world(
        &mut self,
        content: &Content,
        world: &World,
        effects: &Effects,
        cam: &SpaceCamera,
        sun: Vec3,
        hide: Option<ShipId>,
    ) {
        self.begin_solid(cam, sun);
        for i in 0..world.ships.len() {
            let s = &world.ships[i];
            if !s.alive || Some(world.id_of(i)) == hide {
                continue;
            }
            let Some(model) = self.ship_model(content, world, s.class) else { continue };
            // Cull objects that are entirely behind the camera.
            let rel = s.position - cam.position;
            if rel.dot(cam.rotation * Vec3::NEG_Z) < -model.radius {
                continue;
            }
            // Brief red flash when hit.
            let flash = (1.0 - s.since_hit / 0.15).max(0.0) * 0.5;
            self.draw_model(cam, &model, s.position, s.rotation, Vec3::new(flash, flash * 0.3, 0.0));
        }
        for d in &effects.debris {
            if let Some(model) = self.models.get(content, &d.model, d.pack, d.scale) {
                let fade = 1.0 - (d.age / effects::DEBRIS_LIFE);
                self.draw_model(cam, &model, d.position, d.rotation, Vec3::splat((fade - 1.0) * 0.3));
            }
        }
        gl_use_default_material();
    }

    /// Draws engine glows, bolts, missiles, sparks and explosions.
    pub fn draw_glows(
        &mut self,
        content: &Content,
        world: &World,
        effects: &Effects,
        cam: &SpaceCamera,
        hide: Option<ShipId>,
    ) {
        set_camera(cam);
        self.glow_vertices.clear();
        self.glow_indices.clear();
        let right = cam.rotation * Vec3::X;
        let up = cam.rotation * Vec3::Y;
        let rel = |p: Vec3| p - cam.position;
        for i in 0..world.ships.len() {
            let s = &world.ships[i];
            if !s.alive || Some(world.id_of(i)) == hide {
                continue;
            }
            let class = world.rules.ship(s.class);
            if class.max_speed <= 0.0 {
                continue;
            }
            let Some(model) = self.ship_model(content, world, s.class) else { continue };
            let power = if s.afterburning { 1.6 } else { 0.35 + 0.65 * s.throttle };
            for p in &model.engines {
                let world_p = s.position + s.rotation * *p;
                let size = model.radius * 0.25 * power;
                push_billboard(
                    &mut self.glow_vertices,
                    &mut self.glow_indices,
                    rel(world_p),
                    size,
                    right,
                    up,
                    rgb8(class.engine_color, 0.9 * power.min(1.0)),
                );
            }
        }
        for b in &world.bolts {
            let tail = b.position - b.velocity.normalize_or_zero() * (b.velocity.length() * 0.025).min(80.0);
            let dist = rel(b.position).length();
            let width = (dist * 0.0025).max(0.6);
            push_streak(
                &mut self.glow_vertices,
                &mut self.glow_indices,
                rel(tail),
                rel(b.position),
                width,
                rgb8(b.color, 1.0),
            );
            push_billboard(
                &mut self.glow_vertices,
                &mut self.glow_indices,
                rel(b.position),
                width * 3.0,
                right,
                up,
                rgb8(b.color, 0.6),
            );
        }
        for m in &world.missiles {
            let c = world.rules.missile(m.class).color;
            let dist = rel(m.position).length();
            push_billboard(
                &mut self.glow_vertices,
                &mut self.glow_indices,
                rel(m.position),
                (dist * 0.006).max(2.5),
                right,
                up,
                rgb8(c, 1.0),
            );
            let tail = m.position - m.velocity.normalize_or_zero() * 30.0;
            push_streak(
                &mut self.glow_vertices,
                &mut self.glow_indices,
                rel(tail),
                rel(m.position),
                (dist * 0.002).max(0.5),
                rgb8(c, 0.4),
            );
        }
        for s in &effects.sparks {
            let k = 1.0 - s.age / effects::SPARK_LIFE;
            let c = if s.shield { [0.4, 0.7, 1.0] } else { [1.0, 0.7, 0.3] };
            push_billboard(
                &mut self.glow_vertices,
                &mut self.glow_indices,
                rel(s.position),
                6.0 + 10.0 * k,
                right,
                up,
                rgb8(c, k),
            );
        }
        for x in &effects.explosions {
            let (size, bright) = explosion_curve(x.age / x.life);
            push_billboard(
                &mut self.glow_vertices,
                &mut self.glow_indices,
                rel(x.position),
                x.radius * size,
                right,
                up,
                rgb8([1.0, 0.6, 0.25], bright),
            );
            push_billboard(
                &mut self.glow_vertices,
                &mut self.glow_indices,
                rel(x.position),
                x.radius * size * 0.5,
                right,
                up,
                rgb8([1.0, 0.95, 0.8], bright),
            );
        }
        if !self.glow_indices.is_empty() {
            gl_use_material(&self.glow);
            let mesh = Mesh {
                vertices: std::mem::take(&mut self.glow_vertices),
                indices: std::mem::take(&mut self.glow_indices),
                texture: Some(self.glow_texture.clone()),
            };
            draw_mesh(&mesh);
            // Hand the buffers back so their capacity is reused next frame.
            self.glow_vertices = mesh.vertices;
            self.glow_indices = mesh.indices;
            gl_use_default_material();
        }
    }

    /// Draws a 3D cockpit around the pilot's eye (the camera), always in front of the
    /// world: the depth buffer is cleared first, and a near plane of 5 cm lets the
    /// geometry sit right in front of the eye. `offset` is in the eye's frame.
    pub fn draw_cockpit_model(&self, cam: &SpaceCamera, model: &GpuModel, offset: Vec3, sun: Vec3) {
        let mut gl = unsafe_gl_context();
        // Everything batched so far must reach the GPU before the depth is cleared.
        gl.flush();
        gl.quad_context.clear(None, Some(1.0), None);
        let near = cam.with_near(0.05);
        self.begin_solid(&near, sun);
        self.draw_model_lod(&near, model, cam.position + cam.rotation * offset, cam.rotation, Vec3::ZERO, Some(0));
        gl_use_default_material();
    }

    /// A small 3D marker (used by the model viewer for attachment points).
    pub fn draw_axes(&self, cam: &SpaceCamera, at: Vec3, rot: Quat, size: f32) {
        set_camera(cam);
        let o = at - cam.position;
        draw_line_3d(o, o + rot * Vec3::X * size, RED);
        draw_line_3d(o, o + rot * Vec3::Y * size, GREEN);
        draw_line_3d(o, o + rot * Vec3::NEG_Z * size, BLUE);
    }
}

/// Access to macroquad's low-level GL state for the model-matrix stack.
fn unsafe_gl() -> &'static mut macroquad::prelude::QuadGl {
    unsafe_gl_context().quad_gl
}

/// Access to macroquad's internal GL context (batcher + raw rendering backend).
///
/// This is the project's only non-FFI `unsafe`: `get_internal_gl` is marked unsafe
/// because the returned references must not be held across `next_frame().await`.
fn unsafe_gl_context() -> macroquad::window::InternalGlContext<'static> {
    // SAFETY: called during drawing on the main thread; callers use the references
    // immediately and never keep them past the current frame.
    unsafe { get_internal_gl() }
}

/// Identity transform, re-exported for callers drawing in camera space.
pub fn identity() -> Mat4 {
    Mat4::IDENTITY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn billboards_and_streaks_build_two_triangles() {
        let (mut v, mut i) = (Vec::new(), Vec::new());
        push_billboard(&mut v, &mut i, Vec3::new(0.0, 0.0, -10.0), 2.0, Vec3::X, Vec3::Y, [255; 4]);
        assert_eq!((v.len(), i.len()), (4, 6));
        assert_eq!(v[2].position, Vec3::new(2.0, 2.0, -10.0));
        push_streak(&mut v, &mut i, Vec3::new(0.0, 0.0, -10.0), Vec3::new(0.0, 0.0, -20.0), 1.0, [255; 4]);
        assert_eq!(i[6..], [4, 5, 6, 4, 6, 7]);
        // A streak pointing straight away from the camera degenerates gracefully.
        assert!(v[4..].iter().all(|x| x.position.is_finite()));
        let side = v[5].position - v[4].position;
        assert!(side.z.abs() < 1e-6, "width is perpendicular to the beam");
    }

    #[test]
    fn colour_helpers_clamp() {
        assert_eq!(rgb8([1.0, 0.5, 2.0], 1.0), [255, 127, 255, 255]);
        assert_eq!(rgb8([1.0, 1.0, 1.0], 0.0), [0, 0, 0, 255]);
        assert_eq!(identity(), Mat4::IDENTITY);
    }
}
