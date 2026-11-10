//! # Starfield
//!
//! The background is a few thousand stars at "infinity": directions only. They are drawn
//! around the camera every frame without moving with it, so they never get closer — the
//! classic skybox trick. The star directions come from a seeded generator, so each
//! scenario's `starfield_seed` always produces the same sky.

use glam::Vec3;

/// One star: a unit direction, a size and a colour.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Star {
    pub direction: Vec3,
    pub size: f32,
    pub color: [u8; 4],
}

/// Generates `count` stars, uniformly over the sphere, from `seed`.
pub fn generate(seed: u64, count: usize) -> Vec<Star> {
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).max(1);
    // xorshift64*: a tiny, good-enough generator for decoration.
    let mut next = move || {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        (state.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 40) as f32 / (1u64 << 24) as f32
    };
    (0..count)
        .map(|_| {
            // Uniform on a sphere: z uniform in -1..1, angle uniform in 0..2π.
            let z = next() * 2.0 - 1.0;
            let a = next() * std::f32::consts::TAU;
            let r = (1.0 - z * z).sqrt();
            let brightness = next().powi(3);
            let warmth = next();
            let level = (90.0 + 165.0 * brightness) as u8;
            let tint = |k: f32| (level as f32 * (0.8 + 0.2 * k)) as u8;
            Star {
                direction: Vec3::new(r * a.cos(), r * a.sin(), z),
                size: 0.6 + 1.6 * brightness,
                color: [tint(warmth), tint(0.7), tint(1.0 - warmth), 255],
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sky_and_unit_directions() {
        let a = generate(7, 500);
        assert_eq!(a, generate(7, 500));
        assert_ne!(a, generate(8, 500));
        assert!(a.iter().all(|s| (s.direction.length() - 1.0).abs() < 1e-4));
        // Roughly uniform: about half the stars in each hemisphere.
        let up = a.iter().filter(|s| s.direction.y > 0.0).count();
        assert!((200..300).contains(&up), "{up}");
    }
}
