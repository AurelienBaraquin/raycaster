use crate::level::{MaterialId, SectorId, WallKind};
use crate::lighting::shade;
use crate::materials::{material_color, material_texture};

use super::clip::clip_near;
use super::project::{project_x, project_y, projection_scale};
use super::transform::to_camera_space;
use super::RenderContext;

/// Nombre d'unites-monde couvertes par un carreau de texture le long d'un
/// mur (plus petit = motif plus resserre / plus de repetitions).
const TEXTURE_WORLD_SCALE: f32 = 2.0;

/// Garde-fou anti-recursion infinie a travers des portails (niveau mal forme
/// ou boucle de secteurs). En pratique la fenetre de colonnes se retrecit a
/// chaque portail traverse, donc la recursion s'arrete bien avant cette
/// limite sur un niveau sain ; c'est uniquement un filet de securite.
const MAX_PORTAL_DEPTH: u32 = 32;

/// Plage de colonnes visibles + limites verticales *par colonne*. Le "par
/// colonne" est essentiel : une fenetre de portail vue en biais est un
/// trapeze a l'ecran, pas un rectangle. Une premiere version de ce renderer
/// aplatissait `y_top`/`y_bottom` en un seul min/max pour toute la largeur du
/// portail (plus simple, mais donnait des portes visiblement rectangulaires
/// et figees quel que soit l'angle de vue) ; corrige suite a un test manuel
/// par l'utilisateur.
#[derive(Clone)]
struct ScreenBounds {
    x_min: usize,
    x_max: usize,
    /// Indexe par `x - x_min`. `y_top[i]` = plus haute ligne visible (incluse)
    /// pour la colonne `x_min + i`, `y_bottom[i]` = plus basse ligne visible
    /// (exclue).
    y_top: Vec<usize>,
    y_bottom: Vec<usize>,
}

impl ScreenBounds {
    fn full(width: usize, height: usize) -> Self {
        Self {
            x_min: 0,
            x_max: width,
            y_top: vec![0; width],
            y_bottom: vec![height; width],
        }
    }

    fn y_range(&self, x: usize) -> (usize, usize) {
        let i = x - self.x_min;
        (self.y_top[i], self.y_bottom[i])
    }
}

/// Point d'entree appele par `render::render_level` : rendu par transformation
/// camera + projection perspective + clipping plan proche + traversee
/// recursive de portails, colonne d'ecran par colonne. Remplit `depth_buffer`
/// (profondeur du mur plein le plus proche par colonne) au passage, utilise
/// ensuite par `sprite_render` pour occulter les sprites derriere les murs.
pub(super) fn render_geometry(
    ctx: RenderContext,
    start_sector: SectorId,
    framebuffer: &mut [u32],
    depth_buffer: &mut [f32],
) {
    let bounds = ScreenBounds::full(ctx.width, ctx.height);
    render_sector(ctx, start_sector, bounds, framebuffer, depth_buffer, 0);
}

