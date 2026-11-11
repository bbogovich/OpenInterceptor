//! # Engine models (format-neutral)
//!
//! Whatever a model was loaded from — an original `XMSH` resource or a modder's `.obj`
//! file — the rest of the engine sees one type: [`Model`]. It is already in the form the
//! renderer and the simulation want:
//!
//! - **Engine coordinates:** right-handed, +Y up, **−Z forward**, metres. The original
//!   meshes use Direct3D's left-handed convention with +Z forward, so [`Model::from_xmsh`]
//!   negates Z (and reverses triangle winding so faces still face outwards).
//! - **Triangles only**, grouped by texture: one [`MaterialGroup`] per texture, so the
//!   renderer issues one draw per group instead of one per polygon.
//! - **Levels of detail** ordered nearest-first, each with a switch distance.
//! - **Attachments**: points such as engine exhausts, from the original manipulators.
//! - **One-sided faces.** Every face is visible only from its front (the side its
//!   counter-clockwise winding faces), exactly as in the original game: the renderer
//!   culls back faces. A surface meant to be seen from both sides — a fin, a shield
//!   ring — is modelled as two faces back to back, which the original data already does.
//! - **Overlay layers.** A face lying exactly on top of another (a decal) would
//!   "z-fight" with it: the depth buffer cannot tell which is in front, so the two
//!   flicker. [`assign_overlay_layers`] gives each such face a layer number, and the
//!   renderer pulls higher layers very slightly towards the camera.
//!
//! Converting once at load time, instead of every frame, is a key performance pattern:
//! do expensive work when data enters the engine, and keep the per-frame path simple.
//!
//! This module is pure data (no GPU, no window), so it is fully unit-testable.

use crate::formats::obj::ObjMesh;
use crate::formats::xmsh::{XmshMesh, POLY_OVERLAY, POLY_TRANSLUCENT};
use glam::{Vec2, Vec3};

/// One vertex as the renderer consumes it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    /// Vertex colour (RGBA). White for textured faces; the flat colour otherwise.
    pub color: [u8; 4],
    /// Overlay layer of the face this corner belongs to: 0 for an ordinary surface,
    /// 1 for a decal on it, 2 for a decal on the decal… See [`assign_overlay_layers`].
    pub layer: u8,
}

/// Triangles that share one texture (or no texture).
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialGroup {
    /// Texture name to look up in the pack's file system; `None` = untextured.
    pub texture: Option<String>,
    /// Vertex indices, three per triangle, into [`ModelLod::vertices`].
    pub indices: Vec<u32>,
    /// True if the group holds faces the original data flags as overlays
    /// ([`POLY_OVERLAY`](crate::formats::xmsh::POLY_OVERLAY)). Kept apart so a tool can
    /// show the model with and without them.
    pub overlay: bool,
}

/// One original face (polygon) of a LOD, before triangulation. Its corners are the
/// consecutive vertices `first..first + count` of [`ModelLod::vertices`] — every face
/// owns its corners, so a range is enough (no separate index list). Used by tools such
/// as the model viewer's wireframe, and by [`assign_overlay_layers`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Face {
    pub first: u32,
    pub count: u16,
    /// Index into [`ModelLod::groups`].
    pub group: u16,
    /// The original polygon flags (0 for OBJ models).
    pub flags: u32,
}

/// One level of detail.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelLod {
    /// Use this LOD while the camera is nearer than this many metres.
    pub switch_distance: f32,
    pub vertices: Vec<ModelVertex>,
    pub groups: Vec<MaterialGroup>,
    /// The faces, in file order.
    pub faces: Vec<Face>,
}

impl ModelLod {
    /// The corner vertices of one face.
    pub fn face_vertices(&self, f: &Face) -> &[ModelVertex] {
        &self.vertices[f.first as usize..f.first as usize + f.count as usize]
    }

    /// Drops empty material groups and renumbers [`Face::group`] to match.
    fn drop_empty_groups(&mut self) {
        let mut new_index = vec![u16::MAX; self.groups.len()];
        let mut next = 0u16;
        for (i, g) in self.groups.iter().enumerate() {
            if !g.indices.is_empty() {
                new_index[i] = next;
                next += 1;
            }
        }
        self.groups.retain(|g| !g.indices.is_empty());
        for f in &mut self.faces {
            f.group = new_index[f.group as usize];
        }
    }

