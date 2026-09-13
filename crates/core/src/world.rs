use crate::raycaster::cast_ray;
use crate::{Camera, Map, Input};
use glam::Vec2;

const MOVE_SPEED: f32 = 3.0; // cases par seconde
const ROT_SPEED: f32 = 2.5;  // radians par seconde

pub struct World {
    map: Map,
    camera: Camera,
}

impl World {
    pub fn new(map: Map, camera: Camera) -> Self {
        Self { map, camera }
    }

    pub fn render(&self, framebuffer: &mut [u32], width: usize, height: usize) {
        for x in 0..width {
            // colonne d'écran -> position normalisée [-1, 1] sur le plan caméra
            let camera_x = 2.0 * x as f32 / width as f32 - 1.0;
            let hit = cast_ray(&self.map, &self.camera, camera_x);

            // plus la distance est petite, plus le mur occupe de hauteur à l'écran
            let line_height = (height as f32 / hit.distance) as i32;
            let half_height = height as i32 / 2;

            // calcul en i32 d'abord (peut être négatif si line_height > height),
            // clamp ensuite avant de repasser en usize — même précaution que pour
            // les coordonnées de Map::is_wall
            let draw_start = (half_height - line_height / 2).max(0) as usize;
            let draw_end = (half_height + line_height / 2).min(height as i32 - 1) as usize;

            let wall_color = if hit.hit_vertical_side {
                0x00_A0_A0_A0 // face est/ouest, plus clair
            } else {
                0x00_60_60_60 // face nord/sud, plus sombre -> donne du relief
            };

            for y in 0..draw_start {
                framebuffer[y * width + x] = 0x00_30_30_30; // plafond
            }
            for y in draw_start..=draw_end {
                framebuffer[y * width + x] = wall_color;
            }
            for y in (draw_end + 1)..height {
                framebuffer[y * width + x] = 0x00_50_50_50; // sol
            }
        }
    }

    pub fn update(&mut self, input: Input, dt: f32) {
        let turn_amount = input.turn * ROT_SPEED * dt;
        self.camera.dir = self.camera.dir.rotate_angle(turn_amount);
        self.camera.plane = self.camera.plane.rotate_angle(turn_amount);

        // signe inverse par rapport a dir.perp() brut : verifie empiriquement
        // (Phase 5, controle natif) que .perp() pointait a gauche, pas a droite
        let strafe_dir = -self.camera.dir.perp();
        let delta = (self.camera.dir * input.forward + strafe_dir * input.strafe) * MOVE_SPEED * dt;

        let target_x = self.camera.position + Vec2::new(delta.x, 0.0);
        if !self.map.is_wall(target_x.x.floor() as i32, target_x.y.floor() as i32) {
            self.camera.position.x = target_x.x;
        }

        let target_y = self.camera.position + Vec2::new(0.0, delta.y);
        if !self.map.is_wall(target_y.x.floor() as i32, target_y.y.floor() as i32) {
            self.camera.position.y = target_y.y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    #[test]
    fn render_draws_ceiling_wall_and_floor_in_expected_bands() {
        let map = Map::from_layout("#####\n#...#\n#...#\n#...#\n#####");
        let camera = Camera::new(Vec2::new(2.5, 2.5));
        let world = World::new(map, camera);

        let (width, height) = (2, 10);
        let mut framebuffer = vec![0u32; width * height];
        world.render(&mut framebuffer, width, height);

        // Colonne x=1 -> camera_x=0 (rayon droit devant, dir=(-1,0)).
        // Le mur ouest est a la coordonnee de grille x=1.0 (bord de la case (0, y)),
        // camera en x=2.5 -> distance perpendiculaire = 1.5.
        // line_height = (10 / 1.5) as i32 = 6 (troncature), half_height = 5,
        // draw_start = (5 - 3).max(0) = 2, draw_end = (5 + 3).min(9) = 8.
        let x = 1;
        let pixel_at = |y: usize| framebuffer[y * width + x];

        assert_eq!(pixel_at(0), 0x00_30_30_30, "plafond attendu au-dessus du mur");
        assert_eq!(pixel_at(5), 0x00_A0_A0_A0, "mur (cote vertical) attendu au centre");
        assert_eq!(pixel_at(9), 0x00_50_50_50, "sol attendu en dessous du mur");
    }

    #[test]
    fn update_moves_forward_when_path_is_clear() {
        let map = Map::from_layout("#####\n#...#\n#...#\n#...#\n#####");
        let camera = Camera::new(Vec2::new(2.5, 2.5));
        let mut world = World::new(map, camera);

        let input = Input { forward: 1.0, strafe: 0.0, turn: 0.0 };
        world.update(input, 0.1);

        // dir = (-1, 0), MOVE_SPEED = 3.0 -> delta = (-0.3, 0.0)
        approx::assert_relative_eq!(world.camera.position.x, 2.2, epsilon = 1e-5);
        approx::assert_relative_eq!(world.camera.position.y, 2.5, epsilon = 1e-5);
    }

    #[test]
    fn update_is_blocked_by_a_wall() {
        let map = Map::from_layout("#####\n#...#\n#...#\n#...#\n#####");
        let camera = Camera::new(Vec2::new(1.1, 2.5));
        let mut world = World::new(map, camera);

        // deplacement volontairement enorme pour traverser le mur sans ambiguite
        let input = Input { forward: 1.0, strafe: 0.0, turn: 0.0 };
        world.update(input, 1.0);

        assert_eq!(world.camera.position, Vec2::new(1.1, 2.5));
    }
}