fn render_sector(
    ctx: RenderContext,
    sector_id: SectorId,
    bounds: ScreenBounds,
    framebuffer: &mut [u32],
    depth_buffer: &mut [f32],
    depth: u32,
) {
    if depth > MAX_PORTAL_DEPTH || bounds.x_min >= bounds.x_max {
        return;
    }

    let RenderContext { level, camera, width, height } = ctx;
    let sector = &level.sectors[sector_id];

    for wall in &sector.walls {
        let a = level.vertices[wall.start];
        let b = level.vertices[wall.end];

        // Face arriere : un mur ne se rend que si la camera est du cote
        // interieur de CE mur precis (polygone en ordre CCW : interieur a
        // gauche de a->b, ce qui correspond a perp_dot > 0). Verifie a la
        // main sur plusieurs cas avant d'ecrire ce test.
        let wall_dir = b - a;
        if wall_dir.perp_dot(camera.position - a) <= 0.0 {
            continue;
        }

        let cam_a = to_camera_space(camera, a);
        let cam_b = to_camera_space(camera, b);
        let Some((cam_a, cam_b)) = clip_near(cam_a, cam_b) else {
            continue;
        };

        // Pour un mur qui passe le test de face arriere ci-dessus, `b`
        // projette toujours a gauche de l'ecran et `a` a droite (verifie a
        // la main sur plusieurs cas concrets).
        let scale_a = projection_scale(cam_a.y, camera.fov, width as f32);
        let scale_b = projection_scale(cam_b.y, camera.fov, width as f32);
        let screen_left = project_x(cam_b.x, width as f32, scale_b);
        let screen_right = project_x(cam_a.x, width as f32, scale_a);

        if screen_left >= screen_right {
            continue; // mur vu quasiment de profil, largeur ecran nulle
        }

        let x_start = clamp_coord(screen_left, bounds.x_min, bounds.x_max);
        let x_end = clamp_coord(screen_right, bounds.x_min, bounds.x_max);
        if x_start >= x_end {
            continue;
        }

        let wall_length = wall_dir.length();

        match wall.kind {
            WallKind::Solid { material } => {
                // `x` sert a indexer bien plus que `depth_buffer` ici
                // (bounds, framebuffer via draw_column/draw_wall_band) :
                // un iterateur+enumerate() sur un seul de ces tableaux
                // n'apporterait rien.
                #[allow(clippy::needless_range_loop)]
                for x in x_start..x_end {
                    let (y_min, y_max) = bounds.y_range(x);
                    if y_min >= y_max {
                        continue;
                    }

                    let t = inverse_lerp(screen_left, screen_right, x as f32 + 0.5);
                    let wall_depth = perspective_depth(cam_b.y, cam_a.y, t);
                    let scale = projection_scale(wall_depth, camera.fov, width as f32);

                    // Seuls les murs pleins occultent les sprites : un mur
                    // plein remplit toute sa bande verticale, donc une seule
                    // profondeur par colonne suffit a representer "rien de
                    // visible au-dela". Les portails ne mettent PAS a jour ce
                    // buffer (voir sprite_render.rs pour le detail).
                    depth_buffer[x] = depth_buffer[x].min(wall_depth);

                    let ceiling_y = clamp_coord(
                        project_y(sector.ceiling_height, camera.eye_height, height as f32, scale),
                        y_min,
                        y_max,
                    );
                    let floor_y = clamp_coord(
                        project_y(sector.floor_height, camera.eye_height, height as f32, scale),
                        y_min,
                        y_max,
                    );

                    let ceiling_color = shade(material_color(sector.ceiling_material), sector.light_level, wall_depth);
                    let floor_color = shade(material_color(sector.floor_material), sector.light_level, wall_depth);

                    draw_column(framebuffer, width, x, y_min, ceiling_y, ceiling_color);
                    draw_column(framebuffer, width, x, floor_y, y_max, floor_color);

                    // u=0 au sommet `b` (t=0), u=1 au sommet `a` (t=1) - coherent
                    // avec la convention screen_left/screen_right utilisee partout
                    // ailleurs dans cette fonction.
                    let u = perspective_lerp(0.0, 1.0, cam_b.y, cam_a.y, t, wall_depth) * wall_length
                        / TEXTURE_WORLD_SCALE;
                    draw_wall_band(framebuffer, width, x, ceiling_y, floor_y, material, u, sector.light_level, wall_depth);
                }
            }
            WallKind::Portal { neighbor, step_material } => {
                let neighbor_sector = &level.sectors[neighbor];
                let mut next_y_top = Vec::with_capacity(x_end - x_start);
                let mut next_y_bottom = Vec::with_capacity(x_end - x_start);

                // meme raison que le #[allow] du bloc Solid ci-dessus.
                #[allow(clippy::needless_range_loop)]
                for x in x_start..x_end {
                    let (y_min, y_max) = bounds.y_range(x);
                    if y_min >= y_max {
                        next_y_top.push(y_min);
                        next_y_bottom.push(y_min); // colonne deja fermee : plage vide en aval aussi
                        continue;
                    }

                    let t = inverse_lerp(screen_left, screen_right, x as f32 + 0.5);
                    let wall_depth = perspective_depth(cam_b.y, cam_a.y, t);
                    let scale = projection_scale(wall_depth, camera.fov, width as f32);

                    let ceiling_y = clamp_coord(
                        project_y(sector.ceiling_height, camera.eye_height, height as f32, scale),
                        y_min,
                        y_max,
                    );
                    let floor_y = clamp_coord(
                        project_y(sector.floor_height, camera.eye_height, height as f32, scale),
                        y_min,
                        y_max,
                    );
                    let neighbor_ceiling_y = clamp_coord(
                        project_y(neighbor_sector.ceiling_height, camera.eye_height, height as f32, scale),
                        y_min,
                        y_max,
                    );
                    let neighbor_floor_y = clamp_coord(
                        project_y(neighbor_sector.floor_height, camera.eye_height, height as f32, scale),
                        y_min,
                        y_max,
                    );

                    // La fenetre ouverte du portail, POUR CETTE COLONNE, est
                    // bornee par le plus restrictif des deux secteurs sur
                    // chaque bord (plafond le plus bas, sol le plus haut).
                    // Calculee par colonne (pas un min/max global) pour que
                    // la porte se deforme correctement en perspective.
                    let opening_top = ceiling_y.max(neighbor_ceiling_y);
                    let opening_bottom = floor_y.min(neighbor_floor_y);

                    // Colonne sans ouverture reelle (porte fermee, marche qui
                    // bouche tout l'espace) : ca occulte les sprites au-dela
                    // exactement comme un mur plein, sinon un sprite dans le
                    // secteur voisin apparaitrait par-dessus une porte fermee
                    // (le z-buffer ne voit normalement que les murs `Solid`).
                    if opening_top >= opening_bottom {
                        depth_buffer[x] = depth_buffer[x].min(wall_depth);
                    }

                    let ceiling_color = shade(material_color(sector.ceiling_material), sector.light_level, wall_depth);
                    let step_color = shade(material_color(step_material), sector.light_level, wall_depth);
                    let floor_color = shade(material_color(sector.floor_material), sector.light_level, wall_depth);

                    draw_column(framebuffer, width, x, y_min, ceiling_y, ceiling_color);
                    draw_column(framebuffer, width, x, ceiling_y, opening_top, step_color);
                    draw_column(framebuffer, width, x, opening_bottom, floor_y, step_color);
                    draw_column(framebuffer, width, x, floor_y, y_max, floor_color);

                    next_y_top.push(opening_top);
                    next_y_bottom.push(opening_bottom);
                }

                let portal_bounds = ScreenBounds {
                    x_min: x_start,
                    x_max: x_end,
                    y_top: next_y_top,
                    y_bottom: next_y_bottom,
                };
                render_sector(ctx, neighbor, portal_bounds, framebuffer, depth_buffer, depth + 1);
            }
        }
    }
}

