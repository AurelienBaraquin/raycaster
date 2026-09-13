use crate::{Camera, Map};

#[derive(Debug, Clone, Copy)]
pub struct RayHit {
    pub distance: f32,
    pub hit_vertical_side: bool,
    pub wall_id: u8,
}

pub fn cast_ray(map: &Map, camera: &Camera, camera_x: f32) -> RayHit {
    let ray_dir = camera.dir + camera.plane * camera_x;

    // case de la grille où se trouve la caméra actuellement
    let mut map_x = camera.position.x.floor() as i32;
    let mut map_y = camera.position.y.floor() as i32;

    // distance (le long du rayon) pour traverser une case entière, sur chaque axe
    let delta_dist_x = if ray_dir.x == 0.0 { f32::INFINITY } else { (1.0 / ray_dir.x).abs() };
    let delta_dist_y = if ray_dir.y == 0.0 { f32::INFINITY } else { (1.0 / ray_dir.y).abs() };

    // sens du pas (+1/-1) et distance jusqu'à la PREMIÈRE ligne de grille, par axe
    let (step_x, mut side_dist_x) = if ray_dir.x < 0.0 {
        (-1, (camera.position.x - map_x as f32) * delta_dist_x)
    } else {
        (1, (map_x as f32 + 1.0 - camera.position.x) * delta_dist_x)
    };
    let (step_y, mut side_dist_y) = if ray_dir.y < 0.0 {
        (-1, (camera.position.y - map_y as f32) * delta_dist_y)
    } else {
        (1, (map_y as f32 + 1.0 - camera.position.y) * delta_dist_y)
    };

    let mut hit_vertical_side;

    loop {
        if side_dist_x < side_dist_y {
            side_dist_x += delta_dist_x;
            map_x += step_x;
            hit_vertical_side = true;
        } else {
            side_dist_y += delta_dist_y;
            map_y += step_y;
            hit_vertical_side = false;
        }

        if map.is_wall(map_x, map_y) {
            break;
        }
    }

    // on "annule" le dernier pas en trop : side_dist pointe déjà vers LA PROCHAINE
    // frontière après celle qui vient d'être franchie, donc on retire un delta_dist
    // pour obtenir la distance perpendiculaire jusqu'à la frontière franchie.
    let distance = if hit_vertical_side {
        side_dist_x - delta_dist_x
    } else {
        side_dist_y - delta_dist_y
    };

    RayHit {
        distance,
        hit_vertical_side,
        wall_id: map.wall_id(map_x, map_y).expect("Un mur était attendu ici, les limites de la map ont probablement été dépassé.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    fn sample_map() -> Map {
        Map::from_layout("###\n#.#\n###")
    }

    #[test]
    fn cast_ray_from_camera() {
        let cam = Camera::new(Vec2::new(1.5, 1.5));
        let map = sample_map();
        let rayhit = cast_ray(&map, &cam, 0.0);
        assert_eq!(rayhit.distance, 0.5);
        assert!(rayhit.hit_vertical_side);
    }
}
