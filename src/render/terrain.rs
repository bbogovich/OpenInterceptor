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
//!
//! **Detail textures.** Vertex colours are smooth over a grid square tens of metres
//! wide, so up close the ground looks like painted plastic. A terrain may name a small
//! tiling `detail` texture: the mesh then gets a texture coordinate per vertex from its
//! position on the ground ([`detail_uv`]: one repeat per `detail_scale` metres), and its
//! triangles join a textured material group. No new shader is needed — the lit shader
//! already multiplies the vertex colour by the texture, and an untextured group samples
//! a white texture, which is why a terrain without `detail` looks exactly as before.
//! With a `slope_detail` too, steep triangles go to a second group using that texture.
//! The renderer gives detail textures *mipmaps* (smaller, pre-averaged copies) so the
//! repeats far away blend into their average colour instead of shimmering.
//!
//! **Scattered objects.** [`scatter_cells`] bakes every placed piece of one scatter
//! entry into a few big meshes, one per square *cell* of the field: the pieces' vertices
//! are transformed once, at load, so drawing hundreds of rocks costs one draw per visible
//! cell and no per-frame work. Cells farther than the entry's draw distance are skipped.
//! (macroquad has no GPU instancing; baking is the batching it allows.)

use crate::content::environment::{Heightfield, Placement, TerrainDef};
use crate::model::{MaterialGroup, Model, ModelLod, ModelVertex};
use glam::{Quat, Vec2, Vec3};

/// The detail texture coordinate of ground point `(x, z)` (field space, metres): one
/// repeat every `scale` metres. Both meshes (normal and mirrored) take it from the
/// *unmirrored* field position, so neighbouring tiles meet with matching coordinates
/// along their shared edge, just as their heights do.
///
/// Precision note: at the far edge of a 30 km field with a 40 m repeat the coordinate
/// reaches about 375. Desktop OpenGL interpolates it in 32-bit floats, which is plenty;
/// a GPU honouring `mediump` (16-bit) in the fragment shader would show blocky texels
/// far from the field's centre.
pub fn detail_uv(x: f32, z: f32, scale: f32) -> Vec2 {
    Vec2::new(x / scale, z / scale)
}

/// How steep the ground is at a normal, 0 (level) to 1 (steep): the same measure that
/// blends the vertex colour towards the slope colour, and that picks the slope texture.
fn steepness(normal: Vec3) -> f32 {
    ((1.0 - normal.y - 0.08) / 0.3).clamp(0.0, 1.0)
}

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

/// The terrain's mesh: one LOD; one untextured material group, or with detail textures
/// one group for `detail` and (if given) one for `slope_detail`. `mirror_x` builds the
/// field reflected in X (for the reflected tiles beyond it).
pub fn terrain_model(def: &TerrainDef, field: &Heightfield, mirror_x: bool) -> Model {
    let sx = if mirror_x { -1.0 } else { 1.0 };
    let n = field.n;
    let mut vertices = Vec::with_capacity(n * n);
    // Each vertex's steepness, kept to sort triangles between the two detail textures.
    let mut steep_at = Vec::with_capacity(n * n);
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
            let steep = steepness(normal);
            steep_at.push(steep);
            let col = mix(base, def.colors[2], steep);
            // A small, fixed variation per vertex (a hash of its grid position).
            let jitter = 0.92 + 0.16 * (((r * 7919 + c * 104_729) % 1000) as f32 / 1000.0);
            let to8 = |v: f32| (v * jitter * 255.0).clamp(0.0, 255.0) as u8;
            let uv = match def.detail {
                Some(_) => detail_uv(x, z, def.detail_scale),
                // Unused without a texture; kept as it always was.
                None => Vec2::new(c as f32 / (n - 1) as f32, r as f32 / (n - 1) as f32),
            };
            vertices.push(ModelVertex {
                position: Vec3::new(x * sx, h, z),
                normal,
                uv,
                color: [to8(col[0]), to8(col[1]), to8(col[2]), 255],
                layer: 0,
            });
        }
    }
    // Group 0: the ground (textured with `detail`, if any); group 1: steep triangles,
    // when there is a `slope_detail` to give them.
    let mut flat = Vec::with_capacity((n - 1) * (n - 1) * 6);
    let mut steep = Vec::new();
    for r in 0..n - 1 {
        for c in 0..n - 1 {
            let a = (r * n + c) as u32;
            let b = a + 1;
            let cc = a + n as u32;
            let d = cc + 1;
            // Mirroring reverses the winding; swap two corners to keep the front up.
            let tris = if mirror_x { [[a, b, cc], [b, d, cc]] } else { [[a, cc, b], [b, cc, d]] };
            for tri in tris {
                let s = tri.iter().map(|&i| steep_at[i as usize]).sum::<f32>() / 3.0;
                let to = if def.slope_detail.is_some() && s > 0.5 { &mut steep } else { &mut flat };
                to.extend(tri);
            }
        }
    }
    let mut groups = vec![MaterialGroup { texture: def.detail.clone(), indices: flat, overlay: false }];
    if !steep.is_empty() {
        groups.push(MaterialGroup { texture: def.slope_detail.clone(), indices: steep, overlay: false });
    }
    let half = field.size * 0.5;
    Model {
        name: format!("terrain:{}", def.id),
        radius: (half * half * 2.0 + def.height[1] * def.height[1]).sqrt(),
        bounds_min: Vec3::new(-half, def.height[0], -half),
        bounds_max: Vec3::new(half, def.height[1], half),
        lods: vec![ModelLod { switch_distance: f32::MAX, vertices, groups, faces: Vec::new() }],
        attachments: Vec::new(),
    }
}

