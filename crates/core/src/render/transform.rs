use crate::Camera;
use glam::Vec2;

/// Un point exprime dans l'espace camera : `x` = composante le long de
/// `camera.right()` (positif = a droite), `y` = composante le long de
/// `camera.dir` (positif = devant la camera, c'est la profondeur).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraSpace {
    pub x: f32,
    pub y: f32,
}

/// Transforme un point du monde (coordonnees globales) en espace camera.
pub fn to_camera_space(camera: &Camera, world_point: Vec2) -> CameraSpace {
    let relative = world_point - camera.position;
    CameraSpace {
        x: relative.dot(camera.right()),
        y: relative.dot(camera.dir),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_directly_ahead_has_zero_x() {
        let camera = Camera::new(Vec2::new(0.0, 0.0)); // dir = (-1, 0)
        let point = Vec2::new(-5.0, 0.0); // droit devant, 5 unites plus loin
        let result = to_camera_space(&camera, point);

        assert_eq!(result.x, 0.0);
        assert_eq!(result.y, 5.0);
    }

    #[test]
    fn point_to_the_right_has_positive_x() {
        let camera = Camera::new(Vec2::new(0.0, 0.0)); // dir = (-1, 0), right = (0, -1)... voir ci-dessous
        let point = camera.position + camera.right() * 3.0;
        let result = to_camera_space(&camera, point);

        assert!(result.x > 0.0);
        assert_eq!(result.y, 0.0);
    }

    #[test]
    fn point_behind_camera_has_negative_y() {
        let camera = Camera::new(Vec2::new(0.0, 0.0));
        let point = Vec2::new(5.0, 0.0); // derriere (dir pointe vers -X)
        let result = to_camera_space(&camera, point);

        assert!(result.y < 0.0);
    }
}
