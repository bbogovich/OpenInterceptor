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
//!
//! Converting once at load time, instead of every frame, is a key performance pattern:
//! do expensive work when data enters the engine, and keep the per-frame path simple.
//!
//! This module is pure data (no GPU, no window), so it is fully unit-testable.

use crate::formats::{obj::ObjMesh, xmsh::XmshMesh};
use glam::{Vec2, Vec3};

/// One vertex as the renderer consumes it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    /// Vertex colour (RGBA). White for textured faces; the flat colour otherwise.
    pub color: [u8; 4],
}

/// Triangles that share one texture (or no texture).
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialGroup {
    /// Texture name to look up in the pack's file system; `None` = untextured.
    pub texture: Option<String>,
    /// Vertex indices, three per triangle, into [`ModelLod::vertices`].
    pub indices: Vec<u32>,
}

/// One level of detail.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelLod {
    /// Use this LOD while the camera is nearer than this many metres.
    pub switch_distance: f32,
    pub vertices: Vec<ModelVertex>,
    pub groups: Vec<MaterialGroup>,
}

impl ModelLod {
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Attachment {
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
            // One group per texture, plus one (index = textures.len()) for untextured faces.
            let mut groups: Vec<MaterialGroup> = lod
                .textures
                .iter()
                .map(|t| MaterialGroup { texture: Some(t.clone()), indices: Vec::new() })
                .chain(std::iter::once(MaterialGroup { texture: None, indices: Vec::new() }))
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
                    });
                }
                let group = poly.texture.map(|t| t as usize).unwrap_or(lod.textures.len());
                // Fan triangulation (the BSP polygons are convex). Swapping the last two
                // indices reverses the winding to match the mirrored coordinates.
                for k in 1..poly.count as u32 - 1 {
                    groups[group].indices.extend_from_slice(&[base, base + k + 1, base + k]);
                }
            }
            groups.retain(|g| !g.indices.is_empty());
            lods.push(ModelLod { switch_distance: lod.switch_distance, vertices, groups });
        }
        // Manipulators: the first three floats are the position, then a 3x3 rotation
        // whose third row is the facing axis (layout derived from the data).
        let attachments = mesh.lods[0]
            .manipulators
            .iter()
            .map(|m| Attachment {
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
            })
            .chain(std::iter::once(MaterialGroup { texture: None, indices: Vec::new() }))
            .collect();
        for face in &mesh.faces {
            let corners = mesh.face_corners(face);
            let p: Vec<Vec3> = corners.iter().map(|c| Vec3::from(mesh.positions[c.position as usize]) * scale).collect();
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
                    normal: c.normal.map(|i| Vec3::from(mesh.normals[i as usize]).normalize_or_zero()).unwrap_or(face_normal),
                    // OBJ's v axis points up the image; textures are stored top row first.
                    uv: Vec2::new(uv[0], 1.0 - uv[1]),
                    color,
                });
            }
            let group = face.material.map(|m| m as usize).unwrap_or(mesh.materials.len());
            for k in 1..face.count as u32 - 1 {
                groups[group].indices.extend_from_slice(&[base, base + k, base + k + 1]);
            }
        }
        groups.retain(|g| !g.indices.is_empty());
        let (mut min, mut max) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        for v in &vertices {
            min = min.min(v.position);
            max = max.max(v.position);
        }
        if vertices.is_empty() {
            (min, max) = (Vec3::ZERO, Vec3::ZERO);
        }
        let radius = vertices.iter().map(|v| v.position.length()).fold(0.0, f32::max);
        Model {
            name: name.to_string(),
            radius,
            bounds_min: min,
            bounds_max: max,
            lods: vec![ModelLod { switch_distance: f32::MAX, vertices, groups }],
            attachments: Vec::new(),
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

    #[test]
    fn lod_selection_by_distance() {
        let mut m = Model::from_xmsh(&xmsh_quad());
        let mut far = m.lods[0].clone();
        far.switch_distance = f32::MAX;
        far.vertices.truncate(0);
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
