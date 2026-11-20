//! # Battle environments and terrains (Stage 3.3–3.5)
//!
//! Where a battle happens: deep space, in orbit over a planet, inside a small "pocket",
//! or in the atmosphere above a surface. An environment is pack data
//! (`environments: {<id>: …}`); a mission or scenario names one with `environment: <id>`
//! (default: open space). R-MSN-008, R-FLT-011, R-FLT-012.
//!
//! ```yaml
//! environments:
//!   orbit:
//!     backdrop: {planet: {color: [0.3, 0.5, 0.9], size: 35, direction: [0.2, -0.6, -1]}}
//!   moon_surface:
//!     physics: atmosphere
//!     gravity: 3.7                       # m/s², downwards
//!     air_density: 0.4                   # 0 = vacuum (no drag, no lift)
//!     ceiling: 9km                       # climbing above it leaves the battle (to orbit)
//!     terrain: craters
//!     sky: {top: [0.05, 0.05, 0.12], horizon: [0.45, 0.35, 0.40], fog: [0.40, 0.33, 0.38], fog_distance: 12km, stars: 0.4}
//! terrains:
//!   craters:
//!     generate: {kind: craters, seed: 7}  # or heightmap: textures/terrain/hm.png
//!     size: 30km
//!     height: [0, 1800]                  # lowest and highest ground, metres
//!     colors: {low: [0.30, 0.27, 0.25], high: [0.62, 0.58, 0.52], slope: [0.22, 0.20, 0.19]}
//!     detail: textures/terrain_regolith.png   # optional: tiles over the ground up close
//!     detail_scale: 40m                       # one repeat of the texture covers 40 m
//!     slope_detail: textures/terrain_rock.png # optional: a second texture on steep ground
//!     scatter:                                # optional: decoration placed over the ground
//!       - {model: models/scatter_boulder.obj, count: 300, scale: [0.6, 1.8], max_slope: 25deg, align: 1}
//! ```
//!
//! A **heightfield** is a square grid of heights ([`Heightfield`]): the simulation asks it
//! for the ground height under a craft, and the renderer turns it into a mesh. Generated
//! terrains are deterministic for their seed.
//!
//! **Scatter** ([`ScatterDef`], [`scatter_placements`]) is decoration: rocks, trees or
//! buildings placed over the field from the battle's seed, standing on the ground. It is
//! drawn only — craft fly through it. Anything a craft should hit or shoot is a ground
//! installation: a real object in the scenario, with a collision shape and hit points.
//! Placement is pure data and lives here, below the renderer, so it is unit-tested and the
//! simulation could use it later without depending on rendering.

use super::schema::{f, Get, Ty, Unit};
use glam::{Quat, Vec3};

/// How craft fly in an environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Physics {
    /// No air, no gravity: the campaign's flight model unchanged.
    #[default]
    Space,
    /// Drag, gravity, lift and a ground (R-FLT-011).
    Atmosphere,
}

/// A sky drawn instead of black space (atmospheric environments): a gradient from the
/// zenith to the horizon, fog that fades distant things into the horizon colour, and how
/// much of the starfield shows through (0 = none).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sky {
    pub top: [f32; 3],
    pub horizon: [f32; 3],
    pub fog: [f32; 3],
    pub fog_distance: f32,
    pub stars: f32,
}

/// A planet hanging in the sky (orbit): its colour, angular size in degrees and the
/// direction it lies in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Planet {
    pub color: [f32; 3],
    pub size: f32,
    pub direction: Vec3,
}

/// A nebula around the battle (`backdrop: {nebula: …}`): soft clouds of two colours
/// on the sky, with the stars dimmed behind them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Nebula {
    pub colors: [[f32; 3]; 2],
    /// How thick the clouds are, 0..1 (also how much they hide the stars).
    pub density: f32,
    pub seed: u64,
}

impl Nebula {
    /// The cloud colour and opacity in direction `dir` (a unit vector): two octaves of
    /// smooth 3D value noise, one for how much cloud there is, one for its colour.
    /// Deterministic for the seed.
    pub fn color(&self, dir: Vec3) -> [f32; 4] {
        // Summed octaves crowd around 0.5: stretch them back out to 0..1 first.
        let stretch = |v: f32| ((v - 0.5) * 2.8 + 0.5).clamp(0.0, 1.0);
        let amount = stretch(fbm(dir * 2.5, self.seed));
        let tint = stretch(fbm(dir * 1.7 + Vec3::splat(31.0), self.seed ^ 0x9E37));
        // Clouds rather than haze: thin areas fall to nothing.
        let a = ((amount - 0.3) / 0.7).clamp(0.0, 1.0).powf(1.2) * self.density;
        let [c0, c1] = self.colors;
        let mix = |i: usize| c0[i] + (c1[i] - c0[i]) * tint;
        [mix(0), mix(1), mix(2), a]
    }
}

