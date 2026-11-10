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
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u16>,
}

/// Splits one LOD into per-texture chunks with local `u16` indices.
pub fn split_lod(lod: &ModelLod) -> Vec<Chunk> {
    let mut out = Vec::new();
    for group in &lod.groups {
        let mut chunk = Chunk { texture: group.texture.clone(), vertices: Vec::new(), indices: Vec::new() };
        // Maps a LOD-wide vertex index to its index inside the current chunk.
        let mut remap: HashMap<u32, u16> = HashMap::new();
        for tri in group.indices.chunks_exact(3) {
            // Start a new chunk if this triangle might not fit.
            if chunk.vertices.len() + 3 > MAX_CHUNK_VERTICES {
                out.push(std::mem::replace(
                    &mut chunk,
                    Chunk { texture: group.texture.clone(), vertices: Vec::new(), indices: Vec::new() },
                ));
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
}

/// A model on the GPU.
pub struct GpuModel {
    pub radius: f32,
    pub lods: Vec<GpuLod>,
    /// The CPU model is kept for collision shapes and the model viewer's statistics.
    pub model: Model,
}

impl GpuModel {
    /// The LOD to draw at `distance` metres.
    pub fn lod(&self, distance: f32) -> &GpuLod {
        self.lods
            .iter()
            .find(|l| distance < l.switch_distance)
            .unwrap_or_else(|| self.lods.last().expect("models have LODs"))
    }
}

/// Converts an engine vertex to macroquad's layout.
fn to_vertex(v: &ModelVertex) -> Vertex {
    Vertex { position: v.position, uv: v.uv, color: v.color, normal: v.normal.extend(0.0) }
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

    fn texture(&mut self, content: &Content, name: &str, pack: usize) -> Option<Texture2D> {
        let key = (name.to_ascii_lowercase(), pack);
        if let Some(t) = self.textures.get(&key) {
            return t.clone();
        }
        let tex = match content.load_texture(name, pack) {
            Ok(img) => {
                let t = Texture2D::from_rgba8(img.width as u16, img.height as u16, &img.rgba);
                t.set_filter(macroquad::prelude::FilterMode::Linear);
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

    fn upload(&mut self, content: &Content, model: Model, pack: usize) -> GpuModel {
        let mut lods = Vec::with_capacity(model.lods.len());
        for lod in &model.lods {
            let meshes = split_lod(lod)
                .into_iter()
                .map(|c| Mesh {
                    vertices: c.vertices.iter().map(to_vertex).collect(),
                    indices: c.indices,
                    texture: c.texture.as_deref().and_then(|t| self.texture(content, t, pack)),
                })
                .collect();
            lods.push(GpuLod { switch_distance: lod.switch_distance, meshes });
        }
        GpuModel { radius: model.radius, lods, model }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::MaterialGroup;
    use glam::{Vec2, Vec3};

    fn vertex(i: u32) -> ModelVertex {
        ModelVertex { position: Vec3::splat(i as f32), normal: Vec3::Y, uv: Vec2::ZERO, color: [255; 4] }
    }

    #[test]
    fn groups_become_chunks_with_local_indices() {
        let lod = ModelLod {
            switch_distance: 10.0,
            vertices: (0..6).map(vertex).collect(),
            groups: vec![
                MaterialGroup { texture: Some("a".into()), indices: vec![0, 1, 2, 2, 1, 3] },
                MaterialGroup { texture: None, indices: vec![3, 4, 5] },
            ],
        };
        let chunks = split_lod(&lod);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].vertices.len(), 4, "shared vertices are not duplicated");
        assert_eq!(chunks[0].indices, vec![0, 1, 2, 2, 1, 3]);
        assert_eq!(chunks[1].indices, vec![0, 1, 2], "indices restart in each chunk");
        assert_eq!(chunks[1].vertices[0].position, Vec3::splat(3.0));
        assert_eq!(chunks[1].texture, None);
    }

    #[test]
    fn huge_groups_are_split() {
        let n = MAX_CHUNK_VERTICES as u32 + 300;
        let lod = ModelLod {
            switch_distance: 1.0,
            vertices: (0..n).map(vertex).collect(),
            groups: vec![MaterialGroup { texture: None, indices: (0..n - n % 3).collect() }],
        };
        let chunks = split_lod(&lod);
        assert_eq!(chunks.len(), 2);
        assert!(chunks.iter().all(|c| c.vertices.len() <= MAX_CHUNK_VERTICES));
        let total: usize = chunks.iter().map(|c| c.indices.len()).sum();
        assert_eq!(total as u32, n - n % 3);
    }
}
