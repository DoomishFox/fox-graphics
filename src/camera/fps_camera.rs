use cgmath::EuclideanSpace;
use glam::{Mat4, Vec3};
use winit::event::{WindowEvent, KeyboardInput, ElementState, VirtualKeyCode};
use crate::camera::Camera;

pub struct CameraLegacy {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera for CameraLegacy {
    fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(
            Vec3::new(self.eye.x, self.eye.y, self.eye.z),
            Vec3::new(self.target.x, self.target.y, self.target.z),
            Vec3::new(self.up.x, self.up.y, self.up.z)
        );
        let proj = Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);
        proj * view
    }
}