//! # XMSH — the original 3D mesh format
//!
//! Every ship, station, asteroid and debris chunk in *Interceptor* is an `XMSH` resource
//! inside `Resources/xobjects1.dll` or `xobjects2.dll`. The format was decoded for this
//! project (there was no public documentation); the documentation site's
//! *Original game → Formats → XMSH* page has the byte-level table. In short:
//!
//! ```text
//! 0x00 name[12]      0x0C "MSH\0"     0x20 f32 bounding radius
//! 0x28 f32[6] bbox   0x54 i32[4] offsets of up to four LODs (-1 = absent)
//! LOD (offsets inside are relative to the LOD start):
//!   f32 switch distance, u32 0, vertex count+offset, edge count+offset,
//!   texture count+offset, manipulator count+offset, BSP root offset
//!   vertex  = 32 bytes: position xyz, normal xyz, uv
//!   edge    = 16 bytes: two vertex indices (+ padding); edge 0 is unused
//!   texture = 32-byte name
//!   manipulator = 64 bytes: 3x4 matrix + two ints (attachment points)
//!   BSP node = 64 bytes: plane, bbox, front/back child, polygon count; polygons inline
//!   polygon  = plane, colour, texture index, n, then n *signed* edge references
//! ```
//!
//! A polygon lists **edges**, not vertices. A positive reference `+e` walks edge `e`
//! from its first to its second vertex; `-e` walks it backwards. Taking the starting
//! vertex of each walked edge yields the polygon's vertex loop.
//!
//! The reader keeps the data exactly as stored (Direct3D's left-handed coordinates).
//! Conversion to the engine's right-handed system happens in [`crate::model`].
//!
//! ## Data-model note: flattening
//!
//! Instead of giving each polygon its own `Vec<u32>` of vertices (one heap allocation per
//! polygon, scattered in memory), all polygon vertex indices go into one shared
//! `Vec<u32>` and each [`Polygon`] stores a `first`/`count` range into it. Fewer
//! allocations and contiguous memory make loading and iterating faster — a standard
//! data-oriented-design technique.

use super::{FResult, FormatError, Reader};

/// One vertex as stored in the file.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

/// One polygon. Its vertex indices are `lod.polygon_vertices[first..first + count]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Polygon {
    /// Plane normal (unit length in the file's coordinates).
    pub normal: [f32; 3],
    /// Plane distance from the origin.
    pub distance: f32,
    /// Flat colour as `0xAARRGGBB`, used for untextured polygons.
    pub color: u32,
    /// Index into [`Lod::textures`], or `None` for an untextured polygon.
    pub texture: Option<u16>,
    /// Start of this polygon's vertex indices in [`Lod::polygon_vertices`].
    pub first: u32,
    /// Number of vertices (3 or more).
    pub count: u16,
    /// Fields whose meaning is not yet known; kept so research can continue.
    pub unknown: [u32; 3],
}

/// An attachment point ("manipulator") such as a gun mount or engine glow.
/// The meaning of the two integers is not yet decoded.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Manipulator {
    /// Row-major 3x4 matrix: three rows of (x, y, z, translation)? — layout *derived*.
    pub matrix: [f32; 12],
    pub kind: i32,
    pub parent: i32,
}

/// One level of detail. Far-away objects draw a cheaper LOD.
#[derive(Debug, Clone, PartialEq)]
pub struct Lod {
    /// This LOD is used while the viewer is closer than this distance (1e8 = "always").
    pub switch_distance: f32,
    pub vertices: Vec<XVertex>,
    pub textures: Vec<String>,
    pub manipulators: Vec<Manipulator>,
    pub polygons: Vec<Polygon>,
    /// Flattened vertex indices of all polygons (see the module docs).
    pub polygon_vertices: Vec<u32>,
    /// Number of BSP nodes visited (useful for statistics and the format docs).
    pub bsp_nodes: u32,
}

impl Lod {
    /// The vertex indices of one polygon, as a borrowed slice.
    pub fn polygon_indices(&self, p: &Polygon) -> &[u32] {
        &self.polygon_vertices[p.first as usize..p.first as usize + p.count as usize]
    }
}

/// A whole mesh resource.
#[derive(Debug, Clone, PartialEq)]
pub struct XmshMesh {
    pub name: String,
    pub radius: f32,
    /// Bounding box: min x, y, z then max x, y, z.
    pub bbox: [f32; 6],
    pub lods: Vec<Lod>,
}

