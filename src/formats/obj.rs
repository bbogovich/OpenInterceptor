//! # Wavefront OBJ — the model format for mods
//!
//! OBJ is a plain-text 3D format that every modelling tool (Blender, Maya, Wings3D…)
//! can export. It is the format mods use for new ships and stations, so a modder never
//! needs a special tool. We read the subset that matters for game models:
//!
//! ```text
//! mtllib ship.mtl        # material library (colours and texture file names)
//! v  x y z               # vertex position
//! vt u v                 # texture coordinate
//! vn x y z               # normal
//! usemtl hull            # following faces use material "hull"
//! f 1/1/1 2/2/1 3/3/1    # a face: position/uv/normal indices, 1-based
//! ```
//!
//! Faces may also be written `f 1 2 3`, `f 1/2 3/4 5/6` or `f 1//1 2//2 3//3`, and
//! indices may be negative (counting back from the latest vertex).
//!
//! The companion `.mtl` file is parsed by [`parse_mtl`]: we use `newmtl`, `Kd` (diffuse
//! colour) and `map_Kd` (diffuse texture file).

use super::{FResult, FormatError};

/// One face corner: indices (0-based) into the position, uv and normal arrays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Corner {
    pub position: u32,
    pub uv: Option<u32>,
    pub normal: Option<u32>,
}

/// A face: its corners are `mesh.corners[first..first + count]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Face {
    pub material: Option<u16>,
    pub first: u32,
    pub count: u16,
}

/// A parsed OBJ file.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ObjMesh {
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub normals: Vec<[f32; 3]>,
    pub corners: Vec<Corner>,
    pub faces: Vec<Face>,
    /// Material names in order of first use; `Face::material` indexes this.
    pub materials: Vec<String>,
    /// `mtllib` file names referenced by the OBJ.
    pub material_libraries: Vec<String>,
    /// Attachment points: faces in a group or object named `slot_<name>` are not drawn;
    /// each such group becomes a slot at its faces' centre, facing along its first face's
    /// normal: `(name, centre, normal)`.
    pub slots: Vec<(String, [f32; 3], [f32; 3])>,
}

impl ObjMesh {
    /// The corners of one face.
    pub fn face_corners(&self, f: &Face) -> &[Corner] {
        &self.corners[f.first as usize..f.first as usize + f.count as usize]
    }
}

/// A material from an `.mtl` file.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjMaterial {
    pub name: String,
    /// Diffuse colour, 0..1 per channel.
    pub color: [f32; 3],
    /// Diffuse texture file name, relative to the `.mtl` file.
    pub texture: Option<String>,
}

/// Parses floats from the remaining whitespace-separated words of a line.
fn floats<const N: usize>(words: &mut std::str::SplitWhitespace, line: usize) -> FResult<[f32; N]> {
    let mut out = [0.0; N];
    for (i, o) in out.iter_mut().enumerate() {
        let w = words.next();
        // `vt` may legally have only u (v defaults to 0); everything else needs all N.
        *o = match w {
            Some(w) => w.parse().map_err(|_| FormatError::Invalid(format!("line {line}: bad number {w}")))?,
            None if i > 0 && N == 2 => 0.0,
            None => return Err(FormatError::Invalid(format!("line {line}: expected {N} numbers"))),
        };
    }
    Ok(out)
}

/// Resolves a 1-based (or negative, relative) OBJ index against an array length.
fn resolve(index: &str, len: usize, line: usize) -> FResult<u32> {
    let i: i64 = index.parse().map_err(|_| FormatError::Invalid(format!("line {line}: bad index {index}")))?;
    let resolved = if i > 0 { i - 1 } else { len as i64 + i };
    if i == 0 || resolved < 0 || resolved >= len as i64 {
        return Err(FormatError::Invalid(format!("line {line}: index {i} out of range")));
    }
    Ok(resolved as u32)
}

/// Parses OBJ text.
pub fn parse(text: &str) -> FResult<ObjMesh> {
    let mut m = ObjMesh::default();
    let mut material: Option<u16> = None;
    // The current `g`/`o` name; faces of `slot_*` groups are collected separately.
    let mut group = String::new();
    let mut slot_faces: Vec<(String, Vec<u32>)> = Vec::new();
    for (n, raw) in text.lines().enumerate() {
        let line = n + 1;
        let content = raw.split('#').next().unwrap_or("");
        let mut words = content.split_whitespace();
        let Some(tag) = words.next() else { continue };
        match tag {
            "v" => m.positions.push(floats::<3>(&mut words, line)?),
            "vt" => m.uvs.push(floats::<2>(&mut words, line)?),
            "vn" => m.normals.push(floats::<3>(&mut words, line)?),
            "f" => {
                let first = m.corners.len() as u32;
                for w in words {
                    let mut parts = w.split('/');
                    let p = resolve(parts.next().unwrap_or(""), m.positions.len(), line)?;
                    let uv = match parts.next() {
                        Some(s) if !s.is_empty() => Some(resolve(s, m.uvs.len(), line)?),
                        _ => None,
                    };
                    let normal = match parts.next() {
                        Some(s) if !s.is_empty() => Some(resolve(s, m.normals.len(), line)?),
                        _ => None,
                    };
                    m.corners.push(Corner { position: p, uv, normal });
                }
                let count = m.corners.len() as u32 - first;
                if count < 3 {
                    return Err(FormatError::Invalid(format!("line {line}: face needs 3 corners")));
                }
                if let Some(name) = group.strip_prefix("slot_") {
                    let corners: Vec<u32> = m.corners.drain(first as usize..).map(|c| c.position).collect();
                    match slot_faces.iter_mut().find(|(n, _)| n == name) {
                        Some((_, v)) => v.extend(corners),
                        None => slot_faces.push((name.to_string(), corners)),
                    }
                    continue;
                }
                m.faces.push(Face { material, first, count: count as u16 });
            }
            "usemtl" => {
                let name = words.collect::<Vec<_>>().join(" ");
                let idx = match m.materials.iter().position(|x| *x == name) {
                    Some(i) => i,
                    None => {
                        m.materials.push(name);
                        m.materials.len() - 1
                    }
                };
                material = Some(idx as u16);
            }
            "mtllib" => m.material_libraries.push(words.collect::<Vec<_>>().join(" ")),
            "g" | "o" => group = words.collect::<Vec<_>>().join(" ").to_ascii_lowercase(),
            // Smoothing groups do not affect how we draw.
            _ => {}
        }
    }
    for (name, corners) in slot_faces {
        let p = |i: u32| glam::Vec3::from(m.positions[i as usize]);
        let centre = corners.iter().map(|&i| p(i)).sum::<glam::Vec3>() / corners.len().max(1) as f32;
        let normal = if corners.len() >= 3 {
            (p(corners[1]) - p(corners[0])).cross(p(corners[2]) - p(corners[0])).normalize_or(glam::Vec3::NEG_Z)
        } else {
            glam::Vec3::NEG_Z
        };
        m.slots.push((name, centre.into(), normal.into()));
    }
    Ok(m)
}

