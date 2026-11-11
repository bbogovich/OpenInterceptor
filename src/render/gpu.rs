//! # Preparing models for the GPU
//!
//! A [`Model`](crate::model::Model) stores each LOD as one vertex list shared by all of
//! its material groups, with `u32` indices. macroquad's `Mesh` wants something slightly
//! different: **one mesh per texture**, **`u16` indices** (so at most 65 536 vertices),
//! and no more vertices than one draw-call buffer holds.
//!
//! [`split_lod`] does that conversion once, at load time — a pure function, tested
//! below. [`ModelCache`] then uploads the result (textures included) and keeps it, so
//! each model and texture is loaded once per session however many ships use it.

use crate::content::Content;
use crate::model::{Model, ModelLod, ModelVertex};
use macroquad::prelude::{Mesh, Texture2D, Vertex};
use std::collections::HashMap;
use std::rc::Rc;

/// Largest chunk sent in one draw (must fit the window's draw-call capacity).
pub const MAX_CHUNK_VERTICES: usize = 16_000;

/// A CPU-side chunk ready to become a `Mesh`.
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub texture: Option<String>,
    /// From [`MaterialGroup::overlay`](crate::model::MaterialGroup::overlay).
    pub overlay: bool,
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u16>,
}

/// Splits one LOD into per-texture chunks with local `u16` indices.
pub fn split_lod(lod: &ModelLod) -> Vec<Chunk> {
    let mut out = Vec::new();
    for group in &lod.groups {
        let empty = || Chunk {
            texture: group.texture.clone(),
            overlay: group.overlay,
            vertices: Vec::new(),
            indices: Vec::new(),
        };
        let mut chunk = empty();
        // Maps a LOD-wide vertex index to its index inside the current chunk.
        let mut remap: HashMap<u32, u16> = HashMap::new();
        for tri in group.indices.chunks_exact(3) {
            // Start a new chunk if this triangle might not fit.
            if chunk.vertices.len() + 3 > MAX_CHUNK_VERTICES {
                out.push(std::mem::replace(&mut chunk, empty()));
                remap.clear();
            }
            for &v in tri {
                let local = *remap.entry(v).or_insert_with(|| {
                    chunk.vertices.push(lod.vertices[v as usize]);
                    (chunk.vertices.len() - 1) as u16
                });
                chunk.indices.push(local);
            }
        }
        if !chunk.indices.is_empty() {
            out.push(chunk);
        }
    }
    out
}

/// One LOD on the GPU.
pub struct GpuLod {
    pub switch_distance: f32,
    pub meshes: Vec<Mesh>,
    /// `overlay[i]` is true if `meshes[i]` holds flagged overlay faces.
    pub overlay: Vec<bool>,
}

/// A model on the GPU.
pub struct GpuModel {
    pub radius: f32,
    pub lods: Vec<GpuLod>,
    /// Engine glow points (model space), computed once at load so drawing glows each
    /// frame allocates nothing: rear-facing attachments, else the back centre.
    pub engines: Vec<glam::Vec3>,
    /// The CPU model is kept for collision shapes and the model viewer's statistics.
    pub model: Model,
}

impl GpuModel {
    /// The LOD to draw at `distance` metres.
    pub fn lod(&self, distance: f32) -> &GpuLod {
        &self.lods[self.lod_index(distance)]
    }

    /// Index of the LOD to draw at `distance` metres (the last one beyond every switch
    /// distance). `GpuModel::lods` and `model.lods` share indices.
    pub fn lod_index(&self, distance: f32) -> usize {
        self.lods.iter().position(|l| distance < l.switch_distance).unwrap_or(self.lods.len().saturating_sub(1))
    }
}

