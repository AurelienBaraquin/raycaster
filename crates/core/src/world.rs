use glam::Vec2;

use crate::door::Door;
use crate::level::{Level, SectorId};
use crate::render::render_level;
use crate::{Camera, Input};

const MOVE_SPEED: f32 = 3.0; // unites monde par seconde
const ROT_SPEED: f32 = 2.5; // radians par seconde
/// Ecart minimal sol/plafond pour qu'un secteur soit considere franchissable.
/// Une porte fermee a `ceiling_height == floor_height` (ecart nul) : ce seuil
/// la traite alors comme un mur, sans logique spec ifique "porte" ici.
const MIN_CLEARANCE: f32 = 0.1;

pub struct World {
    level: Level,
    camera: Camera,
    doors: Vec<Door>,
}

impl World {
    pub fn new(level: Level, camera: Camera) -> Self {
        let doors = collect_doors(&level);
        Self { level, camera, doors }
    }

    pub fn render(&self, framebuffer: &mut [u32], width: usize, height: usize) {
        render_level(&self.level, &self.camera, framebuffer, width, height);
    }

    pub fn update(&mut self, input: Input, dt: f32) {
        self.camera.dir = self.camera.dir.rotate_angle(input.turn * ROT_SPEED * dt);

        let delta = (self.camera.dir * input.forward + self.camera.right() * input.strafe) * MOVE_SPEED * dt;

        // Collision par axe separe (X puis Y) pour glisser le long des murs :
        // un point est franchissable s'il tombe dans un secteur du niveau ET
        // que ce secteur a assez de degagement (voir MIN_CLEARANCE - c'est ce
        // qui bloque le passage tant qu'une porte n'est pas ouverte).
        let target_x = self.camera.position + Vec2::new(delta.x, 0.0);
        if self.is_passable(target_x) {
            self.camera.position.x = target_x.x;
        }

        let target_y = self.camera.position + Vec2::new(0.0, delta.y);
        if self.is_passable(target_y) {
            self.camera.position.y = target_y.y;
        }

        let player_position = self.camera.position;
        for door in &mut self.doors {
            door.update(&mut self.level, player_position, dt);
        }
    }

    fn is_passable(&self, point: Vec2) -> bool {
        self.level
            .sector_containing(point)
            .is_some_and(|id| self.level.sectors[id].ceiling_height - self.level.sectors[id].floor_height >= MIN_CLEARANCE)
    }
}