/// Size of one serialised record of each kind, in bytes.
const VERTEX_SIZE: usize = 32;
const EDGE_SIZE: usize = 16;
const TEXTURE_NAME_SIZE: usize = 32;
const MANIP_SIZE: usize = 64;
const NODE_SIZE: usize = 64;

/// Parses an `XMSH` resource.
pub fn parse(data: &[u8]) -> FResult<XmshMesh> {
    let r = Reader::new(data);
    if r.bytes(12, 4)? != b"MSH\0" {
        return Err(FormatError::BadMagic { expected: "MSH" });
    }
    let name = r.name(0, 12)?;
    let radius = r.f32(0x20)?;
    let mut bbox = [0.0; 6];
    for (i, v) in bbox.iter_mut().enumerate() {
        *v = r.f32(0x28 + 4 * i)?;
    }
    let mut lods = Vec::new();
    for i in 0..4 {
        let off = r.i32(0x54 + 4 * i)?;
        if off >= 0 {
            lods.push(parse_lod(&r, off as usize)?);
        }
    }
    if lods.is_empty() {
        return Err(FormatError::Invalid(format!("mesh {name} has no LOD")));
    }
    Ok(XmshMesh { name, radius, bbox, lods })
}

fn parse_lod(r: &Reader, base: usize) -> FResult<Lod> {
    let switch_distance = r.f32(base)?;
    // Read the ten u32 header fields after the distance in one go.
    let h: Vec<usize> = (0..10).map(|i| r.u32(base + 4 + 4 * i).map(|v| v as usize)).collect::<FResult<_>>()?;
    let (nv, voff, ne, eoff, nt, toff, nman, moff, root) = (h[1], h[2], h[3], h[4], h[5], h[6], h[7], h[8], h[9]);

    let mut vertices = Vec::with_capacity(nv);
    for i in 0..nv {
        let o = base + voff + VERTEX_SIZE * i;
        let f = |k: usize| r.f32(o + 4 * k);
        vertices.push(XVertex {
            position: [f(0)?, f(1)?, f(2)?],
            normal: [f(3)?, f(4)?, f(5)?],
            uv: [f(6)?, f(7)?],
        });
    }
    let mut edges = Vec::with_capacity(ne);
    for i in 0..ne {
        let o = base + eoff + EDGE_SIZE * i;
        edges.push((r.u32(o)?, r.u32(o + 4)?));
    }
    let textures = (0..nt)
        .map(|i| r.name(base + toff + TEXTURE_NAME_SIZE * i, TEXTURE_NAME_SIZE))
        .collect::<FResult<Vec<_>>>()?;
    let mut manipulators = Vec::with_capacity(nman);
    for i in 0..nman {
        let o = base + moff + MANIP_SIZE * i;
        let mut matrix = [0.0; 12];
        for (k, m) in matrix.iter_mut().enumerate() {
            *m = r.f32(o + 4 * k)?;
        }
        manipulators.push(Manipulator { matrix, kind: r.i32(o + 48)?, parent: r.i32(o + 52)? });
    }

    let mut lod = Lod {
        switch_distance,
        vertices,
        textures,
        manipulators,
        polygons: Vec::new(),
        polygon_vertices: Vec::new(),
        bsp_nodes: 0,
    };
    // Walk the BSP tree with an explicit stack instead of recursion: a corrupt file
    // with a very deep (or cyclic) tree then cannot overflow the call stack. The node
    // budget stops cycles.
    let mut stack = vec![root];
    let budget = data_budget(r.len());
    while let Some(node) = stack.pop() {
        lod.bsp_nodes += 1;
        if lod.bsp_nodes as usize > budget {
            return Err(FormatError::Invalid("BSP tree has a cycle".into()));
        }
        let n = base + node;
        let front = r.i32(n + 40)?;
        let back = r.i32(n + 44)?;
        let poly_count = r.i32(n + 52)?;
        let mut p = n + NODE_SIZE;
        for _ in 0..poly_count.max(0) {
            p = parse_polygon(r, p, &edges, &mut lod)?;
        }
        // Push back first so front is processed first (keeps file order).
        for child in [back, front] {
            if child >= 0 {
                stack.push(child as usize);
            }
        }
    }
    Ok(lod)
}

/// Upper bound on BSP nodes: a node is at least 64 bytes, so a file cannot honestly
/// contain more than len/64 of them.
fn data_budget(len: usize) -> usize {
    len / NODE_SIZE + 1
}