/// Smooth value noise in 3D, 0..1, summed over 4 octaves.
fn fbm(p: Vec3, seed: u64) -> f32 {
    // A lattice value from a hash of its integer coordinates and the seed.
    let hash = |x: i32, y: i32, z: i32| {
        let mut h = (x as u64).wrapping_mul(0x8DA6_B343)
            ^ (y as u64).wrapping_mul(0xD816_3841)
            ^ (z as u64).wrapping_mul(0xCB1A_B31F)
            ^ seed.wrapping_mul(0x9E37_79B9);
        h ^= h >> 13;
        h = h.wrapping_mul(0x5bd1_e995);
        h ^= h >> 15;
        (h & 0xFFFF) as f32 / 65535.0
    };
    let noise = |p: Vec3| {
        let f = p.floor();
        let t = p - f;
        let s = t * t * (Vec3::splat(3.0) - 2.0 * t); // smoothstep: no creases
        let (x, y, z) = (f.x as i32, f.y as i32, f.z as i32);
        let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
        let face = |dz: i32| {
            let a = lerp(hash(x, y, z + dz), hash(x + 1, y, z + dz), s.x);
            let b = lerp(hash(x, y + 1, z + dz), hash(x + 1, y + 1, z + dz), s.x);
            lerp(a, b, s.y)
        };
        lerp(face(0), face(1), s.z)
    };
    let (mut sum, mut amp, mut freq, mut norm) = (0.0, 1.0, 1.0, 0.0);
    for _ in 0..4 {
        sum += noise(p * freq) * amp;
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    sum / norm
}

/// One battle environment.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EnvironmentDef {
    pub id: String,
    pub physics: Physics,
    /// Overrides `flight.arena_radius` (a pocket is a small arena).
    pub arena: Option<f32>,
    pub gravity: f32,
    pub air_density: f32,
    /// Height (metres) above which a craft leaves the battle, as by hyperspace.
    pub ceiling: Option<f32>,
    /// The terrain id under an atmospheric battle.
    pub terrain: Option<String>,
    pub sky: Option<Sky>,
    pub planet: Option<Planet>,
    pub nebula: Option<Nebula>,
}

/// How a terrain's heights are made.
#[derive(Debug, Clone, PartialEq)]
pub enum TerrainSource {
    /// Generated: `craters` (a cratered plain) or `hills` (rolling noise), from a seed.
    Generate { kind: String, seed: u64 },
    /// A greyscale image: black = lowest, white = highest.
    Heightmap(String),
}

/// One terrain definition.
#[derive(Debug, Clone, PartialEq)]
pub struct TerrainDef {
    pub id: String,
    pub source: TerrainSource,
    /// Side length of the square terrain, metres (centred on the origin).
    pub size: f32,
    /// Lowest and highest ground, metres.
    pub height: [f32; 2],
    /// Colours: low ground, high ground, and steep slopes.
    pub colors: [[f32; 3]; 3],
    /// Grid points per side (the mesh and the height lookup).
    pub resolution: usize,
    pub pack: usize,
    /// The heights, built once: generated ones while the rules are read, heightmaps when
    /// the content loads (they need the pack's files). Shared, not copied, per battle.
    pub field: Option<std::sync::Arc<Heightfield>>,
    /// A tiling texture that modulates the ground colours up close (`detail`), and the
    /// optional one used on steep ground instead (`slope_detail`). `None`: vertex colours
    /// only, exactly as without the keys.
    pub detail: Option<String>,
    pub slope_detail: Option<String>,
    /// Metres of ground covered by one repeat of the detail texture.
    pub detail_scale: f32,
    /// Decoration placed over the ground.
    pub scatter: Vec<ScatterDef>,
}

/// One kind of scattered decoration (`terrains.<id>.scatter[i]`): a model placed up to
/// `count` times over the field where the ground suits it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScatterDef {
    /// The model (an asset reference, like a ship's `model`).
    pub model: String,
    /// How many to place (fewer if the field has too few suitable spots).
    pub count: usize,
    /// Smallest and largest size, as multiples of the model's own size.
    pub scale: [f32; 2],
    /// The steepest ground a piece may stand on, in degrees from level.
    pub max_slope: f32,
    /// Only on ground between these heights (metres).
    pub height: [f32; 2],
    /// How far each piece leans with the ground: 0 = upright (trees, buildings), 1 =
    /// square to the slope (rocks).
    pub align: f32,
    /// Mixed with the battle's seed, so two entries with the same model differ.
    pub seed: u64,
    /// Not drawn farther than this from the camera (metres).
    pub draw_distance: f32,
}

/// The most pieces one scatter entry may ask for (a guard against a typo such as
/// `count: 2000000`, which would stall loading and fill the GPU).
pub const MAX_SCATTER_COUNT: usize = 20_000;

/// One placed piece of scatter, in the field's own space (inside the field's square).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: f32,
}

