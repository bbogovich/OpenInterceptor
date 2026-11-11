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

use crate::content::environment::{Heightfield, TerrainDef};
use crate::model::{MaterialGroup, Model, ModelLod, ModelVertex};
use glam::{Vec2, Vec3};

/// The terrain's mesh: one LOD, one untextured material group.
pub fn terrain_model(def: &TerrainDef, field: &Heightfield) -> Model {
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
            let normal = field.normal_at(x, z);
            let t = ((h - def.height[0]) / span).clamp(0.0, 1.0);
            let base = mix(def.colors[0], def.colors[1], t);
            // Steepness: 0 on the flat, rising as the normal tips away from up.
            let steep = ((1.0 - normal.y - 0.08) / 0.3).clamp(0.0, 1.0);
            let col = mix(base, def.colors[2], steep);
            // A small, fixed variation per vertex (a hash of its grid position).
            let jitter = 0.92 + 0.16 * (((r * 7919 + c * 104_729) % 1000) as f32 / 1000.0);
            let to8 = |v: f32| (v * jitter * 255.0).clamp(0.0, 255.0) as u8;
            vertices.push(ModelVertex {
                position: Vec3::new(x, h, z),
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
            indices.extend([a, cc, b, b, cc, d]);
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
        let m = terrain_model(&def, &field);
        let lod = &m.lods[0];
        assert_eq!(lod.vertices.len(), 9);
        assert_eq!(lod.triangle_count(), 8);
        // Every triangle's geometric normal points up (front faces upward).
        for t in lod.triangles() {
            assert!((t[1] - t[0]).cross(t[2] - t[0]).y > 0.0, "{t:?}");
        }
        // The peak is high-ground red (with slope green mixed in); a corner is low blue.
        let peak = lod.vertices[4].color;
        let corner = lod.vertices[0].color;
        assert!(peak[0] > peak[2] && corner[2] > corner[0], "{peak:?} {corner:?}");
    }
}
