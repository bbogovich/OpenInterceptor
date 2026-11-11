//! # The space camera
//!
//! A first-person camera defined by a position and an orientation quaternion.
//!
//! ## Camera-relative rendering
//!
//! `f32` has about 7 significant digits. Ten kilometres from the origin that leaves
//! roughly millimetre precision — fine — but a GPU's depth and vertex maths lose more,
//! and distant stations would jitter. So we never send absolute world positions to the
//! GPU: every object is drawn at `object_position − camera_position`, and the camera
//! itself sits at the origin. Big numbers are subtracted on the CPU (exactly), and only
//! small numbers reach the GPU.
//!
//! The `macroquad::camera::Camera` trait asks for one matrix: projection × view. The
//! view part here is just the inverse rotation, because the translation is already
//! handled by drawing everything camera-relative.

use glam::{Mat4, Quat, Vec2, Vec3};

/// Near and far clip planes, metres. The near plane is small for the cockpit view; the
/// far plane fits the largest scenario.
pub const Z_NEAR: f32 = 1.0;
pub const Z_FAR: f32 = 200_000.0;

/// Camera state for one frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpaceCamera {
    pub position: Vec3,
    pub rotation: Quat,
    /// Vertical field of view, radians.
    pub fov_y: f32,
    /// Width / height of the viewport.
    pub aspect: f32,
    /// Near clip plane, metres. Small for the 3D cockpit pass, [`Z_NEAR`] otherwise.
    pub z_near: f32,
    /// How fast the viewer is moving (m/s), e.g. the ship the camera rides with. Bolt
    /// streaks are drawn along their motion *relative to the viewer*, which is what
    /// the eye sees: a shot from a drifting ship streaks straight out of its guns.
    pub velocity: Vec3,
}

impl SpaceCamera {
    /// A camera with the default clip planes.
    pub fn new(position: Vec3, rotation: Quat, fov_y: f32, aspect: f32) -> SpaceCamera {
        SpaceCamera { position, rotation, fov_y, aspect, z_near: Z_NEAR, velocity: Vec3::ZERO }
    }

    /// The same view with a different near plane (the cockpit pass uses 5 cm, so
    /// geometry right in front of the pilot's eye is not clipped).
    pub fn with_near(self, z_near: f32) -> SpaceCamera {
        SpaceCamera { z_near, ..self }
    }

    /// This camera, moving with `velocity` (see [`SpaceCamera::velocity`]).
    pub fn with_velocity(self, velocity: Vec3) -> SpaceCamera {
        SpaceCamera { velocity, ..self }
    }

    /// Projection × rotation-only view (see the module docs).
    pub fn view_projection(&self) -> Mat4 {
        let proj = Mat4::perspective_rh_gl(self.fov_y, self.aspect, self.z_near, Z_FAR);
        proj * Mat4::from_quat(self.rotation.inverse())
    }

    /// Model matrix for an object at `position`/`rotation` (camera-relative), scaled.
    pub fn model_matrix(&self, position: Vec3, rotation: Quat, scale: f32) -> Mat4 {
        Mat4::from_scale_rotation_translation(Vec3::splat(scale), rotation, position - self.position)
    }

    /// Projects a world point to screen pixels (origin top-left). Returns `None` when
    /// the point is behind the camera.
    pub fn project(&self, world: Vec3, screen: Vec2) -> Option<Vec2> {
        let clip = self.view_projection() * (world - self.position).extend(1.0);
        if clip.w <= 1e-4 {
            return None;
        }
        let ndc = clip.truncate() / clip.w;
        Some(Vec2::new((ndc.x + 1.0) * 0.5 * screen.x, (1.0 - ndc.y) * 0.5 * screen.y))
    }

    /// Direction of `world` in camera space (x right, y up, −z ahead), normalised.
    pub fn local_direction(&self, world: Vec3) -> Vec3 {
        (self.rotation.inverse() * (world - self.position)).normalize_or_zero()
    }
}

/// Implements macroquad's camera trait so `set_camera(&camera)` works.
impl macroquad::camera::Camera for SpaceCamera {
    fn matrix(&self) -> Mat4 {
        self.view_projection()
    }
    fn depth_enabled(&self) -> bool {
        true
    }
    fn render_pass(&self) -> Option<macroquad::texture::RenderPass> {
        None
    }
    fn viewport(&self) -> Option<(i32, i32, i32, i32)> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cam() -> SpaceCamera {
        SpaceCamera::new(Vec3::new(10_000.0, 0.0, 0.0), Quat::IDENTITY, 1.0, 1.0)
    }

    #[test]
    fn points_ahead_project_to_the_centre_and_behind_is_none() {
        let c = cam();
        let p = c.project(Vec3::new(10_000.0, 0.0, -100.0), Vec2::new(800.0, 600.0)).unwrap();
        assert!((p - Vec2::new(400.0, 300.0)).length() < 1e-2);
        assert!(c.project(Vec3::new(10_000.0, 0.0, 100.0), Vec2::new(800.0, 600.0)).is_none());
        let right = c.project(Vec3::new(10_050.0, 20.0, -100.0), Vec2::new(800.0, 600.0)).unwrap();
        assert!(right.x > 400.0 && right.y < 300.0, "right and up on screen");
    }

    #[test]
    fn rotation_turns_the_view() {
        let mut c = cam();
        c.rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2); // looking down -X
        let p = c.project(Vec3::new(9_900.0, 0.0, 0.0), Vec2::new(100.0, 100.0)).unwrap();
        assert!((p - Vec2::new(50.0, 50.0)).length() < 1e-2);
        assert!((c.local_direction(Vec3::new(9_000.0, 0.0, 0.0)) - Vec3::NEG_Z).length() < 1e-5);
    }

    #[test]
    fn near_plane_changes_only_depth() {
        let c = cam();
        let near = c.with_near(0.05);
        assert_eq!(near.z_near, 0.05);
        let p = Vec3::new(10_000.0, 0.2, -0.5);
        assert!(c.project(p, Vec2::splat(100.0)).is_some());
        let (a, b) = (c.project(p, Vec2::splat(100.0)).unwrap(), near.project(p, Vec2::splat(100.0)).unwrap());
        assert!((a - b).length() < 1e-3, "same screen position");
    }

    #[test]
    fn model_matrices_are_camera_relative() {
        let c = cam();
        let m = c.model_matrix(Vec3::new(10_001.0, 2.0, 3.0), Quat::IDENTITY, 2.0);
        assert_eq!(m.transform_point3(Vec3::ZERO), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(m.transform_point3(Vec3::X), Vec3::new(3.0, 2.0, 3.0));
    }
}