impl ScatterDef {
    /// Reads one `scatter` list item. `index` numbers it for messages and the default
    /// seed; `height` is the terrain's own range (the default for the item's).
    fn parse(index: usize, g: Get, height: [f32; 2]) -> Result<ScatterDef, String> {
        let at = format!("scatter[{index}]");
        let model = g.str("model").ok_or(format!("{at}: needs a model"))?.to_string();
        let count = g.num("count").unwrap_or(100.0);
        if !(1.0..=MAX_SCATTER_COUNT as f32).contains(&count) {
            return Err(format!("{at}: count {count} is not between 1 and {MAX_SCATTER_COUNT}"));
        }
        // `scale: 2` (always that size) or `scale: [0.5, 2]` (a range).
        let scale = match (g.num("scale"), g.nums("scale")) {
            (Some(s), _) => [s, s],
            (None, Some(v)) if v.len() == 2 => [v[0], v[1]],
            (None, None) if g.node("scale").is_none() => [1.0, 1.0],
            _ => return Err(format!("{at}: scale is a number or [smallest, largest]")),
        };
        if scale[0] <= 0.0 || scale[1] < scale[0] {
            return Err(format!("{at}: scale [{}, {}] must be positive, smallest first", scale[0], scale[1]));
        }
        let max_slope = g.qty("max_slope", Unit::Angle).unwrap_or(30.0);
        if !(0.0..=90.0).contains(&max_slope) {
            return Err(format!("{at}: max_slope {max_slope} is not between 0 and 90 degrees"));
        }
        let h = match g.nums("height") {
            Some(v) if v.len() == 2 => [v[0], v[1]],
            Some(_) => return Err(format!("{at}: height is [lowest, highest]")),
            None => height,
        };
        if h[1] < h[0] {
            return Err(format!("{at}: height [{}, {}] has the highest below the lowest", h[0], h[1]));
        }
        let align = g.num("align").unwrap_or(0.0);
        if !(0.0..=1.0).contains(&align) {
            return Err(format!("{at}: align {align} is not between 0 and 1"));
        }
        let draw_distance = g.qty("draw_distance", Unit::Distance).unwrap_or(4000.0);
        if draw_distance <= 0.0 {
            return Err(format!("{at}: draw_distance must be more than 0"));
        }
        Ok(ScatterDef {
            model,
            count: count as usize,
            scale,
            max_slope,
            height: h,
            align,
            seed: g.num("seed").map(|s| s as u64).unwrap_or(index as u64 + 1),
            draw_distance,
        })
    }
}

/// Places one scatter entry over `field`, deterministically for `battle_seed`: the same
/// battle always has the same rocks in the same places.
///
/// Spots are drawn at random over the field's square (*rejection sampling*): a spot whose
/// ground is too steep or outside the height range is thrown away and another drawn, up
/// to 30 tries per piece, so a field with few suitable spots gets fewer pieces rather
/// than a loop that never ends. Each piece gets a random turn about the vertical, a size
/// in the scale range, and leans towards the ground's normal by `align`.
pub fn scatter_placements(def: &ScatterDef, field: &Heightfield, battle_seed: u64) -> Vec<Placement> {
    // xorshift64: tiny, fast and fully determined by its seed. Its state must never be
    // 0 (it would stay 0 for ever), hence `| 1`.
    let mut rng = (battle_seed ^ def.seed.wrapping_mul(0x9E37_79B9_7F4A_7C15)).wrapping_mul(0xBF58_476D_1CE4_E5B9) | 1;
    let mut next = move || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        // The top 24 bits as a float in 0..1 (24 bits: exactly what an f32 holds).
        (rng >> 40) as f32 / (1u64 << 24) as f32
    };
    // The slope test compares the normal's up component with the cosine of the steepest
    // allowed tilt: a level normal has y = 1, a wall y = 0.
    let min_up = def.max_slope.to_radians().cos();
    let half = field.size * 0.5;
    let mut out = Vec::with_capacity(def.count);
    for _ in 0..def.count * 30 {
        if out.len() == def.count {
            break;
        }
        // Always draw all four numbers, so every try uses the same amount of the
        // sequence whether or not it is rejected.
        let (u, v, turn, size) = (next(), next(), next(), next());
        let (x, z) = (-half + u * field.size, -half + v * field.size);
        let h = field.height_at(x, z);
        let n = field.normal_at(x, z);
        if n.y < min_up - 1e-6 || h < def.height[0] || h > def.height[1] {
            continue;
        }
        let up = Vec3::Y.lerp(n, def.align).normalize();
        // Quaternions compose right to left: `lean * spin` spins about the vertical
        // first, then leans the spun piece over to `up`.
        let rotation = Quat::from_rotation_arc(Vec3::Y, up) * Quat::from_rotation_y(turn * std::f32::consts::TAU);
        let scale = def.scale[0] + (def.scale[1] - def.scale[0]) * size;
        out.push(Placement { position: Vec3::new(x, h, z), rotation, scale });
    }
    out
}

fn rgb(g: Get, k: &str, d: [f32; 3]) -> [f32; 3] {
    match g.nums(k) {
        Some(v) if v.len() == 3 => [v[0], v[1], v[2]],
        _ => d,
    }
}

impl EnvironmentDef {
    /// Reads one `environments.<id>` entry.
    pub fn parse(id: &str, g: Get) -> Result<EnvironmentDef, String> {
        let physics = match g.str("physics").map(str::to_ascii_lowercase).as_deref() {
            None | Some("space") => Physics::Space,
            Some("atmosphere") => Physics::Atmosphere,
            Some(o) => return Err(format!("physics: '{o}' is not space or atmosphere")),
        };
        let sky = g.node("sky").map(Get).map(|s| Sky {
            top: rgb(s, "top", [0.05, 0.08, 0.2]),
            horizon: rgb(s, "horizon", [0.5, 0.6, 0.75]),
            fog: rgb(s, "fog", rgb(s, "horizon", [0.5, 0.6, 0.75])),
            fog_distance: s.qty("fog_distance", Unit::Distance).unwrap_or(15_000.0).max(100.0),
            stars: s.num("stars").unwrap_or(0.0).clamp(0.0, 1.0),
        });
        let planet = g.node("backdrop").and_then(|b| Get(b).node("planet")).map(Get).map(|p| {
            let d = p.nums("direction").filter(|v| v.len() == 3).map(|v| Vec3::new(v[0], v[1], v[2]));
            Planet {
                color: rgb(p, "color", [0.3, 0.5, 0.9]),
                size: p.num("size").unwrap_or(30.0).clamp(1.0, 170.0),
                direction: d.and_then(|d| d.try_normalize()).unwrap_or(Vec3::new(0.0, -0.5, -1.0).normalize()),
            }
        });
        let nebula = g.node("backdrop").and_then(|b| Get(b).node("nebula")).map(Get).map(|n| Nebula {
            colors: [rgb(n, "color", [0.55, 0.2, 0.6]), rgb(n, "second_color", [0.2, 0.35, 0.75])],
            density: n.num("density").unwrap_or(0.8).clamp(0.0, 1.0),
            seed: n.num("seed").unwrap_or(1.0) as u64,
        });
        let env = EnvironmentDef {
            nebula,
            id: id.to_string(),
            physics,
            arena: g.qty("arena", Unit::Distance),
            gravity: g.num("gravity").unwrap_or(if physics == Physics::Atmosphere { 9.8 } else { 0.0 }).max(0.0),
            air_density: g
                .num("air_density")
                .unwrap_or(if physics == Physics::Atmosphere { 1.0 } else { 0.0 })
                .max(0.0),
            ceiling: g.qty("ceiling", Unit::Distance),
            terrain: g.str("terrain").map(str::to_string),
            sky,
            planet,
        };
        if env.physics == Physics::Atmosphere && env.terrain.is_none() {
            return Err("an atmosphere needs a terrain".into());
        }
        Ok(env)
    }
}

