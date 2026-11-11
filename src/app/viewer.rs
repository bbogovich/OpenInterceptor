//! # Model viewer
//!
//! Browse every model: each ship class's model (so modders see their pack exactly as the
//! game will), and — with original data — every raw mesh on the CD, including debris
//! chunks and unused objects. An orbit camera circles the model; attachment points are
//! shown as small axes; LODs can be forced to compare detail levels.
//!
//! It doubles as a mesh-debugging tool:
//!
//! - **W** cycles solid / solid + wireframe / wireframe only. The wireframe draws each
//!   original face's outline (not the triangles it was split into) and a dot on every
//!   vertex, so faces can be counted and told apart.
//! - **[ / ]** step through the faces one at a time (Shift: ten at a time). The
//!   selected face is outlined and its number, texture, flags, overlay layer and texture
//!   coordinates are shown — enough to report "face 12 of A-SHIPA5 is mis-mapped".
//! - **I** labels every face facing the camera with its number.
//! - **O** hides faces flagged as overlays, showing what lies beneath (stations use
//!   overlays for panels that may depend on what has been built).

use super::{AppContext, Next};
use crate::model::{face_plane, ModelLod};
use crate::render::camera::SpaceCamera;
use glam::{Quat, Vec3};
use macroquad::prelude::*;

/// An orbiting camera around the origin (pure maths, tested).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Orbit {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

impl Orbit {
    /// Camera position and rotation looking at the origin.
    pub fn pose(&self) -> (Vec3, Quat) {
        let rot = Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(-self.pitch);
        (rot * Vec3::new(0.0, 0.0, self.distance), rot)
    }

    /// Applies input (radians per second / zoom factor per second) for `dt` seconds.
    pub fn update(&mut self, yaw: f32, pitch: f32, zoom: f32, dt: f32) {
        self.yaw += yaw * dt;
        self.pitch = (self.pitch + pitch * dt).clamp(-1.5, 1.5);
        self.distance *= (1.0 - zoom * dt).clamp(0.5, 1.5);
    }
}

/// Direction of a mouse-wheel movement: −1, 0 or +1.
///
/// Beware: Rust's `f32::signum(0.0)` is `1.0`, not `0.0` (IEEE-754 zero has a sign), so
/// `wheel.signum()` would report "scrolling up" every frame the wheel is still.
pub fn wheel_steps(wheel: f32) -> f32 {
    if wheel == 0.0 {
        0.0
    } else {
        wheel.signum()
    }
}

/// How the model is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Display {
    Solid,
    SolidWire,
    Wire,
}

impl Display {
    /// The next mode (the W key).
    pub fn next(self) -> Display {
        match self {
            Display::Solid => Display::SolidWire,
            Display::SolidWire => Display::Wire,
            Display::Wire => Display::Solid,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Display::Solid => "solid",
            Display::SolidWire => "solid + wireframe",
            Display::Wire => "wireframe",
        }
    }
}

/// The near clipping plane for an orbit at `distance` around a model of `radius`.
///
/// Depth-buffer precision is spread between the near and far planes very unevenly —
/// most of it sits just beyond the near plane. A 1 m near plane looking at a 600 m
/// station 1.7 km away leaves too little precision at the station for nearly-coplanar
/// faces, so the viewer pushes the near plane out to just in front of the model.
pub fn near_plane(distance: f32, radius: f32) -> f32 {
    (distance - radius * 1.2).max(distance * 0.01).max(0.05)
}

/// One line of text describing face `index` of `lod`.
pub fn face_info(lod: &ModelLod, index: usize) -> String {
    let Some(f) = lod.faces.get(index) else { return String::new() };
    let corners = lod.face_vertices(f);
    let texture =
        lod.groups.get(f.group as usize).and_then(|g| g.texture.clone()).unwrap_or_else(|| "(flat colour)".into());
    let uvs: Vec<String> = corners.iter().map(|v| format!("({:.2},{:.2})", v.uv.x, v.uv.y)).collect();
    format!(
        "face {} of {}   {} corners   texture {}   flags 0x{:08x}   layer {}   uv {}",
        index,
        lod.faces.len(),
        f.count,
        texture,
        f.flags,
        corners.first().map(|v| v.layer).unwrap_or(0),
        uvs.join(" ")
    )
}