    /// Total triangle count over all groups.
    pub fn triangle_count(&self) -> usize {
        self.groups.iter().map(|g| g.indices.len() / 3).sum()
    }

    /// Iterates over every triangle as three positions. Used for collision and picking.
    pub fn triangles(&self) -> impl Iterator<Item = [Vec3; 3]> + '_ {
        self.groups.iter().flat_map(move |g| {
            g.indices.chunks_exact(3).map(move |t| {
                [
                    self.vertices[t[0] as usize].position,
                    self.vertices[t[1] as usize].position,
                    self.vertices[t[2] as usize].position,
                ]
            })
        })
    }
}

/// A named point on the model with a facing direction (model space).
#[derive(Debug, Clone, PartialEq)]
pub struct Attachment {
    /// The slot's name (OBJ `slot_<name>` groups); original manipulators are unnamed and
    /// addressed by index.
    pub name: Option<String>,
    pub position: Vec3,
    pub forward: Vec3,
}

/// A loaded model, ready for rendering and collision.
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub name: String,
    /// Radius of a sphere around the origin enclosing the model.
    pub radius: f32,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
    /// Nearest (most detailed) first. Never empty.
    pub lods: Vec<ModelLod>,
    pub attachments: Vec<Attachment>,
}

/// Converts a Direct3D (left-handed, +Z forward) point to engine space.
fn from_d3d(v: [f32; 3]) -> Vec3 {
    Vec3::new(v[0], v[1], -v[2])
}

impl Model {
    /// Picks the LOD to draw at `distance` metres from the camera.
    pub fn lod_for_distance(&self, distance: f32) -> &ModelLod {
        self.lods
            .iter()
            .find(|l| distance < l.switch_distance)
            .unwrap_or_else(|| self.lods.last().expect("a Model always has at least one LOD"))
    }

    /// The cheapest LOD — used as the collision shape.
    pub fn collision_lod(&self) -> &ModelLod {
        self.lods.last().expect("a Model always has at least one LOD")
    }

    /// Converts an original mesh.
    pub fn from_xmsh(mesh: &XmshMesh) -> Model {
        let mut lods = Vec::with_capacity(mesh.lods.len());
        for lod in &mesh.lods {
            let mut vertices = Vec::new();
            let mut faces = Vec::with_capacity(lod.polygons.len());
            // One group per texture plus one (index = textures.len()) for untextured
            // faces — and that whole set twice: first for ordinary faces, then for
            // flagged overlays.
            let plain: Vec<MaterialGroup> = lod
                .textures
                .iter()
                .map(|t| Some(t.clone()))
                .chain(std::iter::once(None))
                .map(|texture| MaterialGroup { texture, indices: Vec::new(), overlay: false })
                .collect();
            let per_set = plain.len();
            let mut groups: Vec<MaterialGroup> = plain
                .iter()
                .cloned()
                .chain(plain.iter().map(|g| MaterialGroup { overlay: true, ..g.clone() }))
                .collect();
            for poly in &lod.polygons {
                let color = match poly.texture {
                    Some(_) => [255; 4],
                    None => {
                        let c = poly.color;
                        [(c >> 16) as u8, (c >> 8) as u8, c as u8, 255]
                    }
                };
                // Each polygon corner becomes its own vertex: corners of different
                // polygons may need different colours, and the counts are small.
                let base = vertices.len() as u32;
                for &vi in lod.polygon_indices(poly) {
                    let v = lod.vertices[vi as usize];
                    vertices.push(ModelVertex {
                        position: from_d3d(v.position),
                        normal: from_d3d(v.normal),
                        uv: Vec2::new(v.uv[0], v.uv[1]),
                        color,
                        layer: 0,
                    });
                }
                let overlay = poly.flags & POLY_OVERLAY != 0;
                let group = poly.texture.map(|t| t as usize).unwrap_or(lod.textures.len()) + overlay as usize * per_set;
                faces.push(Face { first: base, count: poly.count, group: group as u16, flags: poly.flags });
                // Fan triangulation (the BSP polygons are convex). Swapping the last two
                // indices reverses the winding to match the mirrored coordinates.
                for k in 1..poly.count as u32 - 1 {
                    groups[group].indices.extend_from_slice(&[base, base + k + 1, base + k]);
                }
            }
            let mut out = ModelLod { switch_distance: lod.switch_distance, vertices, groups, faces };
            out.drop_empty_groups();
            // Which face wins where faces overlap: flagged overlays over translucent
            // faces over ordinary ones; among equals, the later one in the file (the
            // original painted in that order, so later faces covered earlier ones).
            let priority = |f: &Face| {
                if f.flags & POLY_OVERLAY != 0 {
                    2
                } else if f.flags & POLY_TRANSLUCENT != 0 {
                    1
                } else {
                    0
                }
            };
            assign_overlay_layers(&mut out, mesh.radius, priority);
            lods.push(out);
        }
        // Manipulators: the first three floats are the position, then a 3x3 rotation
        // whose third row is the facing axis (layout derived from the data).
        let attachments = mesh.lods[0]
            .manipulators
            .iter()
            .map(|m| Attachment {
                name: None,
                position: from_d3d([m.matrix[0], m.matrix[1], m.matrix[2]]),
                forward: from_d3d([m.matrix[9], m.matrix[10], m.matrix[11]]).normalize_or_zero(),
            })
            .collect();
        let b = mesh.bbox;
        // Mirroring Z swaps which corner is the minimum.
        let (min, max) = (Vec3::new(b[0], b[1], -b[5]), Vec3::new(b[3], b[4], -b[2]));
        Model {
            name: mesh.name.clone(),
            radius: mesh.radius.max(min.length()).max(max.length()),
            bounds_min: min,
            bounds_max: max,
            lods,
            attachments,
        }
    }