impl TerrainDef {
    /// Reads one `terrains.<id>` entry.
    pub fn parse(id: &str, g: Get, pack: usize) -> Result<TerrainDef, String> {
        let source = match (g.node("generate").map(Get), g.str("heightmap")) {
            (Some(gen), None) => {
                let kind = gen.str("kind").unwrap_or("hills").to_ascii_lowercase();
                if kind != "hills" && kind != "craters" {
                    return Err(format!("generate.kind: '{kind}' is not hills or craters"));
                }
                TerrainSource::Generate { kind, seed: gen.num("seed").unwrap_or(1.0) as u64 }
            }
            (None, Some(h)) => TerrainSource::Heightmap(h.to_string()),
            (Some(_), Some(_)) => return Err("give generate or heightmap, not both".into()),
            (None, None) => return Err("needs generate or heightmap".into()),
        };
        let h = g.nums("height").filter(|v| v.len() == 2).unwrap_or(vec![0.0, 1500.0]);
        let colors = g.node("colors").map(Get);
        let size = g.qty("size", Unit::Distance).unwrap_or(30_000.0).max(1000.0);
        let resolution = g.num("resolution").map(|r| r.clamp(17.0, 257.0) as usize).unwrap_or(129);
        let height = [h[0], h[1].max(h[0] + 1.0)];
        let field = match &source {
            TerrainSource::Generate { kind, seed } => {
                Some(std::sync::Arc::new(Heightfield::generate(kind, *seed, resolution, size, height)))
            }
            TerrainSource::Heightmap(_) => None,
        };
        let c = |k: &str, d: [f32; 3]| colors.map(|c| rgb(c, k, d)).unwrap_or(d);
        let detail = g.str("detail").map(str::to_string);
        let slope_detail = g.str("slope_detail").map(str::to_string);
        if slope_detail.is_some() && detail.is_none() {
            return Err("slope_detail needs a detail texture too".into());
        }
        let detail_scale = match g.node("detail_scale") {
            None => 50.0,
            Some(_) => g
                .qty("detail_scale", Unit::Distance)
                .filter(|s| *s > 0.0)
                .ok_or("detail_scale must be a distance more than 0 (e.g. 40m)")?,
        };
        let mut scatter = Vec::new();
        if let Some(items) = g.node("scatter") {
            let items = items.as_seq().ok_or("scatter is a list of {model, count, …}")?;
            for (i, item) in items.iter().enumerate() {
                scatter.push(ScatterDef::parse(i, Get(item), height)?);
            }
        }
        Ok(TerrainDef {
            detail,
            slope_detail,
            detail_scale,
            scatter,
            id: id.to_string(),
            source,
            size,
            height,
            colors: [c("low", [0.3, 0.28, 0.25]), c("high", [0.62, 0.58, 0.52]), c("slope", [0.22, 0.2, 0.19])],
            resolution,
            pack,
            field,
        })
    }
}

/// A square grid of ground heights centred on the origin (x and z from −size/2 to
/// +size/2). Outside the grid the edge heights continue.
#[derive(Debug, Clone, PartialEq)]
pub struct Heightfield {
    /// Points per side.
    pub n: usize,
    pub size: f32,
    /// Row-major: `heights[row * n + col]`, row along z, column along x.
    pub heights: Vec<f32>,
}

impl Heightfield {
    /// The world x (or z) of grid column (or row) `i`.
    pub fn coord(&self, i: usize) -> f32 {
        -self.size * 0.5 + self.size * i as f32 / (self.n - 1) as f32
    }

    /// Folds a coordinate anywhere on the infinite ground back into the field's square.
    /// The ground goes on to the horizon as **mirror-image tiles** of the field: the tile
    /// next door is the field reflected across the shared edge, so heights meet exactly at
    /// every edge (no cliffs, no seams). Tile `i` covers `[(i - ½)·size, (i + ½)·size]`;
    /// odd tiles are reflected.
    pub fn fold(&self, x: f32) -> f32 {
        let s = self.size;
        let u = x + s * 0.5; // 0..s inside the field itself
        let i = (u / s).floor();
        let mut local = u - i * s;
        if (i as i64).rem_euclid(2) == 1 {
            local = s - local;
        }
        local - s * 0.5
    }

