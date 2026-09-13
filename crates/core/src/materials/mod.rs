mod texture;

pub use texture::Texture;

use crate::level::MaterialId;
use serde::Deserialize;
use std::sync::OnceLock;

/// Quel sprite dessiner pour un `Entity` (voir `crate::entity`). Un seul
/// modele pour l'instant (`Barrel`) ; en ajouter un nouveau = une variante
/// ici + un generateur de texture dans `texture.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum SpriteKind {
    Barrel,
}

/// Texture (avec transparence) associee a un type de sprite.
pub fn sprite_texture(kind: SpriteKind) -> &'static Texture {
    match kind {
        SpriteKind::Barrel => barrel_texture(),
    }
}

fn barrel_texture() -> &'static Texture {
    static TEXTURE: OnceLock<Texture> = OnceLock::new();
    TEXTURE.get_or_init(texture::barrel)
}

/// Palette couleur de repli par materiau, utilisee quand aucune texture n'est
/// definie (sol/plafond pour l'instant) et comme couleur "moyenne" si jamais
/// besoin. L'id `0`/tout id inconnu retombe sur du magenta, tres visible expres.
pub fn material_color(material: MaterialId) -> u32 {
    match material {
        1 => 0x00_A0_A0_A0, // sol, gris clair
        2 => 0x00_50_50_50, // plafond, gris moyen
        3 => 0x00_C0_C0_C0, // murs (repli si jamais la texture n'est pas trouvee)
        4 => 0x00_90_90_98, // murs alternatifs (idem)
        _ => 0x00_FF_00_FF, // magenta = materiau non reconnu, tres visible expres
    }
}

/// Texture pixel associee a un materiau, si elle existe. `None` = ce
/// materiau reste en couleur plate (`material_color`) - c'est le cas de
/// tous les sols/plafonds pour l'instant (texturage plafond/sol = backlog,
/// l'algorithme de projection est different de celui des murs).
pub fn material_texture(material: MaterialId) -> Option<&'static Texture> {
    match material {
        3 => Some(brick_texture()),
        4 => Some(tile_texture()),
        _ => None,
    }
}

fn brick_texture() -> &'static Texture {
    static TEXTURE: OnceLock<Texture> = OnceLock::new();
    TEXTURE.get_or_init(texture::brick)
}

fn tile_texture() -> &'static Texture {
    static TEXTURE: OnceLock<Texture> = OnceLock::new();
    TEXTURE.get_or_init(texture::tile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_material_is_obviously_visible() {
        assert_eq!(material_color(99), 0x00_FF_00_FF);
    }

    #[test]
    fn materials_1_and_2_have_no_texture() {
        assert!(material_texture(1).is_none());
        assert!(material_texture(2).is_none());
    }

    #[test]
    fn materials_3_and_4_have_distinct_textures() {
        let wall_texture = material_texture(3).expect("materiau 3 doit avoir une texture");
        let alt_texture = material_texture(4).expect("materiau 4 doit avoir une texture");
        assert_ne!(wall_texture.sample(0.1, 0.1), alt_texture.sample(0.1, 0.1));
    }
}