/// Where engine glows go: attachments facing backwards (+Z), or the back centre.
pub fn engine_points(model: &Model) -> Vec<glam::Vec3> {
    let rear: Vec<glam::Vec3> = model.attachments.iter().filter(|a| a.forward.z > 0.5).map(|a| a.position).collect();
    if rear.is_empty() {
        vec![glam::Vec3::new(0.0, 0.0, model.bounds_max.z * 0.95)]
    } else {
        rear
    }
}

/// Converts an engine vertex to macroquad's layout.
fn to_vertex(v: &ModelVertex) -> Vertex {
    // The normal's spare w component carries the overlay layer to the shader.
    Vertex { position: v.position, uv: v.uv, color: v.color, normal: v.normal.extend(v.layer as f32) }
}

/// Loads and caches GPU models and textures.
///
/// `Rc` (*reference counted*) lets several ships hold the same `GpuModel` cheaply. It
/// is the single-threaded cousin of `Arc`; rendering happens on one thread, so the
/// cheaper `Rc` is the right choice.
#[derive(Default)]
pub struct ModelCache {
    models: HashMap<(String, usize), Option<Rc<GpuModel>>>,
    textures: HashMap<(String, usize), Option<Texture2D>>,
    /// Problems met while loading (shown in the developer overlay and logged).
    pub errors: Vec<String>,
}

impl ModelCache {
    /// Gets (loading on first use) the model behind an asset reference.
    pub fn get(&mut self, content: &Content, reference: &str, pack: usize, scale: f32) -> Option<Rc<GpuModel>> {
        let key = (format!("{reference}@{scale}"), pack);
        if let Some(m) = self.models.get(&key) {
            return m.clone();
        }
        let loaded = match content.load_model(reference, pack, scale) {
            Ok(model) => Some(Rc::new(self.upload(content, model, pack))),
            Err(e) => {
                self.errors.push(format!("{reference}: {e}"));
                None
            }
        };
        self.models.insert(key, loaded.clone());
        loaded
    }

    /// Like [`ModelCache::get`], with some textures replaced (a model set's damage level
    /// or state). Each distinct set of swaps is loaded and cached once.
    pub fn get_variant(
        &mut self,
        content: &Content,
        reference: &str,
        pack: usize,
        scale: f32,
        swaps: &crate::content::looks::Swaps,
    ) -> Option<Rc<GpuModel>> {
        if swaps.is_empty() {
            return self.get(content, reference, pack, scale);
        }
        let tag: Vec<String> = swaps.iter().map(|(a, b)| format!("{a}>{b}")).collect();
        let key = (format!("{reference}@{scale}#{}", tag.join(",")), pack);
        if let Some(m) = self.models.get(&key) {
            return m.clone();
        }
        let loaded = match content.load_model(reference, pack, scale) {
            Ok(mut model) => {
                for lod in &mut model.lods {
                    for g in &mut lod.groups {
                        let Some(t) = &g.texture else { continue };
                        let k = crate::content::looks::texture_key(t);
                        if let Some((_, new)) = swaps.iter().find(|(name, _)| *name == k) {
                            g.texture = Some(new.clone());
                        }
                    }
                }
                Some(Rc::new(self.upload(content, model, pack)))
            }
            Err(e) => {
                self.errors.push(format!("{reference}: {e}"));
                None
            }
        };
        self.models.insert(key, loaded.clone());
        loaded
    }

    fn texture(&mut self, content: &Content, name: &str, pack: usize) -> Option<Texture2D> {
        let key = (name.to_ascii_lowercase(), pack);
        if let Some(t) = self.textures.get(&key) {
            return t.clone();
        }
        let tex = match content.load_texture(name, pack) {
            Ok(img) => {
                let t = Texture2D::from_rgba8(img.width as u16, img.height as u16, &img.rgba);
                t.set_filter(macroquad::prelude::FilterMode::Linear);
                super::set_repeat(&t);
                Some(t)
            }
            Err(e) => {
                self.errors.push(format!("texture {name}: {e}"));
                None
            }
        };
        self.textures.insert(key, tex.clone());
        tex
    }