/// One model in the list.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewerEntry {
    pub label: String,
    pub reference: String,
    pub pack: usize,
    pub scale: f32,
    /// The class's model set (class entries only), for the D key's looks.
    pub set: Option<crate::content::looks::ModelSetId>,
}

/// One look of a model set, cycled with D.
#[derive(Debug, Clone, PartialEq)]
pub enum Look {
    Intact,
    /// A damage level (index into the set's `damage`).
    Damage(usize),
    /// A named state (index into the set's `states`).
    State(usize),
    /// The debris pieces, pulled apart.
    Debris,
}

/// Every look a model set can show, in the order D cycles them.
pub fn looks_of(set: &crate::content::looks::ModelSet) -> Vec<Look> {
    let mut v = vec![Look::Intact];
    v.extend((0..set.damage.len()).map(Look::Damage));
    v.extend((0..set.states.len()).map(Look::State));
    if set.debris.is_some() {
        v.push(Look::Debris);
    }
    v
}

/// A look's name for the info line.
pub fn look_label(set: &crate::content::looks::ModelSet, look: &Look) -> String {
    match look {
        Look::Intact => "intact".into(),
        Look::Damage(i) => format!("damage: hull below {:.0}%", set.damage[*i].below * 100.0),
        Look::State(i) => format!("state: {}", set.states[*i].0),
        Look::Debris => "debris (pieces pulled apart)".into(),
    }
}

/// The texture swaps a look shows.
pub fn look_swaps(set: &crate::content::looks::ModelSet, look: &Look) -> crate::content::looks::Swaps {
    match look {
        Look::Damage(i) => set.swaps_for(set.damage[*i].below - 1e-4, None),
        Look::State(i) => set.swaps_for(1.0, Some(&set.states[*i].0)),
        _ => set.swaps_for(1.0, None),
    }
}

/// The viewer screen.
pub struct ModelViewer {
    pub entries: Vec<ViewerEntry>,
    pub selected: usize,
    orbit: Orbit,
    /// `None` = automatic LOD by distance; `Some(i)` forces LOD `i`.
    forced_lod: Option<usize>,
    /// The entry the camera distance was last fitted to (fit again when it changes).
    fitted: Option<usize>,
    spin: bool,
    angle: f32,
    display: Display,
    /// The selected face (index into the drawn LOD's faces), if any.
    face: Option<usize>,
    labels: bool,
    /// Index into [`looks_of`] for the selected class.
    look: usize,
}

/// Builds the list: ship classes first, then raw original meshes.
pub fn entries(content: &crate::content::Content) -> Vec<ViewerEntry> {
    let mut v: Vec<ViewerEntry> = content
        .rules
        .ships
        .iter()
        .map(|s| ViewerEntry {
            label: format!("{} ({})", s.name, s.id),
            reference: s.model.clone(),
            pack: s.model_pack,
            scale: s.model_scale,
            set: Some(s.model_set),
        })
        .collect();
    v.extend(content.original_mesh_names().into_iter().map(|n| ViewerEntry {
        label: format!("mesh {n}"),
        reference: format!("original:{n}"),
        pack: 0,
        scale: 1.0,
        set: None,
    }));
    v
}

/// Finds the entry for `name`: a ship class id or an original mesh name, ignoring case
/// and the quote marks original resource names carry.
pub fn find_entry(entries: &[ViewerEntry], name: &str) -> Option<usize> {
    let want = name.trim_matches('"').to_ascii_lowercase();
    entries.iter().position(|e| {
        let label = e.label.to_ascii_lowercase();
        label.ends_with(&format!("({want})")) || label.trim_start_matches("mesh ").trim_matches('"') == want
    })
}

impl ModelViewer {
    /// A viewer opened on the model `name` (see [`find_entry`]); the first model if
    /// there is no such model.
    pub fn open_at(ctx: &AppContext, name: &str) -> ModelViewer {
        let mut v = ModelViewer::new(ctx);
        match find_entry(&v.entries, name) {
            Some(i) => v.selected = i,
            None => eprintln!("warning: no model '{name}' (see the viewer's list)"),
        }
        v
    }

