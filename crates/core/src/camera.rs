use glam::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub position: Vec2,
    pub dir: Vec2,
    /// Hauteur des yeux du joueur (axe vertical du monde, meme echelle que
    /// floor_height/ceiling_height des secteurs).
    pub eye_height: f32,
    /// Champ de vision horizontal, en radians.
    pub fov: f32,
}

impl Camera {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            dir: Vec2::new(-1.0, 0.0),
            eye_height: 0.5,
            fov: 66f32.to_radians(),
        }
    }

    /// Vecteur unitaire pointant vers la "droite" de la camera. Signe verifie
    /// empiriquement (Phase 5) pour que le strafe D aille bien a droite.
    pub fn right(&self) -> Vec2 {
        -self.dir.perp()
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

    #[test]
    fn right_is_perpendicular_to_dir() {
        let cam = Camera::new(Vec2::new(0.0, 0.0));
        assert_eq!(cam.dir.dot(cam.right()), 0.0);
    }
}
