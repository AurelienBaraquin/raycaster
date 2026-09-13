use glam::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub position: Vec2,
    pub dir: Vec2,
    pub plane: Vec2,
}

impl Camera {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            dir: Vec2::new(-1.0, 0.0),
            plane: Vec2::new(0.0, 0.66),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_camera_has_expected_defaults() {
        let cam = Camera::new(Vec2::new(2.0, 2.0));
        assert_eq!(cam.position, Vec2::new(2.0, 2.0));
        assert_eq!(cam.dir, Vec2::new(-1.0, 0.0));
    }
}