fn inverse_lerp(a: f32, b: f32, x: f32) -> f32 {
    ((x - a) / (b - a)).clamp(0.0, 1.0)
}

/// Interpolation correcte en perspective entre la profondeur a `t=0` et
/// celle a `t=1` : lineaire sur 1/profondeur, pas sur la profondeur
/// elle-meme (sinon les surfaces en biais se dessinent courbes).
fn perspective_depth(depth_at_0: f32, depth_at_1: f32, t: f32) -> f32 {
    let inv = (1.0 - t) / depth_at_0 + t / depth_at_1;
    1.0 / inv
}

/// Interpolation generique correcte en perspective (meme principe que
/// `perspective_depth`, applique a un attribut quelconque - ici la
/// coordonnee de texture le long du mur) : lineaire sur `attribut/profondeur`,
/// pas sur l'attribut lui-meme, sinon la texture se dessine deformee.
fn perspective_lerp(value_at_0: f32, value_at_1: f32, depth_at_0: f32, depth_at_1: f32, t: f32, depth_at_t: f32) -> f32 {
    let inv = (1.0 - t) * value_at_0 / depth_at_0 + t * value_at_1 / depth_at_1;
    inv * depth_at_t
}

fn clamp_coord(value: f32, min: usize, max: usize) -> usize {
    (value.round() as isize).clamp(min as isize, max as isize) as usize
}

