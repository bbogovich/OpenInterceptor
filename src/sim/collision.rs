//! # Collision geometry
//!
//! Small, pure geometric tests used by the simulation. Each one is a free function on
//! plain vectors, so it is trivial to unit-test and reuse.
//!
//! - Fighters collide as **spheres** — cheap, and accurate enough for small craft.
//! - Large objects (stations) collide against their **triangle mesh** (the cheapest
//!   LOD), because a bounding sphere around a station with long arms would be mostly
//!   empty space. A sphere test runs first as a *broad phase*: only segments that touch
//!   the bounding sphere pay for the per-triangle *narrow phase*.
//!
//! Projectiles move fast — a 20 km/s tracer covers 333 m per 60 Hz tick — so instead
//! of testing the projectile's position we test the **segment** it travelled this tick.
//! Testing points would let fast bullets tunnel straight through targets.

use glam::{Quat, Vec3};

/// A triangle mesh in model space, used for large objects.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TriMesh {
    pub triangles: Vec<[Vec3; 3]>,
}

/// The collision shape of a ship class, plus its attachment points (where station
/// modules mount).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Shape {
    /// Bounding radius, metres.
    pub radius: f32,
    /// Present for large objects; `None` means "use the sphere".
    pub mesh: Option<TriMesh>,
    /// Attachment points in model space: `(name if any, position, facing)`.
    pub mounts: Vec<(Option<String>, Vec3, Vec3)>,
}

impl Shape {
    /// A plain sphere.
    pub fn sphere(radius: f32) -> Shape {
        Shape { radius, ..Default::default() }
    }

    /// The shape of a loaded model: its radius, its cheapest LOD as a triangle mesh for
    /// static objects, and its attachment points.
    pub fn from_model(model: &crate::model::Model, is_static: bool) -> Shape {
        Shape {
            radius: model.radius,
            mesh: is_static.then(|| TriMesh { triangles: model.collision_lod().triangles().collect() }),
            mounts: model.attachments.iter().map(|a| (a.name.clone(), a.position, a.forward)).collect(),
        }
    }
}

/// First intersection of the segment `a → b` with a sphere, as a fraction `t` in 0..=1
/// along the segment. `None` if it misses.
///
/// Derivation: points on the segment are `a + t·d`. Setting `|a + t·d − c|² = r²` gives
/// a quadratic `t²(d·d) + 2t(d·m) + (m·m − r²) = 0` with `m = a − c`.
pub fn segment_sphere(a: Vec3, b: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let d = b - a;
    let m = a - center;
    let c = m.length_squared() - radius * radius;
    if c <= 0.0 {
        return Some(0.0); // the segment starts inside the sphere
    }
    let dd = d.length_squared();
    if dd == 0.0 {
        return None;
    }
    let bq = m.dot(d);
    let disc = bq * bq - dd * c;
    if bq > 0.0 || disc < 0.0 {
        return None; // moving away, or the line misses entirely
    }
    let t = (-bq - disc.sqrt()) / dd;
    (t <= 1.0).then_some(t.max(0.0))
}