/// Parses one polygon at `p`, appends it to `lod`, and returns the offset just after it.
fn parse_polygon(r: &Reader, p: usize, edges: &[(u32, u32)], lod: &mut Lod) -> FResult<usize> {
    let normal = [r.f32(p)?, r.f32(p + 4)?, r.f32(p + 8)?];
    let distance = r.f32(p + 12)?;
    let unknown0 = r.u32(p + 16)?;
    let color = r.u32(p + 20)?;
    let unknown1 = r.u32(p + 24)?;
    let unknown2 = r.u32(p + 28)?;
    let texture_raw = r.u32(p + 32)?;
    let n = r.i32(p + 36)?;
    if !(3..=1024).contains(&n) {
        return Err(FormatError::Invalid(format!("polygon with {n} edges")));
    }
    let first = lod.polygon_vertices.len() as u32;
    for k in 0..n as usize {
        let e = r.i32(p + 40 + 4 * k)?;
        let (a, b) = *edges
            .get(e.unsigned_abs() as usize)
            .ok_or_else(|| FormatError::Invalid(format!("edge {e} out of range")))?;
        // Walking an edge forwards starts at `a`; backwards starts at `b`.
        let v = if e > 0 { a } else { b };
        if v as usize >= lod.vertices.len() {
            return Err(FormatError::Invalid(format!("vertex {v} out of range")));
        }
        lod.polygon_vertices.push(v);
    }
    let texture = if (texture_raw as usize) < lod.textures.len() { Some(texture_raw as u16) } else { None };
    lod.polygons.push(Polygon {
        normal,
        distance,
        color,
        texture,
        first,
        count: n as u16,
        unknown: [unknown0, unknown1, unknown2],
    });
    Ok(p + 40 + 4 * n as usize)
}

