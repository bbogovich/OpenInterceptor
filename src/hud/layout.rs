//! # Cockpit layout: instruments in perspective, and the layout editor's maths
//!
//! A 2D cockpit places each instrument in a rectangle of its design space. A **3D
//! cockpit** can instead *anchor* an instrument on its model: a rectangle lying on the
//! dashboard's surface ([`Anchor`]). The instrument is drawn into a texture and the
//! texture is mapped onto that rectangle, so it sits in its bezel in correct
//! perspective at any window shape or field of view.
//!
//! The layout editor (`F12` in the cockpit view) moves instruments:
//!
//! - **2D:** drag a rectangle to move it, drag its corner to resize ([`drag_rect`]).
//! - **3D:** click the dashboard: a ray from the eye through the cursor ([`cursor_ray`])
//!   is intersected with the cockpit model's triangles ([`pick`]); the hit point becomes
//!   the anchor's centre and the hit triangle's normal its facing.
//!
//! Everything here is pure maths on plain values, so it is unit-tested.

use crate::content::rules::Anchor;
use glam::{Vec2, Vec3};

/// The four corners of an anchored instrument — top-left, top-right, bottom-right,
/// bottom-left — in the camera's frame (metres from the eye: x right, y up, −z ahead).
/// `offset` is the cockpit model's offset from the eye.
///
/// The rectangle lies in the plane through `at` with normal `normal`; its up direction
/// is world up projected onto that plane, and its right direction completes the frame.
pub fn anchor_corners(a: &Anchor, offset: Vec3) -> [Vec3; 4] {
    let n = Vec3::from(a.normal).normalize_or(Vec3::Z);
    // Up: +Y with its component along the normal removed (Gram–Schmidt).
    let up = (Vec3::Y - n * Vec3::Y.dot(n)).normalize_or(Vec3::NEG_Z);
    let right = up.cross(n).normalize_or(Vec3::X);
    let c = Vec3::from(a.at) + offset;
    let (hw, hh) = (right * a.size[0] * 0.5, up * a.size[1] * 0.5);
    [c - hw + hh, c + hw + hh, c + hw - hh, c - hw - hh]
}

/// A point of the anchored rectangle at texture coordinates `(u, v)` (0..1, v down),
/// by bilinear interpolation of the corners.
pub fn anchor_point(corners: &[Vec3; 4], u: f32, v: f32) -> Vec3 {
    let top = corners[0].lerp(corners[1], u);
    let bottom = corners[3].lerp(corners[2], u);
    top.lerp(bottom, v)
}

/// The direction (camera frame) of the ray from the eye through screen pixel `cursor`.
pub fn cursor_ray(cursor: Vec2, screen: Vec2, fov_y: f32, aspect: f32) -> Vec3 {
    let ndc = Vec2::new(cursor.x / screen.x * 2.0 - 1.0, 1.0 - cursor.y / screen.y * 2.0);
    let t = (fov_y * 0.5).tan();
    Vec3::new(ndc.x * t * aspect, ndc.y * t, -1.0).normalize()
}