/// Construit l'etat d'execution des portes a partir des secteurs marques
/// `door: Some(...)` dans les donnees du niveau (RON). Le niveau reste la
/// seule source de verite sur "quelles portes existent" - `World` ne fait
/// qu'en deriver le runtime mutable.
fn collect_doors(level: &Level) -> Vec<Door> {
    level
        .sectors
        .iter()
        .enumerate()
        .filter_map(|(id, sector): (SectorId, _)| {
            let def = sector.door?;
            let trigger_point = level.sector_centroid(id);
            Some(Door::new(id, trigger_point, def.trigger_radius, sector.ceiling_height, def.open_height))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::fixtures::single_room;

    #[test]
    fn update_moves_forward_when_path_is_clear() {
        let level = single_room();
        let camera = Camera::new(Vec2::new(5.0, 5.0));
        let mut world = World::new(level, camera);

        let input = Input { forward: 1.0, strafe: 0.0, turn: 0.0 };
        world.update(input, 0.1);

        // dir = (-1, 0), MOVE_SPEED = 3.0 -> delta = (-0.3, 0.0)
        approx::assert_relative_eq!(world.camera.position.x, 4.7, epsilon = 1e-5);
        approx::assert_relative_eq!(world.camera.position.y, 5.0, epsilon = 1e-5);
    }

    #[test]
    fn update_is_blocked_by_a_wall() {
        let level = single_room();
        let camera = Camera::new(Vec2::new(0.5, 5.0)); // tout pres du mur ouest (x=0)
        let mut world = World::new(level, camera);

        let input = Input { forward: 1.0, strafe: 0.0, turn: 0.0 };
        world.update(input, 1.0); // deplacement volontairement enorme

        assert_eq!(world.camera.position, Vec2::new(0.5, 5.0));
    }

    #[test]
    fn render_produces_output_for_a_valid_scene() {
        let level = single_room();
        let camera = Camera::new(Vec2::new(5.0, 5.0));
        let world = World::new(level, camera);

        let mut framebuffer = vec![0u32; 20 * 20];
        world.render(&mut framebuffer, 20, 20);

        assert!(framebuffer.iter().any(|&px| px != 0), "le rendu doit produire des pixels non nuls");
    }

    #[test]
    fn a_closed_door_blocks_movement_even_though_the_point_is_in_a_sector() {
        use crate::level::{DoorDef, Sector, Wall, WallKind};

        // Deux pieces separees par un secteur-porte ferme (ceiling == floor).
        let vertices = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 0.0),
            Vec2::new(5.0, 5.0),
            Vec2::new(0.0, 5.0),
            Vec2::new(5.3, 0.0),
            Vec2::new(5.3, 5.0),
            Vec2::new(10.3, 0.0),
            Vec2::new(10.3, 5.0),
        ];
        let room_a = Sector {
            walls: vec![
                Wall { start: 0, end: 1, kind: WallKind::Solid { material: 3 } },
                Wall { start: 1, end: 2, kind: WallKind::Portal { neighbor: 2, step_material: 3 } },
                Wall { start: 2, end: 3, kind: WallKind::Solid { material: 3 } },
                Wall { start: 3, end: 0, kind: WallKind::Solid { material: 3 } },
            ],
            floor_height: 0.0,
            ceiling_height: 2.0,
            light_level: 1.0,
            floor_material: 1,
            ceiling_material: 2,
            door: None,
        };
        let door_sector = Sector {
            walls: vec![
                Wall { start: 1, end: 4, kind: WallKind::Solid { material: 3 } },
                Wall { start: 4, end: 5, kind: WallKind::Portal { neighbor: 1, step_material: 3 } },
                Wall { start: 5, end: 2, kind: WallKind::Solid { material: 3 } },
                Wall { start: 2, end: 1, kind: WallKind::Portal { neighbor: 0, step_material: 3 } },
            ],
            floor_height: 0.0,
            ceiling_height: 0.0, // fermee : aucun degagement
            light_level: 1.0,
            floor_material: 1,
            ceiling_material: 2,
            door: Some(DoorDef { open_height: 2.0, trigger_radius: 1.0 }),
        };
        let room_b = Sector {
            walls: vec![
                Wall { start: 4, end: 6, kind: WallKind::Solid { material: 3 } },
                Wall { start: 6, end: 7, kind: WallKind::Solid { material: 3 } },
                Wall { start: 7, end: 5, kind: WallKind::Solid { material: 3 } },
                Wall { start: 5, end: 4, kind: WallKind::Portal { neighbor: 2, step_material: 3 } },
            ],
            floor_height: 0.0,
            ceiling_height: 2.0,
            light_level: 1.0,
            floor_material: 1,
            ceiling_material: 2,
            door: None,
        };
        let level = Level::new(vertices, vec![room_a, room_b, door_sector], Vec::new());

        // Camera loin du declencheur (0,0 a distance du centroid de la porte),
        // proche du seuil mais du cote room_a, essaie d'avancer vers la porte.
        let camera = Camera { position: Vec2::new(4.8, 2.5), dir: Vec2::new(1.0, 0.0), eye_height: 1.0, fov: 66f32.to_radians() };
        let mut world = World::new(level, camera);

        // dt modeste expres : un pas de temps enorme sauterait carrement par-dessus
        // le fin secteur-porte (0.3 unite) jusque dans room_b - pas ce qu'on teste ici.
        let input = Input { forward: 1.0, strafe: 0.0, turn: 0.0 };
        world.update(input, 0.1);

        assert_eq!(world.camera.position.x, 4.8, "le joueur ne doit pas pouvoir entrer dans une porte fermee");
    }

    #[test]
    fn the_real_demo_level_door_opens_when_the_player_approaches() {
        use crate::level::demo_level;

        // Secteur 2 = le secteur-porte dans demo.ron (voir le commentaire du fichier).
        const DOOR_SECTOR: usize = 2;

        let level = demo_level();
        assert_eq!(
            level.sectors[DOOR_SECTOR].ceiling_height, level.sectors[DOOR_SECTOR].floor_height,
            "la porte doit demarrer fermee"
        );

        // Pres du centroid de la porte (~10.15, 4), dans la grande salle.
        let camera = Camera::new(Vec2::new(9.0, 4.0));
        let mut world = World::new(level, camera);

        let stand_still = Input { forward: 0.0, strafe: 0.0, turn: 0.0 };
        for _ in 0..10 {
            world.update(stand_still, 0.2);
        }

        assert!(
            world.level.sectors[DOOR_SECTOR].ceiling_height > world.level.sectors[DOOR_SECTOR].floor_height,
            "la porte du niveau de demo doit s'ouvrir quand le joueur reste a proximite"
        );
    }
}