/// Segment–triangle intersection (the Möller–Trumbore algorithm). Returns `t` in 0..=1.
/// Triangles are hit from either side.
pub fn segment_triangle(a: Vec3, b: Vec3, tri: &[Vec3; 3]) -> Option<f32> {
    const EPS: f32 = 1e-7;
    let d = b - a;
    let e1 = tri[1] - tri[0];
    let e2 = tri[2] - tri[0];
    let p = d.cross(e2);
    let det = e1.dot(p);
    if det.abs() < EPS {
        return None; // segment parallel to the triangle
    }
    let inv = 1.0 / det;
    let s = a - tri[0];
    let u = s.dot(p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(e1);
    let v = d.dot(q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = e2.dot(q) * inv;
    (0.0..=1.0).contains(&t).then_some(t)
}

/// Tests a world-space segment against a shape placed at `position`/`rotation`.
/// Returns the fraction along the segment of the first hit.
pub fn segment_shape(a: Vec3, b: Vec3, shape: &Shape, position: Vec3, rotation: Quat) -> Option<f32> {
    let broad = segment_sphere(a, b, position, shape.radius)?;
    let Some(mesh) = &shape.mesh else { return Some(broad) };
    // Move the segment into model space instead of moving every triangle into world
    // space: two transforms instead of thousands.
    let inv = rotation.inverse();
    let (la, lb) = (inv * (a - position), inv * (b - position));
    mesh.triangles.iter().filter_map(|t| segment_triangle(la, lb, t)).min_by(|x, y| x.total_cmp(y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_sphere_hits_misses_and_inside() {
        let c = Vec3::new(0.0, 0.0, -10.0);
        let t = segment_sphere(Vec3::ZERO, Vec3::new(0.0, 0.0, -20.0), c, 2.0).unwrap();
        assert!((t - 0.4).abs() < 1e-6, "enters at z = -8");
        assert_eq!(segment_sphere(Vec3::ZERO, Vec3::new(0.0, 0.0, -5.0), c, 2.0), None, "too short");
        assert_eq!(segment_sphere(Vec3::ZERO, Vec3::new(5.0, 0.0, 0.0), c, 2.0), None, "misses");
        assert_eq!(segment_sphere(c, c + Vec3::X, c, 2.0), Some(0.0), "starts inside");
        assert_eq!(segment_sphere(Vec3::X * 5.0, Vec3::X * 5.0, Vec3::ZERO, 1.0), None, "zero length");
        assert_eq!(segment_sphere(Vec3::Z * 5.0, Vec3::Z * 9.0, Vec3::ZERO, 1.0), None, "moving away");
    }

    #[test]
    fn segment_triangle_hits_inside_only() {
        let tri = [Vec3::new(-1.0, -1.0, -5.0), Vec3::new(1.0, -1.0, -5.0), Vec3::new(0.0, 1.0, -5.0)];
        let t = segment_triangle(Vec3::ZERO, Vec3::new(0.0, 0.0, -10.0), &tri).unwrap();
        assert!((t - 0.5).abs() < 1e-6);
        assert_eq!(segment_triangle(Vec3::new(5.0, 0.0, 0.0), Vec3::new(5.0, 0.0, -10.0), &tri), None);
        assert_eq!(segment_triangle(Vec3::ZERO, Vec3::new(0.0, 0.0, -4.0), &tri), None, "stops short");
        assert_eq!(segment_triangle(Vec3::ZERO, Vec3::X, &tri), None, "parallel");
        assert_eq!(segment_triangle(Vec3::new(0.9, 0.9, 0.0), Vec3::new(0.9, 0.9, -10.0), &tri), None, "outside v");
    }

    #[test]
    fn shapes_use_mesh_in_model_space() {
        let mesh = TriMesh {
            triangles: vec![[Vec3::new(-1.0, -1.0, 0.0), Vec3::new(1.0, -1.0, 0.0), Vec3::new(0.0, 1.0, 0.0)]],
        };
        let shape = Shape { radius: 10.0, mesh: Some(mesh), ..Default::default() };
        let pos = Vec3::new(0.0, 0.0, -100.0);
        // Rotated 90° about Y, the triangle now lies in the YZ plane: a segment along -Z
        // through its centre no longer hits it, but one along X does.
        let rot = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        assert_eq!(
            segment_shape(Vec3::new(0.0, 0.0, -80.0), Vec3::new(0.0, 0.0, -120.0), &shape, pos, Quat::IDENTITY)
                .map(|t| (t * 100.0).round()),
            Some(50.0)
        );
        assert!(segment_shape(Vec3::new(-5.0, 0.0, -100.0), Vec3::new(5.0, 0.0, -100.0), &shape, pos, rot).is_some());
        assert!(
            segment_shape(Vec3::new(0.0, 50.0, 0.0), Vec3::new(0.0, 50.0, -200.0), &shape, pos, rot).is_none(),
            "broad phase"
        );
        let sphere = Shape::sphere(2.0);
        assert!(segment_shape(Vec3::ZERO, Vec3::new(0.0, 0.0, -200.0), &sphere, pos, rot).is_some());
    }
}