/// Ray–triangle intersection (Möller–Trumbore): the distance along `dir` from `origin`
/// to triangle `abc`, if the ray hits its front or back.
pub fn ray_triangle(origin: Vec3, dir: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Option<f32> {
    let (e1, e2) = (b - a, c - a);
    let p = dir.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1e-9 {
        return None; // parallel
    }
    let inv = 1.0 / det;
    let s = origin - a;
    let u = s.dot(p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(e1);
    let v = dir.dot(q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = e2.dot(q) * inv;
    (t > 1e-5).then_some(t)
}

/// The nearest point where a ray from the eye along `dir` hits the cockpit's triangles
/// (model space; `offset` places the model relative to the eye), with the surface normal
/// turned to face the eye. Both are in the model's own space, ready for an [`Anchor`].
pub fn pick(triangles: impl Iterator<Item = [Vec3; 3]>, offset: Vec3, dir: Vec3) -> Option<(Vec3, Vec3)> {
    let mut best: Option<(f32, [Vec3; 3])> = None;
    for tri in triangles {
        if let Some(t) = ray_triangle(Vec3::ZERO, dir, tri[0] + offset, tri[1] + offset, tri[2] + offset) {
            if best.is_none_or(|(bt, _)| t < bt) {
                best = Some((t, tri));
            }
        }
    }
    let (t, tri) = best?;
    let mut n = (tri[1] - tri[0]).cross(tri[2] - tri[0]).normalize_or(Vec3::Z);
    if n.dot(dir) > 0.0 {
        n = -n; // face the pilot
    }
    // Lift the anchor a few millimetres off the surface so it never fights its depth.
    Some((dir * t - offset + n * 0.004, n))
}

/// What a 2D drag changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grip {
    /// Inside the rectangle: move it.
    Move,
    /// Near the bottom-right corner: resize it.
    Resize,
}

/// Where a press at `p` grabs rectangle `r` (design space), if at all. The corner grip
/// is `handle` units wide.
pub fn grip(r: [f32; 4], p: Vec2, handle: f32) -> Option<Grip> {
    let inside = p.x >= r[0] && p.y >= r[1] && p.x <= r[0] + r[2] && p.y <= r[1] + r[3];
    let corner = (p.x - (r[0] + r[2])).abs() <= handle && (p.y - (r[1] + r[3])).abs() <= handle;
    if corner {
        Some(Grip::Resize)
    } else if inside {
        Some(Grip::Move)
    } else {
        None
    }
}

/// The rectangle after dragging by `delta` (design units) with `grip`; sizes stay at
/// least `min`.
pub fn drag_rect(r: [f32; 4], grip: Grip, delta: Vec2, min: f32) -> [f32; 4] {
    match grip {
        Grip::Move => [r[0] + delta.x, r[1] + delta.y, r[2], r[3]],
        Grip::Resize => [r[0], r[1], (r[2] + delta.x).max(min), (r[3] + delta.y).max(min)],
    }
}

/// Scales an anchor's size by `factor` (the editor's wheel and +/- keys), keeping its
/// shape.
pub fn scale_anchor(a: &Anchor, factor: f32) -> Anchor {
    Anchor { size: [(a.size[0] * factor).max(0.005), (a.size[1] * factor).max(0.005)], ..*a }
}

/// The YAML for a cockpit's layout, as the editor saves it into an addon:
/// `cockpits: {<id>: {instruments: …, anchors: …}}`.
pub fn layout_yaml(
    cockpit: &str,
    instruments: &std::collections::HashMap<String, [f32; 4]>,
    anchors: &std::collections::HashMap<String, Anchor>,
) -> String {
    let r = |x: f32| format!("{}", (x * 10_000.0).round() / 10_000.0);
    let mut names: Vec<&String> = instruments.keys().collect();
    names.sort();
    let mut out = format!("# yaml-language-server: $schema=../../../schemas/rules.schema.json\n# Cockpit layout saved by the OpenInterceptor layout editor (F12).\ncockpits:\n  {cockpit}:\n");
    if !names.is_empty() {
        out += "    instruments:\n";
        for n in names {
            let v = instruments[n];
            out += &format!("      {n}: [{}, {}, {}, {}]\n", v[0].round(), v[1].round(), v[2].round(), v[3].round());
        }
    }
    let mut names: Vec<&String> = anchors.keys().collect();
    names.sort();
    if !names.is_empty() {
        out += "    anchors:\n";
        for n in names {
            let a = anchors[n];
            out += &format!(
                "      {n}: {{at: [{}, {}, {}], normal: [{}, {}, {}], size: [{}, {}]}}\n",
                r(a.at[0]),
                r(a.at[1]),
                r(a.at[2]),
                r(a.normal[0]),
                r(a.normal[1]),
                r(a.normal[2]),
                r(a.size[0]),
                r(a.size[1])
            );
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn upright() -> Anchor {
        Anchor { at: [0.0, 0.0, -1.0], normal: [0.0, 0.0, 1.0], size: [0.4, 0.2] }
    }

    #[test]
    fn corners_lie_on_the_surface_with_world_up() {
        let c = anchor_corners(&upright(), Vec3::ZERO);
        assert!((c[0] - Vec3::new(-0.2, 0.1, -1.0)).length() < 1e-6, "top-left");
        assert!((c[2] - Vec3::new(0.2, -0.1, -1.0)).length() < 1e-6, "bottom-right");
        // A dashboard tilted back 20 degrees: corners stay in its plane, the top is
        // further away than the bottom.
        let tilt = Anchor { normal: [0.0, 20f32.to_radians().sin(), 20f32.to_radians().cos()], ..upright() };
        let c = anchor_corners(&tilt, Vec3::new(0.0, -0.5, 0.0));
        let n = Vec3::from(tilt.normal);
        for corner in c {
            assert!((corner - Vec3::new(0.0, -0.5, -1.0)).dot(n).abs() < 1e-5);
        }
        assert!(c[0].z < c[3].z, "the top leans away from the pilot");
        assert!((anchor_point(&c, 0.5, 0.5) - Vec3::new(0.0, -0.5, -1.0)).length() < 1e-5, "centre");
    }

    #[test]
    fn rays_pick_the_nearest_surface() {
        let dir = cursor_ray(Vec2::new(400.0, 300.0), Vec2::new(800.0, 600.0), 1.0, 4.0 / 3.0);
        assert!((dir - Vec3::NEG_Z).length() < 1e-6, "screen centre looks ahead");
        let right = cursor_ray(Vec2::new(800.0, 300.0), Vec2::new(800.0, 600.0), 90f32.to_radians(), 1.0);
        assert!((right.x - right.z.abs()).abs() < 1e-5, "the right edge is 45 degrees off at 90 fov, square screen");
        // Two quads ahead at 1 m and 2 m: the nearer wins, normal faces the eye.
        let quad = |z: f32| {
            let (a, b, c, d) =
                (Vec3::new(-1., -1., z), Vec3::new(1., -1., z), Vec3::new(1., 1., z), Vec3::new(-1., 1., z));
            vec![[a, b, c], [a, c, d]]
        };
        let tris: Vec<[Vec3; 3]> = quad(-2.0).into_iter().chain(quad(-1.0)).collect();
        let (p, n) = pick(tris.iter().copied(), Vec3::ZERO, Vec3::NEG_Z).unwrap();
        assert!((p.z - (-1.0 + 0.004)).abs() < 1e-5, "{p}");
        assert_eq!(n, Vec3::Z);
        // With the model offset, the hit is reported in model space.
        let (p, _) = pick(tris.iter().copied(), Vec3::new(0.0, 0.0, 0.5), Vec3::NEG_Z).unwrap();
        assert!((p.z - (-1.0 + 0.004)).abs() < 1e-5);
        assert!(pick(tris.into_iter(), Vec3::ZERO, Vec3::Z).is_none(), "nothing behind");
        assert!(ray_triangle(
            Vec3::ZERO,
            Vec3::X,
            Vec3::new(0., 0., -1.),
            Vec3::new(1., 0., -1.),
            Vec3::new(0., 1., -1.)
        )
        .is_none());
    }

    #[test]
    fn dragging_and_scaling() {
        let r = [10.0, 10.0, 100.0, 50.0];
        assert_eq!(grip(r, Vec2::new(50.0, 30.0), 6.0), Some(Grip::Move));
        assert_eq!(grip(r, Vec2::new(108.0, 58.0), 6.0), Some(Grip::Resize));
        assert_eq!(grip(r, Vec2::new(0.0, 0.0), 6.0), None);
        assert_eq!(drag_rect(r, Grip::Move, Vec2::new(5.0, -5.0), 8.0), [15.0, 5.0, 100.0, 50.0]);
        assert_eq!(drag_rect(r, Grip::Resize, Vec2::new(-200.0, 10.0), 8.0), [10.0, 10.0, 8.0, 60.0]);
        let s = scale_anchor(&upright(), 1.5);
        assert!((s.size[0] - 0.6).abs() < 1e-6 && (s.size[1] - 0.3).abs() < 1e-6);
    }

    #[test]
    fn saved_layouts_are_valid_rules() {
        let mut inst = std::collections::HashMap::new();
        inst.insert("mfd".to_string(), [1.0, 2.0, 3.0, 4.0]);
        let mut anch = std::collections::HashMap::new();
        anch.insert("mfd".to_string(), upright());
        let text = layout_yaml("guard", &inst, &anch);
        let doc = crate::formats::yaml::parse(&text).unwrap();
        let mut problems = Vec::new();
        crate::content::schema::validate(&doc, &crate::content::files::rules(), "", &mut problems);
        assert!(problems.is_empty(), "{problems:?}\n{text}");
        assert_eq!(
            doc.get("cockpits").unwrap().get("guard").unwrap().get("anchors").unwrap().as_map().unwrap().len(),
            1
        );
    }
}
