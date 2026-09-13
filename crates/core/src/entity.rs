use glam::Vec2;
use serde::Deserialize;

use crate::materials::SpriteKind;

/// Un objet statique place dans le niveau (tonneau, decor...), rendu comme
/// un billboard (toujours face a la camera) plutot que comme de la geometrie
/// de secteur. Pas de comportement/logique pour l'instant - juste du decor.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Entity {
    /// Position au sol (au centre de la base du sprite).
    pub position: Vec2,
    /// Hauteur monde de la base du sprite (le sol du secteur ou il se trouve).
    pub base_height: f32,
    /// Demi-largeur monde du sprite.
    pub half_width: f32,
    /// Hauteur monde du sprite.
    pub height: f32,
    pub sprite: SpriteKind,
}