    pub fn new(ctx: &AppContext) -> ModelViewer {
        ModelViewer {
            entries: entries(&ctx.content),
            selected: 0,
            orbit: Orbit { yaw: 0.6, pitch: 0.35, distance: 30.0 },
            forced_lod: None,
            fitted: None,
            spin: true,
            angle: 0.0,
            display: Display::Solid,
            face: None,
            labels: false,
            look: 0,
        }
    }

    pub fn frame(&mut self, ctx: &mut AppContext) -> Next {
        let raw = ctx.devices.read();
        let pressed = |k: &str| raw.pressed.contains(k);
        let held = |k: &str| raw.held.contains(k);
        let n = self.entries.len().max(1);
        let mut changed = false;
        if pressed("pagedown") || pressed("n") {
            self.selected = (self.selected + 1) % n;
            changed = true;
        }
        if pressed("pageup") || pressed("b") {
            self.selected = (self.selected + n - 1) % n;
            changed = true;
        }
        if pressed("escape") {
            ctx.renderer.show_overlays = true; // leave the shared renderer as we found it
            return Next::Launcher;
        }
        if changed {
            self.face = None;
            self.look = 0;
        }
        if pressed("d") {
            self.look += 1;
        }
        if pressed("w") {
            self.display = self.display.next();
        }
        if pressed("i") {
            self.labels = !self.labels;
        }
        if pressed("o") {
            ctx.renderer.show_overlays = !ctx.renderer.show_overlays;
        }
        let face_step = if raw.shift { 10 } else { 1 };
        let face_move = pressed("rightbracket") as isize - pressed("leftbracket") as isize;
        if pressed("l") {
            self.forced_lod = match self.forced_lod {
                None => Some(0),
                Some(i) if i < 3 => Some(i + 1),
                Some(_) => None,
            };
        }
        if pressed("space") {
            self.spin = !self.spin;
        }
        let dt = get_frame_time();
        let axis = |a: &str, b: &str| (held(b) as i32 - held(a) as i32) as f32;
        let zoom = axis("minus", "equal") + wheel_steps(mouse_wheel().1) * 4.0;
        self.orbit.update(axis("left", "right") * 1.5, axis("down", "up") * 1.0, zoom * 1.5, dt);
        if is_mouse_button_down(MouseButton::Left) {
            let d = mouse_delta_position();
            self.orbit.yaw += d.x * 3.0;
            self.orbit.pitch = (self.orbit.pitch - d.y * 3.0).clamp(-1.5, 1.5);
        }
        if self.spin {
            self.angle += dt * 0.3;
        }

        clear_background(Color::new(0.02, 0.02, 0.05, 1.0));
        let Some(entry) = self.entries.get(self.selected).cloned() else {
            draw_text("No models.", 40.0, 60.0, 30.0, WHITE);
            return Next::Stay;
        };
        // The look to show (D cycles them): a model set's damage levels, states, debris.
        let set = entry.set.map(|id| ctx.content.rules.model_set(id).clone());
        let looks = set.as_ref().map(looks_of).unwrap_or_else(|| vec![Look::Intact]);
        let look = looks[self.look % looks.len()].clone();
        let model = match &set {
            Some(set) => {
                let swaps = look_swaps(set, &look);
                let state = match &look {
                    Look::State(i) => Some(set.states[*i].0.as_str()),
                    _ => None,
                };
                ctx.renderer.models.get_variant(&ctx.content, set.model_for(state), set.pack, set.scale, &swaps)
            }
            None => ctx.renderer.models.get(&ctx.content, &entry.reference, entry.pack, entry.scale),
        };
        if let Some(m) = &model {
            if changed || self.fitted != Some(self.selected) {
                self.orbit.distance = (m.radius * 2.8).max(5.0);
                self.fitted = Some(self.selected);
            }
            // Never let the camera enter the model.
            self.orbit.distance = self.orbit.distance.max(m.radius * 1.05);
            let (pos, rot) = self.orbit.pose();
            let cam = SpaceCamera::new(pos, rot, 0.8, screen_width() / screen_height())
                .with_near(near_plane(self.orbit.distance, m.radius));
            let sun = Vec3::new(-0.4, 0.7, 0.6);
            ctx.renderer.draw_sky(&cam, 9, sun);
            let lod_index =
                self.forced_lod.filter(|&i| i < m.lods.len()).unwrap_or_else(|| m.lod_index(self.orbit.distance));
            let lod = &m.model.lods[lod_index];
            if face_move != 0 && !lod.faces.is_empty() {
                let n = lod.faces.len() as isize;
                let current = self.face.map(|f| f as isize).unwrap_or(if face_move > 0 { -1 } else { 0 });
                self.face = Some((current + face_move * face_step).rem_euclid(n) as usize);
            }
            let model_rot = Quat::from_rotation_y(self.angle);
            if let (Look::Debris, Some(d)) = (&look, set.as_ref().and_then(|s| s.debris)) {
                // Each piece sits where it belongs in the object; push it out along the
                // direction of its own centre so the cut lines show.
                let debris = ctx.content.rules.debris_set(d).clone();
                ctx.renderer.begin_solid(&cam, sun);
                for piece in &debris.pieces {
                    if let Some(p) = ctx.renderer.models.get(&ctx.content, piece, debris.pack, entry.scale) {
                        let lod = p.model.lods.first();
                        let centre = lod
                            .map(|l| {
                                l.vertices.iter().map(|v| v.position).sum::<Vec3>() / l.vertices.len().max(1) as f32
                            })
                            .unwrap_or_default();
                        let out = model_rot * centre.normalize_or_zero() * m.radius * 0.35;
                        ctx.renderer.draw_model_lod(&cam, &p, out, model_rot, Vec3::ZERO, Some(0));
                    }
                }
            } else if self.display != Display::Wire {
                ctx.renderer.begin_solid(&cam, sun);
                ctx.renderer.draw_model_lod(&cam, m, Vec3::ZERO, model_rot, Vec3::ZERO, Some(lod_index));
            }
            gl_use_default_material();
            for a in &m.model.attachments {
                ctx.renderer.draw_axes(&cam, model_rot * a.position, model_rot, m.radius * 0.15);
            }
            if self.display != Display::Solid {
                draw_wireframe(&cam, lod, model_rot, self.display == Display::SolidWire);
            }
            set_default_camera();
            draw_face_marks(&cam, lod, model_rot, self.face, self.labels, self.display != Display::Solid);
            let lod_info: Vec<String> = m
                .model
                .lods
                .iter()
                .map(|l| format!("{} tris < {:.0} m", l.triangle_count(), l.switch_distance.min(1e7)))
                .collect();
            let lines = [
                entry.label.clone(),
                format!("radius {:.1} m   attachments {}", m.radius, m.model.attachments.len()),
                format!("LODs: {}", lod_info.join("  |  ")),
                format!(
                    "showing: {} (LOD {lod_index}, {} faces)   {}   overlays {}   distance {:.0} m",
                    self.forced_lod.map(|_| "forced LOD").unwrap_or("auto LOD"),
                    lod.faces.len(),
                    self.display.label(),
                    if ctx.renderer.show_overlays { "shown" } else { "hidden" },
                    self.orbit.distance
                ),
                match &set {
                    Some(set) if looks.len() > 1 => {
                        format!(
                            "look {}/{}: {}   (D: next look)",
                            self.look % looks.len() + 1,
                            looks.len(),
                            look_label(set, &look)
                        )
                    }
                    _ => self.face.map(|f| face_info(lod, f)).unwrap_or_default(),
                },
            ];
            for (i, l) in lines.iter().enumerate() {
                draw_text(l, 20.0, 34.0 + i as f32 * 24.0, if i == 0 { 28.0 } else { 20.0 }, WHITE);
            }
        } else {
            set_default_camera();
            draw_text(&format!("{}: could not load", entry.label), 20.0, 40.0, 26.0, ORANGE);
            if let Some(e) = ctx.renderer.models.errors.last() {
                draw_text(e, 20.0, 70.0, 18.0, ORANGE);
            }
        }
        draw_text(
            &format!(
                "{}/{}   PgUp/PgDn (B/N) model   arrows/drag orbit   +/- wheel zoom   L LOD   Space spin   W wireframe   [ ] face   I labels   O overlays   D look   Esc back",
                self.selected + 1,
                n
            ),
            20.0,
            screen_height() - 20.0,
            18.0,
            GRAY,
        );
        Next::Stay
    }
}

