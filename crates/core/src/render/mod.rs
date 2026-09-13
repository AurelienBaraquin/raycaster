mod clip;
mod project;
mod sector_render;
mod sprite_render;
mod transform;

use crate::level::Level;
use crate::Camera;

/// Couleur de ce qui n'appartient a aucun secteur (void / hors niveau).
const VOID_COLOR: u32 = 0x00_10_10_18;

/// Regroupe ce qui ne change jamais pendant un rendu (niveau, camera, taille
/// ecran) - partage entre la passe geometrie (secteurs/portails) et la passe
/// sprites, pour eviter de trainer 4 parametres separes partout.
#[derive(Clone, Copy)]
struct RenderContext<'a> {
    level: &'a Level,
    camera: &'a Camera,
    width: usize,
    height: usize,
}

/// Rendu complet d'une scene, en deux passes :
/// 1. La geometrie des secteurs (transformation camera + projection
///    perspective + clipping plan proche + traversee recursive de portails),
///    qui remplit au passage un z-buffer par colonne (profondeur du mur plein
///    le plus proche).
/// 2. Les sprites (billboards toujours face camera), tries du plus loin au
///    plus proche, occultes colonne par colonne via ce z-buffer.
pub fn render_level(level: &Level, camera: &Camera, framebuffer: &mut [u32], width: usize, height: usize) {
    framebuffer.fill(VOID_COLOR);

    let Some(start_sector) = level.sector_containing(camera.position) else {
        return; // camera hors de tout secteur : rien a dessiner
    };

    let ctx = RenderContext { level, camera, width, height };
    let mut depth_buffer = vec![f32::INFINITY; width];
    sector_render::render_geometry(ctx, start_sector, framebuffer, &mut depth_buffer);
    sprite_render::render_sprites(ctx, &depth_buffer, framebuffer);
}