    /// The ground height under `(x, z)` (anywhere: see [`Heightfield::fold`]), bilinearly
    /// interpolated.
    pub fn height_at(&self, x: f32, z: f32) -> f32 {
        let (x, z) = (self.fold(x), self.fold(z));
        let cell = self.size / (self.n - 1) as f32;
        let fx = ((x + self.size * 0.5) / cell).clamp(0.0, (self.n - 1) as f32);
        let fz = ((z + self.size * 0.5) / cell).clamp(0.0, (self.n - 1) as f32);
        let (c0, r0) = (fx.floor() as usize, fz.floor() as usize);
        let (c1, r1) = ((c0 + 1).min(self.n - 1), (r0 + 1).min(self.n - 1));
        let (tx, tz) = (fx - c0 as f32, fz - r0 as f32);
        let h = |r: usize, c: usize| self.heights[r * self.n + c];
        let a = h(r0, c0) + (h(r0, c1) - h(r0, c0)) * tx;
        let b = h(r1, c0) + (h(r1, c1) - h(r1, c0)) * tx;
        a + (b - a) * tz
    }

    /// The surface normal at `(x, z)`, from the slope of the heights around it.
    pub fn normal_at(&self, x: f32, z: f32) -> Vec3 {
        let e = self.size / (self.n - 1) as f32;
        let dx = self.height_at(x + e, z) - self.height_at(x - e, z);
        let dz = self.height_at(x, z + e) - self.height_at(x, z - e);
        Vec3::new(-dx, 2.0 * e, -dz).normalize()
    }

    /// Generates a heightfield: `hills` is smooth value noise in several octaves;
    /// `craters` is a gently rolling plain with bowl-shaped craters and raised rims.
    /// Heights span `height[0]..height[1]`. Deterministic for `seed`.
    pub fn generate(kind: &str, seed: u64, n: usize, size: f32, height: [f32; 2]) -> Heightfield {
        let mut rng = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        let mut next = move || {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            (rng >> 40) as f32 / (1u64 << 24) as f32
        };
        // Value noise: random values on a coarse lattice, smoothly interpolated.
        let lattice: Vec<f32> = (0..65 * 65).map(|_| next()).collect();
        let noise = |u: f32, v: f32| {
            let (x, y) = (u.rem_euclid(64.0), v.rem_euclid(64.0));
            let (x0, y0) = (x.floor() as usize, y.floor() as usize);
            let (tx, ty) = (x - x0 as f32, y - y0 as f32);
            // Smoothstep weights: no creases at the lattice lines.
            let (sx, sy) = (tx * tx * (3.0 - 2.0 * tx), ty * ty * (3.0 - 2.0 * ty));
            let l = |i: usize, j: usize| lattice[(j % 65) * 65 + (i % 65)];
            let a = l(x0, y0) + (l(x0 + 1, y0) - l(x0, y0)) * sx;
            let b = l(x0, y0 + 1) + (l(x0 + 1, y0 + 1) - l(x0, y0 + 1)) * sx;
            a + (b - a) * sy
        };
        let mut raw = vec![0.0f32; n * n];
        for r in 0..n {
            for c in 0..n {
                let (u, v) = (c as f32 / (n - 1) as f32, r as f32 / (n - 1) as f32);
                let mut h = 0.0;
                let mut amp = 1.0;
                for octave in 0..5 {
                    let freq = 4.0 * (1 << octave) as f32;
                    h += amp * noise(u * freq, v * freq);
                    amp *= 0.5;
                }
                raw[r * n + c] = if kind == "craters" { h * 0.35 } else { h };
            }
        }
        if kind == "craters" {
            // Craters: a bowl below the surroundings with a raised rim, in sizes from
            // large to small (more small ones).
            for k in 0..60 {
                let (cx, cy) = (next(), next());
                let rad = 0.02 + 0.12 * next() * next() + if k < 4 { 0.08 } else { 0.0 };
                let depth = rad * 3.0;
                for r in 0..n {
                    for c in 0..n {
                        let (u, v) = (c as f32 / (n - 1) as f32, r as f32 / (n - 1) as f32);
                        let d = ((u - cx).powi(2) + (v - cy).powi(2)).sqrt() / rad;
                        if d < 1.4 {
                            let bowl = if d < 1.0 { -(1.0 - d * d) } else { 0.0 };
                            let rim = (-((d - 1.0) / 0.18).powi(2)).exp() * 0.35;
                            raw[r * n + c] += depth * (bowl + rim);
                        }
                    }
                }
            }
        }
        Heightfield::scaled(n, size, &raw, height)
    }

    /// A heightfield from raw values, rescaled so they span `height`.
    pub fn scaled(n: usize, size: f32, raw: &[f32], height: [f32; 2]) -> Heightfield {
        let (lo, hi) = raw.iter().fold((f32::MAX, f32::MIN), |(a, b), &h| (a.min(h), b.max(h)));
        let span = (hi - lo).max(1e-6);
        let heights = raw.iter().map(|h| height[0] + (h - lo) / span * (height[1] - height[0])).collect();
        Heightfield { n, size, heights }
    }

