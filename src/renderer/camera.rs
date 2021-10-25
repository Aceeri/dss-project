
use glam::{Vec2, Vec3, Mat4};

// An orthographic camera, mostly just used here for keeping scaling of objects tidy, but
// could be easily swapped out later for a perspective camera to make things look more fancy.
#[derive(Debug, Clone)]
pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        let aspect_ratio = width / height;
        let scaling = 5.0; // Just make things a bit easier to work with.
        Self {
            // Back up 1 so we can actually see the images.
            eye: Vec3::new(0.0, 0.0, 1.0),
            target: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::Y,

            // Scale vertically and have it "anchor" at the top left.
            left: 0.0,
            right: aspect_ratio * scaling,
            top: 0.0,
            bottom: -1.0 * scaling,

            near: 0.0,
            far: 100.0,
        }
    }

    pub fn build_view_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let ortho = Mat4::orthographic_rh(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        );
        ortho * view
    }

    pub fn point_in_window<V: AsRef<Vec2>>(&self, point: V) -> bool {
        let point = point.as_ref();
        point.x > self.left && point.x < self.right && point.y > self.top && point.y < -self.bottom
    }
}

// Something that we will actually send to the GPU for the shaders to use.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_matrix: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_matrix: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn set_view_matrix(&mut self, camera: &Camera) {
        self.view_matrix = camera.build_view_matrix().to_cols_array_2d();
    }
}