/// Draws every face outline as 3D lines (call with the 3D camera set). With `on_solid`
/// the lines sit on a drawn model, so each point is pulled 0.3% towards the camera to
/// keep the line in front of its own face in the depth test.
fn draw_wireframe(cam: &SpaceCamera, lod: &ModelLod, rot: Quat, on_solid: bool) {
    set_camera(cam);
    let pull = if on_solid { 0.997 } else { 1.0 };
    let colour = Color::new(0.4, 1.0, 0.6, 0.9);
    for f in &lod.faces {
        let corners = lod.face_vertices(f);
        for (k, v) in corners.iter().enumerate() {
            let w = corners[(k + 1) % corners.len()];
            // Camera-relative coordinates, as everywhere in the renderer.
            let a = (rot * v.position - cam.position) * pull;
            let b = (rot * w.position - cam.position) * pull;
            draw_line_3d(a, b, colour);
        }
    }
}

/// Vertex dots (with `dots`), the selected face's outline and number, and (optionally) every
/// front-facing face's number — in 2D, on top of the scene.
fn draw_face_marks(cam: &SpaceCamera, lod: &ModelLod, rot: Quat, selected: Option<usize>, labels: bool, dots: bool) {
    let screen = vec2(screen_width(), screen_height());
    let facing = |corners: &[crate::model::ModelVertex]| {
        face_plane(corners).is_some_and(|(n, _)| (rot * n).dot(cam.position - rot * corners[0].position) > 0.0)
    };
    for (i, f) in lod.faces.iter().enumerate() {
        let corners = lod.face_vertices(f);
        let points: Vec<Vec2> = corners.iter().filter_map(|v| cam.project(rot * v.position, screen)).collect();
        if points.len() != corners.len() {
            continue; // partly behind the camera
        }
        let front = facing(corners);
        if Some(i) == selected {
            for (k, p) in points.iter().enumerate() {
                let q = points[(k + 1) % points.len()];
                draw_line(p.x, p.y, q.x, q.y, 3.0, YELLOW);
            }
        }
        if Some(i) == selected || (labels && front) {
            let c = points.iter().copied().sum::<Vec2>() / points.len() as f32;
            let colour = if Some(i) == selected { YELLOW } else { WHITE };
            draw_text(&i.to_string(), c.x - 6.0, c.y + 5.0, 18.0, colour);
        }
        if front && dots {
            for p in &points {
                draw_rectangle(p.x - 2.0, p.y - 2.0, 4.0, 4.0, Color::new(1.0, 0.9, 0.3, 0.9));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orbit_looks_at_the_origin_from_its_distance() {
        let o = Orbit { yaw: 0.0, pitch: 0.0, distance: 10.0 };
        let (p, r) = o.pose();
        assert!((p - Vec3::new(0.0, 0.0, 10.0)).length() < 1e-5);
        assert!(((r * Vec3::NEG_Z) - Vec3::NEG_Z).length() < 1e-5, "looking towards the origin");
        let o = Orbit { yaw: 0.0, pitch: 0.5, distance: 10.0 };
        let (p, r) = o.pose();
        assert!(p.y > 0.0, "positive pitch is above");
        assert!(((r * Vec3::NEG_Z) - (-p.normalize())).length() < 1e-5, "still aimed at the origin");
    }

    #[test]
    fn entries_are_found_by_ship_id_or_mesh_name() {
        let e =
            |label: &str| ViewerEntry { label: label.into(), reference: String::new(), pack: 0, scale: 1.0, set: None };
        let list = [e("Dart (dart)"), e("mesh \"XBASE7\""), e("mesh \"A-FRT3\"")];
        assert_eq!(find_entry(&list, "dart"), Some(0));
        assert_eq!(find_entry(&list, "xbase7"), Some(1));
        assert_eq!(find_entry(&list, "\"A-FRT3\""), Some(2));
        assert_eq!(find_entry(&list, "FRT3"), None, "whole names only");
    }

    #[test]
    fn display_modes_cycle() {
        assert_eq!(Display::Solid.next(), Display::SolidWire);
        assert_eq!(Display::SolidWire.next(), Display::Wire);
        assert_eq!(Display::Wire.next(), Display::Solid);
    }

    #[test]
    fn near_plane_moves_out_to_the_model() {
        assert_eq!(near_plane(1700.0, 600.0), 980.0, "just in front of a far station");
        assert_eq!(near_plane(5.0, 4.9), 0.05, "never below 5 cm");
        assert_eq!(near_plane(1000.0, 900.0), 10.0, "at least 1% of the distance");
    }

    #[test]
    fn face_info_describes_a_face() {
        let v = |x: f32, y: f32| [x, y, 0.0, 0.0, 0.0, 1.0, x, y];
        let data = crate::formats::xmsh::build_test_xmsh_with_flags(
            &[v(0.0, 0.0), v(1.0, 0.0), v(1.0, 1.0)],
            &[(0, 1), (1, 2), (2, 0)],
            &["hull"],
            &[(0, 0x2000_011d, &[1, 2, 3])],
        );
        let m = crate::model::Model::from_xmsh(&crate::formats::xmsh::parse(&data).unwrap());
        let text = face_info(&m.lods[0], 0);
        assert!(text.starts_with("face 0 of 1   3 corners   texture hull   flags 0x2000011d   layer 0"), "{text}");
        assert!(text.contains("(1.00,1.00)"));
        assert_eq!(face_info(&m.lods[0], 5), "");
    }

    #[test]
    fn a_still_wheel_does_not_zoom() {
        assert_eq!(wheel_steps(0.0), 0.0);
        assert_eq!(wheel_steps(3.5), 1.0);
        assert_eq!(wheel_steps(-0.1), -1.0);
    }

    #[test]
    fn orbit_input_clamps() {
        let mut o = Orbit { yaw: 0.0, pitch: 0.0, distance: 10.0 };
        o.update(1.0, 100.0, 1.0, 1.0);
        assert_eq!(o.pitch, 1.5);
        assert_eq!(o.distance, 5.0, "zoom limited per step");
        o.update(0.0, 0.0, -10.0, 1.0);
        assert_eq!(o.distance, 7.5);
        assert_eq!(o.yaw, 1.0);
    }

    #[test]
    fn looks_cover_damage_states_and_debris() {
        use crate::content::looks::{DamageLevel, DebrisId, ModelSet};
        let set = ModelSet {
            id: "s".into(),
            intact: "m".into(),
            pack: 0,
            scale: 1.0,
            textures: vec![],
            damage: vec![
                DamageLevel { below: 0.66, swaps: vec![("hull".into(), "a.png".into())] },
                DamageLevel { below: 0.33, swaps: vec![("hull".into(), "b.png".into())] },
            ],
            states: vec![("building".into(), vec![("hull".into(), "c.png".into())])],
            state_models: Vec::new(),
            state_debris: Vec::new(),
            debris: Some(DebrisId(0)),
            destroyed_effect: None,
        };
        let looks = looks_of(&set);
        assert_eq!(looks, vec![Look::Intact, Look::Damage(0), Look::Damage(1), Look::State(0), Look::Debris]);
        assert_eq!(look_swaps(&set, &Look::Damage(0))[0].1, "a.png");
        assert_eq!(look_swaps(&set, &Look::Damage(1))[0].1, "b.png", "the worse level shows its own swap");
        assert_eq!(look_swaps(&set, &Look::State(0))[0].1, "c.png");
        assert!(look_swaps(&set, &Look::Intact).is_empty());
        assert_eq!(look_label(&set, &Look::Damage(1)), "damage: hull below 33%");
    }

    #[test]
    fn default_entries_list_every_class() {
        let packs = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("packs");
        let c = crate::content::Content::load(&crate::content::ContentConfig {
            pack_dirs: vec![packs],
            no_original: true,
            ..Default::default()
        })
        .unwrap();
        let e = entries(&c);
        assert_eq!(e.len(), c.rules.ships.len());
        assert!(e.iter().any(|x| x.label.starts_with("Comet")));
    }
}