    /// From a greyscale image (any size, resampled to `n` per side; the red channel).
    pub fn from_image(
        width: u32,
        image_height: u32,
        rgba: &[u8],
        n: usize,
        size: f32,
        height: [f32; 2],
    ) -> Heightfield {
        let (w, h) = (width as usize, image_height as usize);
        let mut raw = vec![0.0f32; n * n];
        for r in 0..n {
            for c in 0..n {
                let x = c * (w - 1) / (n - 1).max(1);
                let y = r * (h - 1) / (n - 1).max(1);
                raw[r * n + c] = rgba[(y * w + x) * 4] as f32;
            }
        }
        Heightfield::scaled(n, size, &raw, height)
    }
}

/// The schema of `environments.<id>`.
pub fn environment_schema() -> Ty {
    Ty::Def(vec![
        f("physics", Ty::Enum(&["space", "atmosphere"]), "space (the flight model unchanged) or atmosphere"),
        f("arena", Ty::Qty(Unit::Distance), "arena radius for battles here (a pocket is small)"),
        f("gravity", Ty::Num, "m/s² downwards (atmosphere; default 9.8)"),
        f("air_density", Ty::Num, "scales drag and lift; 0 = vacuum (atmosphere; default 1)"),
        f("ceiling", Ty::Qty(Unit::Distance), "climbing above this leaves the battle (to orbit)"),
        f("terrain", Ty::Name, "the terrain under an atmospheric battle"),
        f(
            "sky",
            Ty::Struct(vec![
                f("top", Ty::Numbers(3), "colour at the zenith"),
                f("horizon", Ty::Numbers(3), "colour at the horizon"),
                f("fog", Ty::Numbers(3), "fog colour (default: the horizon)"),
                f("fog_distance", Ty::Qty(Unit::Distance), "distance at which things fade into the fog"),
                f("stars", Ty::Num, "how much of the starfield shows, 0..1"),
            ]),
            "a sky instead of black space",
        ),
        f(
            "backdrop",
            Ty::Struct(vec![
                f(
                    "planet",
                    Ty::Struct(vec![
                        f("color", Ty::Numbers(3), "the planet's colour"),
                        f("size", Ty::Num, "angular size, degrees"),
                        f("direction", Ty::Numbers(3), "the direction it lies in"),
                    ]),
                    "a planet in the sky (orbit)",
                ),
                f(
                    "nebula",
                    Ty::Struct(vec![
                        f("color", Ty::Numbers(3), "the clouds' main colour"),
                        f("second_color", Ty::Numbers(3), "the colour they shade into"),
                        f("density", Ty::Num, "how thick, 0..1 (also dims the stars)"),
                        f("seed", Ty::Num, "a different cloud pattern"),
                    ]),
                    "nebula clouds all around (a pocket)",
                ),
            ]),
            "what hangs in the sky",
        ),
    ])
}

/// The schema of `terrains.<id>`.
pub fn terrain_schema() -> Ty {
    Ty::Def(vec![
        f(
            "generate",
            Ty::Struct(vec![
                f("kind", Ty::Enum(&["hills", "craters"]), "rolling hills or a cratered plain"),
                f("seed", Ty::Num, "the random seed (the same seed, the same terrain)"),
            ]),
            "generate the heights",
        ),
        f("heightmap", Ty::Asset, "or: a greyscale image (black low, white high)"),
        f("size", Ty::Qty(Unit::Distance), "side length of the square terrain"),
        f("height", Ty::Numbers(2), "[lowest, highest] ground, metres"),
        f(
            "colors",
            Ty::Struct(vec![
                f("low", Ty::Numbers(3), "low ground"),
                f("high", Ty::Numbers(3), "high ground"),
                f("slope", Ty::Numbers(3), "steep slopes"),
            ]),
            "ground colours",
        ),
        f("resolution", Ty::Num, "grid points per side (17..257, default 129)"),
        f("detail", Ty::Asset, "a tiling texture multiplying the ground colours up close (white = unchanged)"),
        f("detail_scale", Ty::Qty(Unit::Distance), "metres covered by one repeat of the detail texture (default 50m)"),
        f("slope_detail", Ty::Asset, "a second detail texture for steep ground (needs detail)"),
        f(
            "scatter",
            Ty::List(Box::new(Ty::Struct(vec![
                f("model", Ty::Asset, "the model to place (OBJ or original:)"),
                f("count", Ty::Num, "how many (1..20000, default 100; fewer if few spots suit)"),
                f(
                    "scale",
                    Ty::Either(Box::new(Ty::Num), Box::new(Ty::Numbers(2))),
                    "size: a number or [smallest, largest] (default 1)",
                ),
                f("max_slope", Ty::Qty(Unit::Angle), "steepest ground to stand on, degrees (default 30)"),
                f("height", Ty::Numbers(2), "only on ground between [lowest, highest] metres (default: all)"),
                f("align", Ty::Num, "lean with the ground: 0 upright .. 1 square to the slope (default 0)"),
                f("seed", Ty::Num, "mixed with the battle's seed (default: the item's number)"),
                f("draw_distance", Ty::Qty(Unit::Distance), "not drawn farther away than this (default 4km)"),
            ]))),
            "decoration placed over the ground (drawn only: craft fly through it)",
        ),
    ])
}

#[cfg(test)]
mod nebula_tests {
    use super::*;