/// Parses an `.mtl` material library.
pub fn parse_mtl(text: &str) -> Vec<ObjMaterial> {
    let mut out: Vec<ObjMaterial> = Vec::new();
    for raw in text.lines() {
        // A comment is a line starting with '#'. A '#' later on the line is part of the
        // value: `map_Kd original:CockpitA.dll#12000` names an original resource.
        let content = raw.trim();
        if content.starts_with('#') {
            continue;
        }
        let Some((tag, rest)) = content.split_once(char::is_whitespace) else { continue };
        let rest = rest.trim();
        match tag {
            "newmtl" => out.push(ObjMaterial { name: rest.to_string(), color: [1.0; 3], texture: None }),
            "Kd" => {
                if let Some(m) = out.last_mut() {
                    let v: Vec<f32> = rest.split_whitespace().filter_map(|x| x.parse().ok()).collect();
                    if v.len() >= 3 {
                        m.color = [v[0], v[1], v[2]];
                    }
                }
            }
            "map_Kd" => {
                if let Some(m) = out.last_mut() {
                    // Options like `-s 1 1 1` may precede the file name; it is the last word.
                    m.texture = rest.split_whitespace().last().map(str::to_string);
                }
            }
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_groups_become_attachment_points() {
        let m = parse("v 0 0 0\nv 1 0 0\nv 0 1 0\nv 4 0 -2\nv 6 0 -2\nv 4 2 -2\nf 1 2 3\ng slot_turret\nf 4 5 6\ng hull\nf 1 2 3\n").unwrap();
        assert_eq!(m.faces.len(), 2, "the slot face is not drawn");
        assert_eq!(m.slots.len(), 1);
        let (name, c, n) = &m.slots[0];
        assert_eq!(name, "turret");
        assert!((c[0] - 14.0 / 3.0).abs() < 1e-5 && (c[2] + 2.0).abs() < 1e-6);
        assert_eq!(*n, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn parses_positions_uvs_normals_and_faces() {
        let text = "mtllib ship.mtl\n# comment\nv 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nvt 0 0\nvt 1\nvn 0 0 1\n\
                    usemtl hull\nf 1/1/1 2/2/1 3/1/1 4/2/1\nusemtl glass\nf 1//1 2//1 3//1\nusemtl hull\nf -1 -2 -3\n";
        let m = parse(text).unwrap();
        assert_eq!(m.positions.len(), 4);
        assert_eq!(m.uvs[1], [1.0, 0.0], "missing v defaults to 0");
        assert_eq!(m.faces.len(), 3);
        assert_eq!(m.materials, vec!["hull", "glass"]);
        assert_eq!(m.material_libraries, vec!["ship.mtl"]);
        assert_eq!(m.faces[0].count, 4);
        assert_eq!(m.faces[2].material, Some(0));
        let c = m.face_corners(&m.faces[1]);
        assert_eq!(c[0], Corner { position: 0, uv: None, normal: Some(0) });
        let neg = m.face_corners(&m.faces[2]);
        assert_eq!(neg[0].position, 3, "-1 is the latest vertex");
    }

    #[test]
    fn rejects_bad_indices_and_short_faces() {
        assert!(parse("v 0 0 0\nf 1 2 3\n").is_err());
        assert!(parse("v 0 0 0\nv 0 0 0\nf 1 2\n").is_err());
        assert!(parse("v 0 0 0\nf 0 1 1\n").is_err());
        assert!(parse("v 0 x 0\n").is_err());
        assert!(parse("v 0 0\n").is_err());
        assert!(parse("v 0 0 0\nf a 1 1\n").is_err());
    }

    #[test]
    fn parses_materials() {
        let mats = parse_mtl("newmtl hull\nKd 0.5 0.25 1\nmap_Kd -s 1 1 1 hull.png\nnewmtl glass\nKd 1\n");
        assert_eq!(mats.len(), 2);
        assert_eq!(mats[0].color, [0.5, 0.25, 1.0]);
        assert_eq!(mats[0].texture.as_deref(), Some("hull.png"));
        assert_eq!(mats[1].color, [1.0; 3], "short Kd is ignored");
        assert_eq!(mats[1].texture, None);
        let m = parse_mtl("# a comment\nnewmtl pit\nmap_Kd original:CockpitA.dll#12000\n");
        assert_eq!(m[0].texture.as_deref(), Some("original:CockpitA.dll#12000"), "'#' inside a value is kept");
    }
}