/// Test helper: serialises a simple single-LOD, single-node mesh in XMSH layout.
/// `polys` are lists of signed edge references. Used by tests here and in `model`.
#[cfg(test)]
pub(crate) fn build_test_xmsh(
    verts: &[[f32; 8]],
    edges: &[(u32, u32)],
    textures: &[&str],
    polys: &[(u32, &[i32])],
) -> Vec<u8> {
    let mut m = vec![0xcdu8; 0x70];
    m[..4].copy_from_slice(b"test");
    m[4] = 0;
    m[12..16].copy_from_slice(b"MSH\0");
    let put_f = |m: &mut Vec<u8>, at: usize, v: f32| m[at..at + 4].copy_from_slice(&v.to_le_bytes());
    put_f(&mut m, 0x20, 2.0);
    for (i, v) in [-1.0f32, -1.0, -1.0, 1.0, 1.0, 1.0].iter().enumerate() {
        put_f(&mut m, 0x28 + 4 * i, *v);
    }
    m[0x54..0x58].copy_from_slice(&0x70i32.to_le_bytes());
    for i in 1..4 {
        m[0x54 + 4 * i..0x58 + 4 * i].copy_from_slice(&(-1i32).to_le_bytes());
    }
    // LOD body, built separately then appended.
    let mut l: Vec<u8> = vec![0; 0x30];
    let voff = l.len();
    for v in verts {
        for f in v {
            l.extend_from_slice(&f.to_le_bytes());
        }
    }
    let eoff = l.len();
    l.extend_from_slice(&[0u8; 16]); // unused edge 0
    for (a, b) in edges {
        l.extend_from_slice(&a.to_le_bytes());
        l.extend_from_slice(&b.to_le_bytes());
        l.extend_from_slice(&[0u8; 8]);
    }
    let toff = l.len();
    for t in textures {
        let mut name = [0u8; 32];
        name[..t.len()].copy_from_slice(t.as_bytes());
        l.extend_from_slice(&name);
    }
    let root = l.len();
    let mut node = vec![0u8; 64];
    node[40..44].copy_from_slice(&(-1i32).to_le_bytes());
    node[44..48].copy_from_slice(&(-1i32).to_le_bytes());
    node[52..56].copy_from_slice(&(polys.len() as i32).to_le_bytes());
    l.extend_from_slice(&node);
    for (tex, es) in polys {
        l.extend_from_slice(&[0u8; 16]); // plane
        l.extend_from_slice(&0u32.to_le_bytes());
        l.extend_from_slice(&0xff00ff00u32.to_le_bytes());
        l.extend_from_slice(&[0u8; 8]);
        l.extend_from_slice(&tex.to_le_bytes());
        l.extend_from_slice(&(es.len() as i32).to_le_bytes());
        for e in *es {
            l.extend_from_slice(&e.to_le_bytes());
        }
    }
    let header: [u32; 10] = [
        0,
        verts.len() as u32,
        voff as u32,
        edges.len() as u32 + 1,
        eoff as u32,
        textures.len() as u32,
        toff as u32,
        0,
        root as u32,
        root as u32,
    ];
    l[0..4].copy_from_slice(&100.0f32.to_le_bytes());
    for (i, h) in header.iter().enumerate() {
        l[4 + 4 * i..8 + 4 * i].copy_from_slice(&h.to_le_bytes());
    }
    m.extend_from_slice(&l);
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad() -> Vec<u8> {
        // A unit square in the XY plane: 4 vertices, 4 edges, one textured polygon that
        // walks edges 1,2 forwards and 3,4 backwards (exercising signed references).
        let v = |x: f32, y: f32| [x, y, 0.0, 0.0, 0.0, 1.0, x, y];
        build_test_xmsh(
            &[v(0.0, 0.0), v(1.0, 0.0), v(1.0, 1.0), v(0.0, 1.0)],
            &[(0, 1), (1, 2), (3, 2), (0, 3)],
            &["hull"],
            &[(0, &[1, 2, -3, -4])],
        )
    }

    #[test]
    fn parses_a_hand_built_mesh() {
        let m = parse(&quad()).unwrap();
        assert_eq!(m.name, "test");
        assert_eq!(m.radius, 2.0);
        assert_eq!(m.lods.len(), 1);
        let lod = &m.lods[0];
        assert_eq!(lod.switch_distance, 100.0);
        assert_eq!(lod.vertices.len(), 4);
        assert_eq!(lod.textures, vec!["hull".to_string()]);
        assert_eq!(lod.polygons.len(), 1);
        let p = lod.polygons[0];
        assert_eq!(p.texture, Some(0));
        assert_eq!(p.color, 0xff00ff00);
        assert_eq!(p.count, 4);
        assert_eq!(lod.bsp_nodes, 1);
    }

    #[test]
    fn signed_edges_give_the_vertex_loop() {
        let m = parse(&quad()).unwrap();
        let lod = &m.lods[0];
        // +1 (0→1) starts at 0, +2 (1→2) at 1, -3 (2←3) at 2, -4 (3←0) at 3.
        assert_eq!(lod.polygon_indices(&lod.polygons[0]), &[0, 1, 2, 3]);
    }

    #[test]
    fn untextured_polygons_have_no_texture() {
        let v = [0.0f32; 8];
        let data = build_test_xmsh(&[v, v, v], &[(0, 1), (1, 2), (2, 0)], &[], &[(0xffff_ffff, &[1, 2, 3])]);
        let m = parse(&data).unwrap();
        assert_eq!(m.lods[0].polygons[0].texture, None);
    }

    #[test]
    fn rejects_bad_signature_and_bad_references() {
        let mut d = quad();
        d[12] = b'X';
        assert_eq!(parse(&d), Err(FormatError::BadMagic { expected: "MSH" }));
        let v = [0.0f32; 8];
        let bad_edge = build_test_xmsh(&[v, v, v], &[(0, 1)], &[], &[(0, &[1, 5, 1])]);
        assert!(matches!(parse(&bad_edge), Err(FormatError::Invalid(_))));
        let bad_vertex = build_test_xmsh(&[v], &[(0, 9), (9, 0)], &[], &[(0, &[1, 2, 1])]);
        assert!(matches!(parse(&bad_vertex), Err(FormatError::Invalid(_))));
        let degenerate = build_test_xmsh(&[v, v], &[(0, 1)], &[], &[(0, &[1, -1])]);
        assert!(matches!(parse(&degenerate), Err(FormatError::Invalid(_))));
    }

    #[test]
    fn mesh_without_lods_is_invalid() {
        let mut d = quad();
        d[0x54..0x58].copy_from_slice(&(-1i32).to_le_bytes());
        assert!(matches!(parse(&d), Err(FormatError::Invalid(_))));
    }

    #[test]
    fn truncated_mesh_is_an_error() {
        let d = quad();
        assert!(parse(&d[..d.len() - 3]).is_err());
    }
}
