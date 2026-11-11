//! # Terrain as a mesh (Stage 3.5)
//!
//! Turns a terrain's [`Heightfield`] into an ordinary [`Model`], so the renderer draws it
//! like any other model: lit by the sun, faded by fog, depth-tested against the craft.
//! Each grid square becomes two triangles. Vertices are coloured from the terrain's
//! colours: low ground to high ground by height, blended towards the slope colour where
//! the ground is steep. A little per-vertex variation keeps large flats from looking
//! painted.
//!
//! Winding: seen from above (looking down −Y), each triangle runs counter-clockwise, so
//! its front faces up, as the renderer's back-face culling expects.
//!
//! **To the horizon.** The ground continues past the field as mirror-image tiles (see
//! [`Heightfield::fold`]). A tile reflected in X alone cannot be drawn by rotating the
//! normal mesh, so a second, X-mirrored mesh is built (`mirror_x`), with its triangles'
//! winding reversed so they still face up. A tile reflected in both X and Z is the normal
//! mesh turned 180° about Y, and one reflected in Z alone is the mirrored mesh turned
//! 180° — so two meshes draw every tile ([`tile_placement`]).

use crate::content::environment::{Heightfield, TerrainDef};
use crate::model::{MaterialGroup, Model, ModelLod, ModelVertex};
use glam::{Quat, Vec2, Vec3};

/// For tile `(i, j)` (tile `(0, 0)` is the field itself): whether to draw the X-mirrored
/// mesh, and the position and rotation to draw it at.
pub fn tile_placement(field: &Heightfield, i: i32, j: i32) -> (bool, Vec3, Quat) {
    let (odd_i, odd_j) = (i.rem_euclid(2) == 1, j.rem_euclid(2) == 1);
    let at = Vec3::new(i as f32 * field.size, 0.0, j as f32 * field.size);
    let half_turn = Quat::from_rotation_y(std::f32::consts::PI);
    match (odd_i, odd_j) {
        (false, false) => (false, at, Quat::IDENTITY),
        (true, false) => (true, at, Quat::IDENTITY),
        (false, true) => (true, at, half_turn),
        (true, true) => (false, at, half_turn),
    }
}

/// The terrain's mesh: one LOD, one untextured material group. `mirror_x` builds the
/// field reflected in X (for the reflected tiles beyond it).
pub fn terrain_model(def: &TerrainDef, field: &Heightfield, mirror_x: bool) -> Model {
    let sx = if mirror_x { -1.0 } else { 1.0 };
    let n = field.n;
    let mut vertices = Vec::with_capacity(n * n);
    let mix = |a: [f32; 3], b: [f32; 3], t: f32| {
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
    };
    let span = (def.height[1] - def.height[0]).max(1.0);
    for r in 0..n {
        for c in 0..n {
            let (x, z) = (field.coord(c), field.coord(r));
            let h = field.heights[r * n + c];
            let n0 = field.normal_at(x, z);
            let normal = Vec3::new(n0.x * sx, n0.y, n0.z);
            let t = ((h - def.height[0]) / span).clamp(0.0, 1.0);
            let base = mix(def.colors[0], def.colors[1], t);
            // Steepness: 0 on the flat, rising as the normal tips away from up.
            let steep = ((1.0 - normal.y - 0.08) / 0.3).clamp(0.0, 1.0);
            let col = mix(base, def.colors[2], steep);
            // A small, fixed variation per vertex (a hash of its grid position).
            let jitter = 0.92 + 0.16 * (((r * 7919 + c * 104_729) % 1000) as f32 / 1000.0);
            let to8 = |v: f32| (v * jitter * 255.0).clamp(0.0, 255.0) as u8;
            vertices.push(ModelVertex {
                position: Vec3::new(x * sx, h, z),
                normal,
                uv: Vec2::new(c as f32 / (n - 1) as f32, r as f32 / (n - 1) as f32),
                color: [to8(col[0]), to8(col[1]), to8(col[2]), 255],
                layer: 0,
            });
        }
    }
    let mut indices = Vec::with_capacity((n - 1) * (n - 1) * 6);
    for r in 0..n - 1 {
        for c in 0..n - 1 {
            let a = (r * n + c) as u32;
            let b = a + 1;
            let cc = a + n as u32;
            let d = cc + 1;
            // Mirroring reverses the winding; swap two corners to keep the front up.
            if mirror_x {
                indices.extend([a, b, cc, b, d, cc]);
            } else {
                indices.extend([a, cc, b, b, cc, d]);
            }
        }
    }
    let half = field.size * 0.5;
    Model {
        name: format!("terrain:{}", def.id),
        radius: (half * half * 2.0 + def.height[1] * def.height[1]).sqrt(),
        bounds_min: Vec3::new(-half, def.height[0], -half),
        bounds_max: Vec3::new(half, def.height[1], half),
        lods: vec![ModelLod {
            switch_distance: f32::MAX,
            vertices,
            groups: vec![MaterialGroup { texture: None, indices, overlay: false }],
            faces: Vec::new(),
        }],
        attachments: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::environment::TerrainSource;

    #[test]
    fn tiles_match_the_folded_ground_heights() {
        // An asymmetric field, so a wrong reflection shows.
        let field =
            Heightfield { n: 3, size: 200.0, heights: vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0] };
        let def = TerrainDef {
            id: "t".into(),
            source: TerrainSource::Generate { kind: "hills".into(), seed: 1 },
            size: 200.0,
            height: [0.0, 80.0],
            colors: [[0.0; 3]; 3],
            resolution: 3,
            pack: 0,
            field: None,
        };
        let meshes = [terrain_model(&def, &field, false), terrain_model(&def, &field, true)];
        for (i, j) in [(1, 0), (0, 1), (1, 1), (-1, -2), (2, -1)] {
            let (mirror, at, rot) = tile_placement(&field, i, j);
            for v in &meshes[mirror as usize].lods[0].vertices {
                let p = at + rot * v.position;
                let h = field.height_at(p.x, p.z);
                assert!((p.y - h).abs() < 1e-3, "tile ({i}, {j}) at {p:?}: mesh {} ground {h}", p.y);
            }
        }
    }

    #[test]
    fn terrain_meshes_face_up_and_colour_by_height() {
        let field = Heightfield { n: 3, size: 200.0, heights: vec![0.0, 0.0, 0.0, 0.0, 50.0, 0.0, 0.0, 0.0, 0.0] };
        let def = TerrainDef {
            id: "t".into(),
            source: TerrainSource::Generate { kind: "hills".into(), seed: 1 },
            size: 200.0,
            height: [0.0, 50.0],
            colors: [[0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            resolution: 3,
            pack: 0,
            field: None,
        };
        let m = terrain_model(&def, &field, false);
        let lod = &m.lods[0];
        assert_eq!(lod.vertices.len(), 9);
        assert_eq!(lod.triangle_count(), 8);
        // Every triangle's geometric normal points up (front faces upward).
        for t in lod.triangles() {
            assert!((t[1] - t[0]).cross(t[2] - t[0]).y > 0.0, "{t:?}");
        }
        // The mirrored mesh faces up too.
        for t in terrain_model(&def, &field, true).lods[0].triangles() {
            assert!((t[1] - t[0]).cross(t[2] - t[0]).y > 0.0, "{t:?}");
        }
        // The peak is high-ground red (with slope green mixed in); a corner is low blue.
        let peak = lod.vertices[4].color;
        let corner = lod.vertices[0].color;
        assert!(peak[0] > peak[2] && corner[2] > corner[0], "{peak:?} {corner:?}");
    }
}