    #[test]
    fn nebula_clouds_vary_stay_in_range_and_repeat_for_a_seed() {
        let n = Nebula { colors: [[1.0, 0.0, 0.0], [0.0, 0.0, 1.0]], density: 0.8, seed: 3 };
        let mut alphas = Vec::new();
        for k in 0..200 {
            let a = k as f32 * 0.37;
            let d = Vec3::new(a.cos() * (a * 0.5).cos(), (a * 0.5).sin(), a.sin() * (a * 0.5).cos()).normalize();
            let c = n.color(d);
            assert!(c.iter().all(|x| (0.0..=1.0).contains(x)), "{c:?}");
            assert!(c[3] <= 0.8 + 1e-6, "no thicker than the density");
            assert_eq!(c, n.color(d), "deterministic");
            alphas.push(c[3]);
        }
        assert!(alphas.iter().any(|a| *a < 0.05) && alphas.iter().any(|a| *a > 0.3), "clouds and gaps");
        assert_ne!(n.color(Vec3::X), Nebula { seed: 4, ..n }.color(Vec3::X));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::yaml;

    #[test]
    fn environments_and_terrains_parse_and_report_mistakes() {
        let doc = yaml::parse(
            "physics: atmosphere\ngravity: 3.7\nceiling: 9km\nterrain: craters\n\
             sky: {top: [0, 0, 0.1], horizon: [0.4, 0.3, 0.3], fog_distance: 12km}\n",
        )
        .unwrap();
        let e = EnvironmentDef::parse("moon", Get(&doc)).unwrap();
        assert_eq!((e.physics, e.gravity, e.ceiling, e.air_density), (Physics::Atmosphere, 3.7, Some(9000.0), 1.0));
        assert_eq!(e.sky.unwrap().fog, [0.4, 0.3, 0.3], "fog defaults to the horizon");
        let orbit = yaml::parse("backdrop: {planet: {color: [0.2, 0.4, 0.8], size: 40}}\n").unwrap();
        let o = EnvironmentDef::parse("orbit", Get(&orbit)).unwrap();
        assert_eq!((o.physics, o.gravity), (Physics::Space, 0.0));
        assert_eq!(o.planet.unwrap().size, 40.0);
        let bad = yaml::parse("physics: atmosphere\n").unwrap();
        assert!(EnvironmentDef::parse("x", Get(&bad)).unwrap_err().contains("terrain"));
        let t = yaml::parse("generate: {kind: craters, seed: 7}\nsize: 20km\nheight: [0, 1200]\n").unwrap();
        let t = TerrainDef::parse("craters", Get(&t), 0).unwrap();
        assert_eq!((t.size, t.height), (20000.0, [0.0, 1200.0]));
        let both = yaml::parse("generate: {kind: hills}\nheightmap: a.png\n").unwrap();
        assert!(TerrainDef::parse("x", Get(&both), 0).is_err());
        // No detail or scatter keys: vertex colours only, nothing scattered.
        assert_eq!((t.detail.as_deref(), t.slope_detail.as_deref(), t.detail_scale), (None, None, 50.0));
        assert!(t.scatter.is_empty());
    }

    /// Parses a terrain with `extra` YAML lines after a valid base.
    fn terrain(extra: &str) -> Result<TerrainDef, String> {
        let doc = yaml::parse(&format!("generate: {{kind: hills, seed: 2}}\nsize: 4km\nheight: [100, 900]\n{extra}"))
            .map_err(|e| format!("{e:?}"))?;
        TerrainDef::parse("t", Get(&doc), 0)
    }

    #[test]
    fn detail_textures_and_scatter_parse_with_defaults() {
        let t = terrain(
            "detail: textures/d.png\ndetail_scale: 0.04km\nslope_detail: textures/s.png\nscatter:\n  \
             - {model: models/rock.obj, count: 50, scale: [0.5, 2], max_slope: 20deg, height: [200, 800], align: 1, seed: 9, draw_distance: 2km}\n  \
             - {model: models/tree.obj, scale: 3}\n",
        )
        .unwrap();
        assert_eq!(t.detail.as_deref(), Some("textures/d.png"));
        assert_eq!(t.slope_detail.as_deref(), Some("textures/s.png"));
        assert!((t.detail_scale - 40.0).abs() < 1e-4);
        let r = &t.scatter[0];
        assert_eq!((r.model.as_str(), r.count, r.scale, r.max_slope), ("models/rock.obj", 50, [0.5, 2.0], 20.0));
        assert_eq!((r.height, r.align, r.seed, r.draw_distance), ([200.0, 800.0], 1.0, 9, 2000.0));
        // Defaults: 100 pieces, upright, the terrain's whole height range, seed = number + 1.
        let d = &t.scatter[1];
        assert_eq!((d.count, d.scale, d.max_slope, d.height), (100, [3.0, 3.0], 30.0, [100.0, 900.0]));
        assert_eq!((d.align, d.seed, d.draw_distance), (0.0, 2, 4000.0));
    }

    #[test]
    fn bad_detail_and_scatter_values_say_what_is_wrong() {
        for (extra, needle) in [
            ("slope_detail: textures/s.png\n", "slope_detail needs a detail"),
            ("detail: d.png\ndetail_scale: 0\n", "detail_scale must be"),
            ("detail: d.png\ndetail_scale: far\n", "detail_scale must be"),
            ("scatter: {model: m.obj}\n", "scatter is a list"),
            ("scatter:\n  - {count: 4}\n", "scatter[0]: needs a model"),
            ("scatter:\n  - {model: m.obj, count: 0}\n", "count 0 is not between"),
            ("scatter:\n  - {model: m.obj, count: 900000}\n", "not between 1 and 20000"),
            ("scatter:\n  - {model: m.obj, scale: [2, 1]}\n", "smallest first"),
            ("scatter:\n  - {model: m.obj, scale: [1, 2, 3]}\n", "scale is a number or"),
            ("scatter:\n  - {model: m.obj, scale: -1}\n", "must be positive"),
            ("scatter:\n  - {model: m.obj}\n  - {model: m.obj, max_slope: 120}\n", "scatter[1]: max_slope 120"),
            ("scatter:\n  - {model: m.obj, height: [500, 100]}\n", "highest below the lowest"),
            ("scatter:\n  - {model: m.obj, height: [5]}\n", "height is [lowest, highest]"),
            ("scatter:\n  - {model: m.obj, align: 2}\n", "align 2 is not between 0 and 1"),
            ("scatter:\n  - {model: m.obj, draw_distance: 0}\n", "draw_distance must be"),
        ] {
            let e = terrain(extra).unwrap_err();
            assert!(e.contains(needle), "{extra:?}: '{e}' lacks '{needle}'");
        }
    }

    fn rocks(count: usize) -> ScatterDef {
        ScatterDef {
            model: "m".into(),
            count,
            scale: [0.5, 2.0],
            max_slope: 90.0,
            height: [f32::MIN, f32::MAX],
            align: 0.0,
            seed: 1,
            draw_distance: 1000.0,
        }
    }

    #[test]
    fn scatter_is_deterministic_and_stays_inside_the_field() {
        let field = Heightfield::generate("hills", 3, 33, 4000.0, [0.0, 600.0]);
        let def = rocks(200);
        let a = scatter_placements(&def, &field, 42);
        assert_eq!(a.len(), 200, "every piece placed when any ground will do");
        assert_eq!(a, scatter_placements(&def, &field, 42), "same battle seed, same placement");
        assert_ne!(a, scatter_placements(&def, &field, 43), "another battle, other places");
        assert_ne!(a, scatter_placements(&ScatterDef { seed: 2, ..def.clone() }, &field, 42), "the entry's seed");
        for p in &a {
            assert!(p.position.x.abs() <= 2000.0 && p.position.z.abs() <= 2000.0, "{p:?} outside the field");
            assert!((p.position.y - field.height_at(p.position.x, p.position.z)).abs() < 1e-3, "on the ground");
            assert!((0.5..=2.0).contains(&p.scale));
            // Upright (align 0): the piece's up is the world's up.
            assert!((p.rotation * Vec3::Y).abs_diff_eq(Vec3::Y, 1e-4));
        }
    }

    #[test]
    fn scatter_respects_slope_height_and_alignment() {
        let field = Heightfield::generate("craters", 5, 33, 4000.0, [0.0, 600.0]);
        let def = ScatterDef { max_slope: 8.0, height: [100.0, 400.0], align: 1.0, ..rocks(300) };
        let placed = scatter_placements(&def, &field, 7);
        assert!(!placed.is_empty());
        for p in &placed {
            let n = field.normal_at(p.position.x, p.position.z);
            assert!(n.y >= 8f32.to_radians().cos() - 1e-5, "too steep: {n:?}");
            assert!((100.0..=400.0).contains(&p.position.y), "{}", p.position.y);
            // Fully aligned: the piece's up is the ground's normal.
            assert!((p.rotation * Vec3::Y).abs_diff_eq(n, 1e-3));
        }
        // Ground that never suits gives nothing rather than looping for ever.
        let none = ScatterDef { height: [5000.0, 6000.0], ..rocks(50) };
        assert!(scatter_placements(&none, &field, 7).is_empty());
    }

    #[test]
    fn generated_heightfields_span_their_range_are_deterministic_and_interpolate() {
        let a = Heightfield::generate("craters", 7, 65, 20000.0, [0.0, 1200.0]);
        let b = Heightfield::generate("craters", 7, 65, 20000.0, [0.0, 1200.0]);
        assert_eq!(a, b, "same seed, same ground");
        assert_ne!(a, Heightfield::generate("craters", 8, 65, 20000.0, [0.0, 1200.0]));
        let (lo, hi) = a.heights.iter().fold((f32::MAX, f32::MIN), |(l, h), &x| (l.min(x), h.max(x)));
        assert!((lo - 0.0).abs() < 1e-3 && (hi - 1200.0).abs() < 1e-2);
        // Interpolation: a grid point gives its own height; beyond the edge the ground
        // continues as a mirror image (the same height as far back inside).
        let (x, z) = (a.coord(10), a.coord(20));
        assert!((a.height_at(x, z) - a.heights[20 * 65 + 10]).abs() < 1e-3);
        let edge = a.size * 0.5;
        for d in [10.0, 1234.5, 9000.0] {
            assert!((a.height_at(edge + d, z) - a.height_at(edge - d, z)).abs() < 1e-2, "mirrored at {d}");
        }
        assert!((a.height_at(x + 2.0 * a.size, z) - a.height_at(x, z)).abs() < 1e-2, "two tiles over: the same");
        assert!(a.normal_at(0.0, 0.0).y > 0.0, "normals point up");
        let flat = Heightfield { n: 3, size: 100.0, heights: vec![5.0; 9] };
        assert_eq!(flat.normal_at(0.0, 0.0), Vec3::Y);
        let img = Heightfield::from_image(2, 1, &[0, 0, 0, 255, 255, 0, 0, 255], 2, 100.0, [10.0, 20.0]);
        assert_eq!(img.heights, vec![10.0, 20.0, 10.0, 20.0]);
    }
}