    /// Converts a mod's OBJ mesh. OBJ is already right-handed; by the modding convention
    /// (documented on the site) models face −Z with +Y up, like the engine. `scale`
    /// converts the file's units to metres. `materials` maps each OBJ material name to a
    /// texture file and colour (from the `.mtl` file).
    pub fn from_obj(name: &str, mesh: &ObjMesh, scale: f32, materials: &[crate::formats::obj::ObjMaterial]) -> Model {
        let mut vertices = Vec::with_capacity(mesh.corners.len());
        let mut groups: Vec<MaterialGroup> = mesh
            .materials
            .iter()
            .map(|m| MaterialGroup {
                texture: materials.iter().find(|x| &x.name == m).and_then(|x| x.texture.clone()),
                indices: Vec::new(),
                overlay: false,
            })
            .chain(std::iter::once(MaterialGroup { texture: None, indices: Vec::new(), overlay: false }))
            .collect();
        let mut faces = Vec::with_capacity(mesh.faces.len());
        for face in &mesh.faces {
            let corners = mesh.face_corners(face);
            let p: Vec<Vec3> =
                corners.iter().map(|c| Vec3::from(mesh.positions[c.position as usize]) * scale).collect();
            // A face normal for corners that have none.
            let face_normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
            let colour = face
                .material
                .and_then(|m| materials.iter().find(|x| x.name == mesh.materials[m as usize]))
                .map(|x| x.color)
                .unwrap_or([1.0; 3]);
            let color = [(colour[0] * 255.0) as u8, (colour[1] * 255.0) as u8, (colour[2] * 255.0) as u8, 255];
            let base = vertices.len() as u32;
            for (c, pos) in corners.iter().zip(&p) {
                let uv = c.uv.map(|i| mesh.uvs[i as usize]).unwrap_or([0.0, 0.0]);
                vertices.push(ModelVertex {
                    position: *pos,
                    normal: c
                        .normal
                        .map(|i| Vec3::from(mesh.normals[i as usize]).normalize_or_zero())
                        .unwrap_or(face_normal),
                    // OBJ's v axis points up the image; textures are stored top row first.
                    uv: Vec2::new(uv[0], 1.0 - uv[1]),
                    color,
                    layer: 0,
                });
            }
            let group = face.material.map(|m| m as usize).unwrap_or(mesh.materials.len());
            faces.push(Face { first: base, count: face.count, group: group as u16, flags: 0 });
            for k in 1..face.count as u32 - 1 {
                groups[group].indices.extend_from_slice(&[base, base + k, base + k + 1]);
            }
        }
        let mut lod = ModelLod { switch_distance: f32::MAX, vertices, groups, faces };
        lod.drop_empty_groups();
        let vertices = &lod.vertices;
        let (mut min, mut max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        for v in vertices {
            min = min.min(v.position);
            max = max.max(v.position);
        }
        if vertices.is_empty() {
            (min, max) = (Vec3::ZERO, Vec3::ZERO);
        }
        let radius = vertices.iter().map(|v| v.position.length()).fold(0.0, f32::max);
        // OBJ has no overlay flags: where faces overlap, the later one in the file wins.
        assign_overlay_layers(&mut lod, radius, |_| 0);
        Model {
            name: name.to_string(),
            radius,
            bounds_min: min,
            bounds_max: max,
            lods: vec![lod],
            attachments: mesh
                .slots
                .iter()
                .map(|(name, c, n)| Attachment {
                    name: Some(name.clone()),
                    position: Vec3::from(*c) * scale,
                    forward: Vec3::from(*n).normalize_or(Vec3::NEG_Z),
                })
                .collect(),
        }
    }
}

/// The plane a face lies in, as a unit normal `n` and offset `d` (`n · p = d` for every
/// point `p` on it). `None` for a degenerate (zero-area) face.
pub fn face_plane(corners: &[ModelVertex]) -> Option<(Vec3, f32)> {
    // Newell's method: sums over the edges give an area-weighted normal that is robust
    // for any planar polygon, even if some corners are collinear or repeated.
    let mut n = Vec3::ZERO;
    for (k, a) in corners.iter().enumerate() {
        let b = corners[(k + 1) % corners.len()].position;
        let a = a.position;
        n += Vec3::new((a.y - b.y) * (a.z + b.z), (a.z - b.z) * (a.x + b.x), (a.x - b.x) * (a.y + b.y));
    }
    let n = n.try_normalize()?;
    Some((n, n.dot(corners[0].position)))
}

/// Do two convex faces in the same plane (normal `n`) overlap by more than `tolerance`?
///
/// The *separating axis theorem*: two convex shapes are apart exactly when some line
/// separates them, and for polygons it is enough to try the directions perpendicular
/// to their edges. Faces that merely share an edge are not overlapping.
fn coplanar_overlap(a: &[ModelVertex], b: &[ModelVertex], n: Vec3, tolerance: f32) -> bool {
    let range = |face: &[ModelVertex], axis: Vec3| {
        face.iter().fold((f32::MAX, f32::MIN), |(lo, hi), v| {
            let d = v.position.dot(axis);
            (lo.min(d), hi.max(d))
        })
    };
    for face in [a, b] {
        for (k, v) in face.iter().enumerate() {
            let edge = face[(k + 1) % face.len()].position - v.position;
            let Some(axis) = n.cross(edge).try_normalize() else { continue };
            let ((a0, a1), (b0, b1)) = (range(a, axis), range(b, axis));
            if a1 <= b0 + tolerance || b1 <= a0 + tolerance {
                return false;
            }
        }
    }
    true
}

/// Gives every face that lies on top of another, facing the same way, an overlay layer
/// one above the highest face it covers, and stores it in the face's corner vertices.
///
/// `priority` decides which of two overlapping faces is on top (higher wins; equal
/// priorities: the later face). `radius` scales the "same plane" tolerance to the model.
///
/// This runs once at load time. It compares every pair of faces (O(n²)), but rejects
/// most pairs on the cheap plane test; the largest original mesh (~650 faces) takes well
/// under a millisecond.
pub fn assign_overlay_layers(lod: &mut ModelLod, radius: f32, priority: impl Fn(&Face) -> u8) {
    let tolerance = radius.max(1.0) * 1e-4;
    let planes: Vec<Option<(Vec3, f32)>> = lod.faces.iter().map(|f| face_plane(lod.face_vertices(f))).collect();
    // Visit faces bottom-up: by priority, then file order. A stable sort keeps file
    // order among equal priorities.
    let mut order: Vec<usize> = (0..lod.faces.len()).collect();
    order.sort_by_key(|&i| priority(&lod.faces[i]));
    let mut layers = vec![0u8; lod.faces.len()];
    for (k, &i) in order.iter().enumerate() {
        let Some((n, d)) = planes[i] else { continue };
        let mut layer = 0u8;
        for &j in &order[..k] {
            let Some((m, e)) = planes[j] else { continue };
            let same_plane = n.dot(m) > 0.999 && (d - e).abs() < tolerance;
            if same_plane
                && layers[j] >= layer
                && coplanar_overlap(lod.face_vertices(&lod.faces[i]), lod.face_vertices(&lod.faces[j]), n, tolerance)
            {
                layer = layers[j].saturating_add(1);
            }
        }
        layers[i] = layer;
    }
    for (f, &layer) in lod.faces.iter().zip(&layers) {
        for v in &mut lod.vertices[f.first as usize..f.first as usize + f.count as usize] {
            v.layer = layer;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::{obj, xmsh};

    fn xmsh_quad() -> XmshMesh {
        let v = |x: f32, y: f32, z: f32| [x, y, z, 0.0, 0.0, 1.0, x, y];
        let data = xmsh::build_test_xmsh(
            &[v(0.0, 0.0, 1.0), v(1.0, 0.0, 1.0), v(1.0, 1.0, 1.0), v(0.0, 1.0, 1.0)],
            &[(0, 1), (1, 2), (2, 3), (3, 0)],
            &["hull"],
            &[(0, &[1, 2, 3, 4]), (0xffff_ffff, &[1, 2, 3])],
        );
        xmsh::parse(&data).unwrap()
    }

    #[test]
    fn xmsh_is_triangulated_grouped_and_mirrored() {
        let m = Model::from_xmsh(&xmsh_quad());
        let lod = &m.lods[0];
        assert_eq!(lod.groups.len(), 2, "textured group and untextured group");
        assert_eq!(lod.groups[0].texture.as_deref(), Some("hull"));
        assert_eq!(lod.groups[0].indices.len(), 6, "quad -> 2 triangles");
        assert_eq!(lod.groups[1].texture, None);
        assert_eq!(lod.triangle_count(), 3);
        // +Z in the file becomes -Z in the engine (in front of the ship).
        assert_eq!(lod.vertices[0].position.z, -1.0);
        assert_eq!(lod.vertices[0].normal.z, -1.0);
        // Untextured polygon takes its flat colour (0xff00ff00 = green).
        assert_eq!(lod.vertices[4].color, [0, 255, 0, 255]);
        // Bounding box min/max swap on the mirrored axis.
        assert_eq!(m.bounds_min.z, -1.0);
        assert_eq!(m.bounds_max.z, 1.0);
        assert_eq!(lod.triangles().count(), 3);
    }

    #[test]
    fn mirrored_triangles_keep_outward_winding() {
        // In the file the quad (counter-clockwise seen from +Z in a left-handed system)
        // faces +Z. After mirroring it must face -Z by the right-hand rule.
        let m = Model::from_xmsh(&xmsh_quad());
        let [a, b, c] = m.lods[0].triangles().next().unwrap();
        let n = (b - a).cross(c - a);
        assert!(n.z < 0.0, "normal {n:?} should point along -Z");
    }

    /// A mesh with a 2x2 base square and, on top of it in the same plane, a small
    /// square — given in the order `first, second` with the given flags.
    fn decal_mesh(base_flags: u32, decal_flags: u32, decal_first: bool) -> XmshMesh {
        let v = |x: f32, y: f32| [x, y, 0.0, 0.0, 0.0, -1.0, x, y];
        let verts =
            [v(0.0, 0.0), v(2.0, 0.0), v(2.0, 2.0), v(0.0, 2.0), v(0.5, 0.5), v(1.0, 0.5), v(1.0, 1.0), v(0.5, 1.0)];
        let edges = [(0, 1), (1, 2), (2, 3), (3, 0), (4, 5), (5, 6), (6, 7), (7, 4)];
        let base: (u32, u32, &[i32]) = (0, base_flags, &[1, 2, 3, 4]);
        let decal: (u32, u32, &[i32]) = (0, decal_flags, &[5, 6, 7, 8]);
        let polys = if decal_first { [decal, base] } else { [base, decal] };
        xmsh::parse(&xmsh::build_test_xmsh_with_flags(&verts, &edges, &["hull"], &polys)).unwrap()
    }

    fn layers(m: &Model) -> Vec<u8> {
        let lod = &m.lods[0];
        lod.faces.iter().map(|f| lod.face_vertices(f)[0].layer).collect()
    }

    #[test]
    fn later_overlapping_face_is_layered_on_top() {
        let m = Model::from_xmsh(&decal_mesh(0, 0, false));
        assert_eq!(layers(&m), vec![0, 1], "faces in file order: base, decal");
        // Every corner of the face carries the layer.
        assert!(m.lods[0].face_vertices(&m.lods[0].faces[1]).iter().all(|v| v.layer == 1));
    }

    #[test]
    fn overlay_flag_beats_file_order() {
        let m = Model::from_xmsh(&decal_mesh(0, POLY_OVERLAY, true));
        assert_eq!(layers(&m), vec![1, 0], "the flagged decal comes first in the file but is on top");
        assert!(m.lods[0].groups.iter().any(|g| g.overlay), "flagged faces get their own group");
        let f = m.lods[0].faces[0];
        assert!(m.lods[0].groups[f.group as usize].overlay, "face group index survives dropping empty groups");
    }

    #[test]
    fn faces_that_only_touch_or_face_away_are_not_layered() {
        let corner = |x: f32, y: f32, z: f32| ModelVertex {
            position: Vec3::new(x, y, z),
            normal: Vec3::Z,
            uv: Vec2::ZERO,
            color: [255; 4],
            layer: 0,
        };
        let mut lod = ModelLod {
            switch_distance: 1.0,
            vertices: vec![
                // Two unit squares sharing the edge x = 1.
                corner(0.0, 0.0, 0.0),
                corner(1.0, 0.0, 0.0),
                corner(1.0, 1.0, 0.0),
                corner(0.0, 1.0, 0.0),
                corner(1.0, 0.0, 0.0),
                corner(2.0, 0.0, 0.0),
                corner(2.0, 1.0, 0.0),
                corner(1.0, 1.0, 0.0),
                // The first square again, wound the other way (back to back).
                corner(0.0, 0.0, 0.0),
                corner(0.0, 1.0, 0.0),
                corner(1.0, 1.0, 0.0),
                corner(1.0, 0.0, 0.0),
            ],
            groups: vec![],
            faces: (0..3).map(|k| Face { first: k * 4, count: 4, group: 0, flags: 0 }).collect(),
        };
        assign_overlay_layers(&mut lod, 1.0, |_| 0);
        assert!(lod.vertices.iter().all(|v| v.layer == 0));
    }

    #[test]
    fn faces_record_their_corners_and_group() {
        let m = Model::from_xmsh(&xmsh_quad());
        let lod = &m.lods[0];
        assert_eq!(lod.faces.len(), 2);
        assert_eq!((lod.faces[0].first, lod.faces[0].count), (0, 4));
        assert_eq!((lod.faces[1].first, lod.faces[1].count), (4, 3));
        assert_eq!(lod.groups[lod.faces[1].group as usize].texture, None);
        assert_eq!(lod.face_vertices(&lod.faces[1]).len(), 3);
    }

    #[test]
    fn lod_selection_by_distance() {
        let mut m = Model::from_xmsh(&xmsh_quad());
        let mut far = m.lods[0].clone();
        far.switch_distance = f32::MAX;
        far.vertices.clear();
        m.lods.push(far);
        assert_eq!(m.lod_for_distance(10.0).switch_distance, 100.0);
        assert_eq!(m.lod_for_distance(500.0).switch_distance, f32::MAX);
        assert!(m.collision_lod().vertices.is_empty());
    }

    #[test]
    fn obj_models_use_materials_scale_and_flip_v() {
        let text = "v 0 0 0\nv 2 0 0\nv 0 2 0\nvt 0 0.25\nusemtl red\nf 1/1 2/1 3/1\nf 1 2 3\n";
        let mesh = obj::parse(text).unwrap();
        let mats = obj::parse_mtl("newmtl red\nKd 1 0 0\nmap_Kd red.png\n");
        let m = Model::from_obj("box", &mesh, 0.5, &mats);
        let lod = &m.lods[0];
        assert_eq!(lod.groups.len(), 1, "both faces use material red");
        assert_eq!(lod.groups[0].texture.as_deref(), Some("red.png"));
        assert_eq!(lod.vertices[1].position, Vec3::new(1.0, 0.0, 0.0), "scaled");
        assert_eq!(lod.vertices[0].uv, Vec2::new(0.0, 0.75), "v flipped");
        assert_eq!(lod.vertices[0].color, [255, 0, 0, 255]);
        assert_eq!(lod.vertices[0].normal, Vec3::Z, "face normal fallback");
        assert_eq!(lod.faces.len(), 2);
        assert_eq!(lod.vertices[3].layer, 1, "the second, identical face is drawn on top");
        assert!((m.radius - 1.0).abs() < 1e-6);
        assert_eq!(m.bounds_max, Vec3::new(1.0, 1.0, 0.0));
    }

    #[test]
    fn empty_obj_has_zero_bounds() {
        let m = Model::from_obj("empty", &obj::ObjMesh::default(), 1.0, &[]);
        assert_eq!(m.bounds_min, Vec3::ZERO);
        assert_eq!(m.radius, 0.0);
    }
}