/// Square cells per field side that scattered pieces are baked into (so at most
/// 16 × 16 = 256 meshes per entry; about 1.9 km across on a 30 km field). Smaller cells
/// cull more tightly but cost more draws and more culling tests.
pub const SCATTER_CELLS: usize = 16;

/// One baked cell of scattered pieces: a mesh in field space plus a sphere around it
/// (for culling by distance).
pub struct ScatterCell {
    pub centre: Vec3,
    pub radius: f32,
    pub model: Model,
}

/// Bakes `placements` of `piece` into one mesh per occupied cell of a `size`-metre field
/// ([`SCATTER_CELLS`] per side). Every placement's vertices are scaled, rotated and moved
/// to their place once, here; the result's material groups are the piece's own (one per
/// texture), each listing the triangles of every piece in the cell. `mirror_x` bakes the
/// X-mirrored copy for the reflected tiles, as [`terrain_model`] does.
///
/// Only the piece's most detailed LOD is used: at the distances scatter is drawn the
/// pieces are small, and one LOD keeps the bake simple.
pub fn scatter_cells(placements: &[Placement], piece: &Model, size: f32, mirror_x: bool) -> Vec<ScatterCell> {
    let Some(lod) = piece.lods.first() else { return Vec::new() };
    let sx = if mirror_x { -1.0 } else { 1.0 };
    let cell_of = |v: f32| (((v / size + 0.5) * SCATTER_CELLS as f32) as usize).min(SCATTER_CELLS - 1);
    // Sort the placements into their cells first: a list of indices per cell.
    let mut members: Vec<Vec<usize>> = vec![Vec::new(); SCATTER_CELLS * SCATTER_CELLS];
    for (i, p) in placements.iter().enumerate() {
        members[cell_of(p.position.z) * SCATTER_CELLS + cell_of(p.position.x)].push(i);
    }
    let mut cells = Vec::new();
    for list in members.iter().filter(|l| !l.is_empty()) {
        let mut vertices = Vec::with_capacity(list.len() * lod.vertices.len());
        let mut groups: Vec<MaterialGroup> = lod
            .groups
            .iter()
            .map(|g| MaterialGroup { texture: g.texture.clone(), indices: Vec::new(), overlay: g.overlay })
            .collect();
        let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        for &i in list {
            let p = placements[i];
            // Indices of this copy start after every vertex already in the cell.
            let base = vertices.len() as u32;
            for v in &lod.vertices {
                let at = p.position + p.rotation * (v.position * p.scale);
                let normal = p.rotation * v.normal;
                vertices.push(ModelVertex {
                    position: Vec3::new(at.x * sx, at.y, at.z),
                    normal: Vec3::new(normal.x * sx, normal.y, normal.z),
                    ..*v
                });
            }
            // The copy's bounding box, to size the cell's culling sphere.
            let r = piece.radius * p.scale;
            lo = lo.min(p.position - Vec3::splat(r));
            hi = hi.max(p.position + Vec3::splat(r));
            for (out, g) in groups.iter_mut().zip(&lod.groups) {
                for t in g.indices.chunks_exact(3) {
                    // A mirror image turns the winding round: swap two corners back.
                    let t = if mirror_x { [t[0], t[2], t[1]] } else { [t[0], t[1], t[2]] };
                    out.indices.extend(t.iter().map(|&k| base + k));
                }
            }
        }
        groups.retain(|g| !g.indices.is_empty());
        let (lo, hi) = (Vec3::new(lo.x * sx, lo.y, lo.z), Vec3::new(hi.x * sx, hi.y, hi.z));
        let (lo, hi) = (lo.min(hi), lo.max(hi)); // mirroring swaps the X bounds
        let centre = (lo + hi) * 0.5;
        cells.push(ScatterCell {
            centre,
            radius: (hi - lo).length() * 0.5,
            model: Model {
                name: format!("scatter:{}", piece.name),
                radius: (hi - lo).length() * 0.5 + centre.length(),
                bounds_min: lo,
                bounds_max: hi,
                lods: vec![ModelLod { switch_distance: f32::MAX, vertices, groups, faces: Vec::new() }],
                attachments: Vec::new(),
            },
        });
    }
    cells
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::environment::TerrainSource;

    /// A plain terrain definition of `size` metres and `n` points per side (no detail,
    /// no scatter); tests change what they need with `..def(…)`.
    fn def(size: f32, n: usize) -> TerrainDef {
        TerrainDef {
            id: "t".into(),
            source: TerrainSource::Generate { kind: "hills".into(), seed: 1 },
            size,
            height: [0.0, 100.0],
            colors: [[0.5; 3]; 3],
            resolution: n,
            pack: 0,
            field: None,
            detail: None,
            slope_detail: None,
            detail_scale: 50.0,
            scatter: Vec::new(),
        }
    }

    #[test]
    fn detail_coordinates_repeat_per_scale_and_meet_at_tile_edges() {
        assert_eq!(detail_uv(80.0, -20.0, 40.0), Vec2::new(2.0, -0.5));
        let field =
            Heightfield { n: 3, size: 200.0, heights: vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0] };
        let d = TerrainDef { detail: Some("d.png".into()), detail_scale: 25.0, ..def(200.0, 3) };
        let meshes = [terrain_model(&d, &field, false), terrain_model(&d, &field, true)];
        // Vertex 5 (column 2, row 1) lies at x = 100, z = 0: 4 repeats of 25 m along u.
        assert_eq!(meshes[0].lods[0].vertices[5].uv, Vec2::new(4.0, 0.0));
        assert_eq!(meshes[0].lods[0].groups[0].texture.as_deref(), Some("d.png"));
        // Wherever two neighbouring tiles share a corner, it has one texture coordinate:
        // no seam in the texture across the mirror-image tiles.
        let corners = |i: i32, j: i32| -> Vec<(Vec3, Vec2)> {
            let (mirror, at, rot) = tile_placement(&field, i, j);
            meshes[mirror as usize].lods[0].vertices.iter().map(|v| (at + rot * v.position, v.uv)).collect()
        };
        let mut shared = 0;
        for (a, b) in [((0, 0), (1, 0)), ((0, 0), (0, 1)), ((1, 0), (1, 1)), ((0, 1), (1, 1))] {
            for (p, uv) in corners(a.0, a.1) {
                for (q, uv2) in corners(b.0, b.1) {
                    if p.distance(q) < 1e-3 {
                        shared += 1;
                        assert!(uv.abs_diff_eq(uv2, 1e-5), "tiles {a:?}/{b:?} at {p:?}: {uv} vs {uv2}");
                    }
                }
            }
        }
        assert_eq!(shared, 12, "three corners along each of four shared edges");
    }

    #[test]
    fn steep_triangles_take_the_slope_texture() {
        // A sharp spike in the middle: its four sides are steep, the rest of a big flat
        // field is level.
        let mut heights = vec![0.0; 25];
        heights[12] = 300.0;
        let field = Heightfield { n: 5, size: 400.0, heights };
        let d = TerrainDef { detail: Some("d.png".into()), slope_detail: Some("s.png".into()), ..def(400.0, 5) };
        let lod = &terrain_model(&d, &field, false).lods[0];
        assert_eq!(lod.groups.len(), 2);
        assert_eq!(lod.groups[1].texture.as_deref(), Some("s.png"));
        let (flat, steep) = (lod.groups[0].indices.len() / 3, lod.groups[1].indices.len() / 3);
        assert_eq!(flat + steep, 32, "every triangle in exactly one group");
        assert!(steep > 0 && flat > steep, "{flat} flat, {steep} steep");
        // The steep group's triangles really are the tilted ones.
        for t in lod.groups[1].indices.chunks_exact(3) {
            let s: f32 = t.iter().map(|&i| steepness(lod.vertices[i as usize].normal)).sum::<f32>() / 3.0;
            assert!(s > 0.5);
        }
        // Without slope_detail all triangles stay in the one group.
        let one = TerrainDef { slope_detail: None, ..d };
        assert_eq!(terrain_model(&one, &field, false).lods[0].groups.len(), 1);
    }

    /// A one-triangle "model" pointing up +Y from its origin, 1 m tall.
    fn spike() -> Model {
        let v = |p: Vec3| ModelVertex { position: p, normal: Vec3::Z, uv: Vec2::ZERO, color: [255; 4], layer: 0 };
        Model {
            name: "spike".into(),
            radius: 1.0,
            bounds_min: Vec3::new(-0.5, 0.0, 0.0),
            bounds_max: Vec3::new(0.5, 1.0, 0.0),
            lods: vec![ModelLod {
                switch_distance: f32::MAX,
                vertices: vec![v(Vec3::new(-0.5, 0.0, 0.0)), v(Vec3::new(0.5, 0.0, 0.0)), v(Vec3::new(0.0, 1.0, 0.0))],
                groups: vec![MaterialGroup {
                    texture: Some("rock.png".into()),
                    indices: vec![0, 1, 2],
                    overlay: false,
                }],
                faces: Vec::new(),
            }],
            attachments: Vec::new(),
        }
    }

    #[test]
    fn scatter_is_baked_into_cells_in_place() {
        let at = |x: f32, z: f32, scale: f32| Placement {
            position: Vec3::new(x, 10.0, z),
            rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            scale,
        };
        // Two pieces in one corner cell, one in the opposite corner (field 800 m).
        let placed = [at(-390.0, -390.0, 2.0), at(-360.0, -380.0, 1.0), at(390.0, 390.0, 1.0)];
        let cells = scatter_cells(&placed, &spike(), 800.0, false);
        assert_eq!(cells.len(), 2, "only occupied cells are built");
        let first = &cells[0].model.lods[0];
        assert_eq!((first.vertices.len(), first.triangle_count()), (6, 2));
        assert_eq!(first.groups[0].texture.as_deref(), Some("rock.png"), "the piece's material is kept");
        assert_eq!(first.groups[0].indices, vec![0, 1, 2, 3, 4, 5], "each copy's indices follow its vertices");
        // The tip of the first, twice-size spike: 2 m straight up from where it stands.
        assert!(first.vertices[2].position.abs_diff_eq(Vec3::new(-390.0, 12.0, -390.0), 1e-4));
        // Rotated a quarter turn about Y, the spike's base runs along Z.
        assert!(first.vertices[1].position.abs_diff_eq(Vec3::new(-390.0, 10.0, -391.0), 1e-4));
        // Every vertex lies inside the cell's culling sphere.
        for c in &cells {
            for v in &c.model.lods[0].vertices {
                assert!(v.position.distance(c.centre) <= c.radius + 1e-3);
            }
        }
        // The mirrored bake: x negated, winding reversed (the face still faces the same way
        // relative to its mirrored shape), placed in the mirrored cell.
        let mirrored = scatter_cells(&placed, &spike(), 800.0, true);
        let m = &mirrored[0].model.lods[0];
        assert!(m.vertices[2].position.abs_diff_eq(Vec3::new(390.0, 12.0, -390.0), 1e-4));
        assert_eq!(m.groups[0].indices[..3], [0, 2, 1]);
        assert!(mirrored[0].centre.x > 0.0);
        assert!(scatter_cells(&[], &spike(), 800.0, false).is_empty());
    }

    #[test]
    fn tiles_match_the_folded_ground_heights() {
        // An asymmetric field, so a wrong reflection shows.
        let field =
            Heightfield { n: 3, size: 200.0, heights: vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0] };
        let def = TerrainDef { height: [0.0, 80.0], ..def(200.0, 3) };
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
            height: [0.0, 50.0],
            colors: [[0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            ..def(200.0, 3)
        };
        let m = terrain_model(&def, &field, false);
        let lod = &m.lods[0];
        assert_eq!(lod.vertices.len(), 9);
        assert_eq!(lod.triangle_count(), 8);
        // Without detail: one untextured group, as before detail textures existed.
        assert_eq!(lod.groups.len(), 1);
        assert_eq!(lod.groups[0].texture, None);
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
