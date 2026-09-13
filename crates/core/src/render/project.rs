/// Facteur d'echelle pixels/unite-monde a une profondeur donnee, calibre sur
/// le FOV horizontal et la largeur d'ecran. Le meme facteur sert pour X et Y :
/// avec un FOV vertical derive du FOV horizontal et du ratio largeur/hauteur
/// (pixels carres), les deux echelles sont mathematiquement identiques - voir
/// le detail dans la conversation/le plan. Ca evite de calculer un FOV
/// vertical separe.
pub fn projection_scale(depth: f32, fov_horizontal: f32, screen_width: f32) -> f32 {
    screen_width / (2.0 * depth * (fov_horizontal / 2.0).tan())
}

/// Colonne d'ecran pour une coordonnee X en espace camera, a l'echelle donnee.
pub fn project_x(camera_x: f32, screen_width: f32, scale: f32) -> f32 {
    screen_width / 2.0 + camera_x * scale
}

/// Ligne d'ecran pour une hauteur monde donnee (sol/plafond), relative a la
/// hauteur des yeux de la camera, a l'echelle donnee. L'axe ecran est inverse
/// par rapport a l'axe monde (une hauteur monde plus grande doit apparaitre
/// plus haut a l'ecran, donc une ligne plus petite).
pub fn project_y(world_height: f32, eye_height: f32, screen_height: f32, scale: f32) -> f32 {
    screen_height / 2.0 - (world_height - eye_height) * scale
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    #[test]
    fn scale_matches_hand_computed_value() {
        // FOV 90 degres -> tan(45deg) == 1.0 -> scale = width / (2*depth*1.0)
        let scale = projection_scale(1.0, FRAC_PI_2, 200.0);
        assert!((scale - 100.0).abs() < 1e-4);
    }

    #[test]
    fn point_straight_ahead_projects_to_screen_center() {
        let scale = projection_scale(1.0, FRAC_PI_2, 200.0);
        assert_eq!(project_x(0.0, 200.0, scale), 100.0);
    }

    #[test]
    fn point_at_fov_edge_projects_to_screen_edge() {
        // a depth=1 avec un FOV de 90deg, le bord du champ de vision est a
        // camera_x = tan(45deg) = 1.0
        let scale = projection_scale(1.0, FRAC_PI_2, 200.0);
        assert!((project_x(1.0, 200.0, scale) - 200.0).abs() < 1e-3);
    }

    #[test]
    fn height_equal_to_eye_height_is_the_horizon() {
        let scale = projection_scale(1.0, FRAC_PI_2, 200.0);
        assert_eq!(project_y(0.5, 0.5, 100.0, scale), 50.0);
    }

    #[test]
    fn height_above_eye_projects_above_center() {
        let scale = projection_scale(1.0, FRAC_PI_2, 200.0);
        // plus haut dans le monde -> ligne d'ecran plus petite (plus haut a l'ecran)
        assert!(project_y(1.0, 0.5, 100.0, scale) < 50.0);
    }
}