    /// Uploads a model built in code (a terrain mesh) rather than loaded from a pack.
    pub fn upload_model(&mut self, content: &Content, model: Model, pack: usize) -> Rc<GpuModel> {
        Rc::new(self.upload(content, model, pack))
    }

    fn upload(&mut self, content: &Content, model: Model, pack: usize) -> GpuModel {
        let mut lods = Vec::with_capacity(model.lods.len());
        for lod in &model.lods {
            let chunks = split_lod(lod);
            let overlay = chunks.iter().map(|c| c.overlay).collect();
            let meshes = chunks
                .into_iter()
                .map(|c| Mesh {
                    vertices: c.vertices.iter().map(to_vertex).collect(),
                    indices: c.indices,
                    texture: c.texture.as_deref().and_then(|t| self.texture(content, t, pack)),
                })
                .collect();
            lods.push(GpuLod { switch_distance: lod.switch_distance, meshes, overlay });
        }
        GpuModel { radius: model.radius, lods, engines: engine_points(&model), model }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::MaterialGroup;
    use glam::{Vec2, Vec3};

    fn vertex(i: u32) -> ModelVertex {
        ModelVertex { position: Vec3::splat(i as f32), normal: Vec3::Y, uv: Vec2::ZERO, color: [255; 4], layer: 0 }
    }

    #[test]
    fn groups_become_chunks_with_local_indices() {
        let lod = ModelLod {
            switch_distance: 10.0,
            vertices: (0..6).map(vertex).collect(),
            groups: vec![
                MaterialGroup { texture: Some("a".into()), indices: vec![0, 1, 2, 2, 1, 3], overlay: false },
                MaterialGroup { texture: None, indices: vec![3, 4, 5], overlay: true },
            ],
            faces: vec![],
        };
        let chunks = split_lod(&lod);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].vertices.len(), 4, "shared vertices are not duplicated");
        assert_eq!(chunks[0].indices, vec![0, 1, 2, 2, 1, 3]);
        assert_eq!(chunks[1].indices, vec![0, 1, 2], "indices restart in each chunk");
        assert_eq!(chunks[1].vertices[0].position, Vec3::splat(3.0));
        assert_eq!(chunks[1].texture, None);
        assert!(!chunks[0].overlay && chunks[1].overlay, "the overlay flag follows the group");
    }

    #[test]
    fn engines_use_rear_attachments_or_the_back_centre() {
        use crate::model::Attachment;
        let mut m = Model {
            name: "m".into(),
            radius: 5.0,
            bounds_min: Vec3::splat(-1.0),
            bounds_max: Vec3::new(1.0, 1.0, 4.0),
            lods: vec![],
            attachments: vec![Attachment { name: None, position: Vec3::X, forward: Vec3::NEG_Z }],
        };
        assert_eq!(engine_points(&m), vec![Vec3::new(0.0, 0.0, 3.8)], "front-facing attachment ignored");
        m.attachments.push(Attachment { name: None, position: Vec3::new(0.0, 0.0, 4.0), forward: Vec3::Z });
        assert_eq!(engine_points(&m), vec![Vec3::new(0.0, 0.0, 4.0)]);
    }

    #[test]
    fn huge_groups_are_split() {
        let n = MAX_CHUNK_VERTICES as u32 + 300;
        let lod = ModelLod {
            switch_distance: 1.0,
            vertices: (0..n).map(vertex).collect(),
            groups: vec![MaterialGroup { texture: None, indices: (0..n - n % 3).collect(), overlay: false }],
            faces: vec![],
        };
        let chunks = split_lod(&lod);
        assert_eq!(chunks.len(), 2);
        assert!(chunks.iter().all(|c| c.vertices.len() <= MAX_CHUNK_VERTICES));
        let total: usize = chunks.iter().map(|c| c.indices.len()).sum();
        assert_eq!(total as u32, n - n % 3);
    }
}
