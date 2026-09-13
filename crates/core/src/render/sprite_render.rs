use crate::entity::Entity;
use crate::lighting::shade;
use crate::materials::sprite_texture;
use crate::Camera;

use super::project::{project_x, project_y, projection_scale};
use super::transform::{to_camera_space, CameraSpace};
use super::RenderContext;

/// Distance minimale devant la camera pour qu'un sprite soit considere
/// visible (evite une projection instable/division degeneree tout pres du
/// plan camera, meme principe que `NEAR` dans `clip.rs` mais un concept
/// distinct : les sprites n'ont pas besoin d'etre clippes, juste ignores
/// s'ils sont trop proches).
const MIN_SPRITE_DEPTH: f32 = 0.05;

/// Dessine tous les sprites du niveau, tries du plus loin au plus proche
/// (comme la geometrie), et occultes colonne par colonne via le z-buffer
/// rempli par `sector_render` pendant la passe geometrie.
pub(super) fn render_sprites(ctx: RenderContext, depth_buffer: &[f32], framebuffer: &mut [u32]) {
    let RenderContext { level, camera, width, height } = ctx;

    let mut visible: Vec<(&Entity, CameraSpace, f32)> = level
        .entities
        .iter()
        .filter_map(|entity| {
            let cam_pos = to_camera_space(camera, entity.position);
            if cam_pos.y <= MIN_SPRITE_DEPTH {
                return None;
            }
            let light_level = level
                .sector_containing(entity.position)
                .map(|id| level.sectors[id].light_level)
                .unwrap_or(1.0);
            Some((entity, cam_pos, light_level))
        })
        .collect();

    // Plus loin d'abord : un sprite plus proche doit pouvoir recouvrir un
    // sprite plus lointain dans la meme colonne (peinture arriere-avant,
    // meme logique que le reste du renderer).
    visible.sort_by(|a, b| b.1.y.total_cmp(&a.1.y));

    for (entity, cam_pos, light_level) in visible {
        draw_sprite(entity, cam_pos, light_level, camera, depth_buffer, framebuffer, width, height);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_sprite(
    entity: &Entity,
    cam_pos: CameraSpace,
    light_level: f32,
    camera: &Camera,
    depth_buffer: &[f32],
    framebuffer: &mut [u32],
    width: usize,
    height: usize,
) {
    let texture = sprite_texture(entity.sprite);
    let scale = projection_scale(cam_pos.y, camera.fov, width as f32);

    let center_x = project_x(cam_pos.x, width as f32, scale);
    let half_width_px = entity.half_width * scale;
    let left = center_x - half_width_px;
    let right = center_x + half_width_px;

    if right <= 0.0 || left >= width as f32 {
        return; // entierement hors ecran
    }

    let top_y = project_y(entity.base_height + entity.height, camera.eye_height, height as f32, scale);
    let bottom_y = project_y(entity.base_height, camera.eye_height, height as f32, scale);

    let x_start = clamp_to_range(left, 0, width);
    let x_end = clamp_to_range(right, 0, width);
    let y_start = clamp_to_range(top_y, 0, height);
    let y_end = clamp_to_range(bottom_y, 0, height);
    if x_start >= x_end || y_start >= y_end {
        return;
    }

    let band_width = (right - left).max(1.0);
    let band_height = (y_end - y_start) as f32;

    for x in x_start..x_end {
        if cam_pos.y >= depth_buffer[x] {
            continue; // un mur plus proche occulte le sprite sur cette colonne
        }

        let u = (x as f32 + 0.5 - left) / band_width;
        for y in y_start..y_end {
            let v = (y - y_start) as f32 / band_height;
            if let Some(color) = texture.sample(u, v) {
                framebuffer[y * width + x] = shade(color, light_level, cam_pos.y);
            }
        }
    }
}

fn clamp_to_range(value: f32, min: usize, max: usize) -> usize {
    (value.round() as isize).clamp(min as isize, max as isize) as usize
}

#[cfg(test)]
mod tests {
    use crate::entity::Entity;
    use crate::level::{Level, Sector, Wall, WallKind};
    use crate::materials::SpriteKind;
    use crate::render::render_level;
    use crate::Camera;
    use glam::Vec2;

    fn room_with_entities(entities: Vec<Entity>) -> Level {
        let vertices = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ];
        let sector = Sector {
            walls: vec![
                Wall { start: 0, end: 1, kind: WallKind::Solid { material: 9 } },
                Wall { start: 1, end: 2, kind: WallKind::Solid { material: 9 } },
                Wall { start: 2, end: 3, kind: WallKind::Solid { material: 9 } },
                Wall { start: 3, end: 0, kind: WallKind::Solid { material: 9 } },
            ],
            floor_height: 0.0,
            ceiling_height: 2.0,
            light_level: 1.0,
            floor_material: 1,
            ceiling_material: 2,
            door: None,
        };
        Level::new(vertices, vec![sector], entities)
    }

    fn barrel_at(position: Vec2) -> Entity {
        Entity { position, base_height: 0.0, half_width: 0.4, height: 0.8, sprite: SpriteKind::Barrel }
    }

    fn render(level: &Level, camera: &Camera, width: usize, height: usize) -> Vec<u32> {
        let mut framebuffer = vec![0u32; width * height];
        render_level(level, camera, &mut framebuffer, width, height);
        framebuffer
    }

    #[test]
    fn unoccluded_sprite_changes_the_render() {
        let camera = Camera { position: Vec2::new(5.0, 5.0), dir: Vec2::new(-1.0, 0.0), eye_height: 0.5, fov: 66f32.to_radians() };
        let (width, height) = (100, 100);

        let without_sprite = render(&room_with_entities(vec![]), &camera, width, height);
        let with_sprite = render(&room_with_entities(vec![barrel_at(Vec2::new(3.0, 5.0))]), &camera, width, height);

        assert_ne!(with_sprite, without_sprite, "un sprite non occulte doit changer le rendu");
    }

    #[test]
    fn sprite_behind_a_wall_does_not_change_the_render() {
        // Camera face au mur est (depth=5), tonneau aligne sur le meme rayon
        // mais a depth=7 (au-dela du mur, donc geometriquement impossible a
        // voir) -> le z-buffer doit l'occulter completement.
        let camera = Camera { position: Vec2::new(5.0, 5.0), dir: Vec2::new(1.0, 0.0), eye_height: 0.5, fov: 66f32.to_radians() };
        let (width, height) = (100, 100);

        let without_sprite = render(&room_with_entities(vec![]), &camera, width, height);
        let with_sprite = render(&room_with_entities(vec![barrel_at(Vec2::new(12.0, 5.0))]), &camera, width, height);

        assert_eq!(with_sprite, without_sprite, "un sprite occulte par un mur plus proche ne doit rien changer au rendu");
    }

    #[test]
    fn sprite_behind_a_closed_door_is_not_visible() {
        // Regression : le z-buffer n'etait rempli que par les murs `Solid`,
        // jamais par les bandes "marche" d'un portail ferme (porte close) ->
        // un tonneau dans la piece voisine apparaissait par-dessus la porte
        // fermee. Utilise le vrai niveau de demo (porte fermee par defaut).
        use crate::level::demo_level;

        let camera = Camera { position: Vec2::new(9.0, 4.0), dir: Vec2::new(1.0, 0.0), eye_height: 1.0, fov: 66f32.to_radians() };
        let (width, height) = (200, 150);

        let with_entities = demo_level();
        let mut without_entities = demo_level();
        without_entities.entities.clear();

        let with = render(&with_entities, &camera, width, height);
        let without = render(&without_entities, &camera, width, height);

        assert_eq!(with, without, "un sprite derriere une porte fermee ne doit pas etre visible");
    }
}