fn draw_column(framebuffer: &mut [u32], width: usize, x: usize, y_start: usize, y_end: usize, color: u32) {
    for y in y_start..y_end {
        framebuffer[y * width + x] = color;
    }
}

/// Dessine la bande d'un mur (entre `y_top` et `y_bottom`) : echantillonne sa
/// texture ligne par ligne si le materiau en a une, sinon retombe sur un
/// simple remplissage en couleur plate (cas des materiaux sans texture).
#[allow(clippy::too_many_arguments)]
fn draw_wall_band(
    framebuffer: &mut [u32],
    width: usize,
    x: usize,
    y_top: usize,
    y_bottom: usize,
    material: MaterialId,
    u: f32,
    light_level: f32,
    depth: f32,
) {
    let Some(texture) = material_texture(material) else {
        let color = shade(material_color(material), light_level, depth);
        draw_column(framebuffer, width, x, y_top, y_bottom, color);
        return;
    };

    let band_height = (y_bottom.saturating_sub(y_top)).max(1) as f32;
    for y in y_top..y_bottom {
        let v = (y - y_top) as f32 / band_height;
        // Une texture de mur est toujours entierement opaque (teste dans
        // materials::texture) ; le repli n'est jamais vraiment exerce.
        let sampled = texture.sample(u, v).unwrap_or_else(|| material_color(material));
        framebuffer[y * width + x] = shade(sampled, light_level, depth);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::fixtures::single_room;
    use crate::render::render_level;
    use crate::Camera;
    use glam::Vec2;

    #[test]
    fn renders_ceiling_wall_floor_bands_facing_a_wall() {
        // Materiau 9 (sans texture associee) expres, pour verifier le
        // placement des bandes plafond/mur/sol independamment du sampling
        // de texture (teste separement ci-dessous).
        let vertices = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ];
        let sector = crate::level::Sector {
            walls: vec![
                crate::level::Wall { start: 0, end: 1, kind: WallKind::Solid { material: 9 } },
                crate::level::Wall { start: 1, end: 2, kind: WallKind::Solid { material: 9 } },
                crate::level::Wall { start: 2, end: 3, kind: WallKind::Solid { material: 9 } },
                crate::level::Wall { start: 3, end: 0, kind: WallKind::Solid { material: 9 } },
            ],
            floor_height: 0.0,
            ceiling_height: 2.0,
            light_level: 1.0,
            floor_material: 1,
            ceiling_material: 2,
            door: None,
        };
        let level = crate::level::Level::new(vertices, vec![sector], Vec::new());

        let camera = Camera {
            position: Vec2::new(5.0, 5.0),
            dir: Vec2::new(1.0, 0.0), // face au mur (10,0)-(10,10)
            eye_height: 1.0,
            fov: 66f32.to_radians(),
        };

        let (width, height) = (100, 100);
        let mut framebuffer = vec![0u32; width * height];
        render_level(&level, &camera, &mut framebuffer, width, height);

        let pixel_at = |row: usize| framebuffer[row * width + 50];

        // distance perpendiculaire au mur = 5.0 exactement (camera au centre
        // de la piece, face a un mur perpendiculaire a sa direction).
        let expected_ceiling = shade(material_color(2), 1.0, 5.0);
        let expected_wall = shade(material_color(9), 1.0, 5.0);
        let expected_floor = shade(material_color(1), 1.0, 5.0);

        assert_eq!(pixel_at(0), expected_ceiling, "plafond attendu en haut de l'ecran");
        assert_eq!(pixel_at(50), expected_wall, "mur attendu au centre vertical");
        assert_eq!(pixel_at(99), expected_floor, "sol attendu en bas de l'ecran");
    }

    #[test]
    fn textured_wall_varies_across_the_band_instead_of_a_flat_fill() {
        let level = single_room(); // murs en materiau 3 (brique, texture)
        let camera = Camera {
            position: Vec2::new(5.0, 5.0),
            dir: Vec2::new(1.0, 0.0),
            eye_height: 1.0,
            fov: 66f32.to_radians(),
        };

        let (width, height) = (100, 100);
        let mut framebuffer = vec![0u32; width * height];
        render_level(&level, &camera, &mut framebuffer, width, height);

        // Colonne au centre de l'ecran, uniquement les lignes du mur (pas
        // plafond/sol) : avec une texture, elles ne doivent pas etre toutes
        // identiques (contrairement a l'ancien remplissage en couleur plate).
        let wall_rows: Vec<u32> = (20..80).map(|y| framebuffer[y * width + 50]).collect();
        assert!(
            wall_rows.iter().any(|&px| px != wall_rows[0]),
            "un mur texture ne devrait pas etre une seule couleur plate"
        );
    }

    #[test]
    fn camera_outside_every_sector_renders_only_void() {
        let level = single_room();
        let camera = Camera::new(Vec2::new(-5.0, -5.0));

        let (width, height) = (20, 20);
        let mut framebuffer = vec![0u32; width * height];
        render_level(&level, &camera, &mut framebuffer, width, height);

        assert!(framebuffer.iter().all(|&px| px == super::super::VOID_COLOR));
    }

    #[test]
    fn portal_traversal_renders_the_neighbor_sector() {
        use crate::level::fixtures::demo_level;

        let level = demo_level();
        let camera = Camera {
            position: Vec2::new(8.0, 4.0), // dans la salle 0, face a la porte
            dir: Vec2::new(1.0, 0.0),
            eye_height: 1.0,
            fov: 66f32.to_radians(),
        };

        let (width, height) = (100, 100);
        let mut framebuffer = vec![0u32; width * height];
        render_level(&level, &camera, &mut framebuffer, width, height);

        // A travers la porte (colonne centrale), on doit voir de la vraie
        // geometrie (secteur 1), pas juste le vide de fond.
        let center_pixel = framebuffer[50 * width + 50];
        assert_ne!(center_pixel, super::super::VOID_COLOR, "la piece voisine doit etre visible a travers le portail");
    }

    #[test]
    fn portal_opening_is_not_a_flat_rectangle_when_viewed_at_an_angle() {
        // Regression : une premiere version aplatissait la fenetre de
        // recursion en un seul rectangle, donnant une porte aux bords
        // parfaitement horizontaux/verticaux quel que soit l'angle de vue.
        use crate::level::fixtures::demo_level;

        let level = demo_level();
        let camera = Camera {
            position: Vec2::new(7.0, 2.0),
            dir: Vec2::from_angle(18.4f32.to_radians()), // vise le montant de la porte en biais
            eye_height: 1.0,
            fov: 66f32.to_radians(),
        };

        let (width, height) = (200, 150);
        let mut framebuffer = vec![0u32; width * height];
        render_level(&level, &camera, &mut framebuffer, width, height);

        // Vue en biais : deux colonnes distinctes de la zone du portail ne
        // doivent pas montrer le meme decoupage vertical (ce qui arriverait
        // si l'ouverture etait un rectangle plat au lieu d'un trapeze en
        // perspective).
        let col_a: Vec<u32> = (0..height).map(|y| framebuffer[y * width + 90]).collect();
        let col_b: Vec<u32> = (0..height).map(|y| framebuffer[y * width + 150]).collect();
        assert_ne!(col_a, col_b, "deux colonnes distantes de la porte ne devraient pas etre identiques en vue de biais");
    }
}
