use serde::Deserialize;

use super::VertexId;

/// Identifiant de materiau. Pour l'instant mappe vers une couleur plate
/// (voir `materials.rs`) ; une vraie texture pourra reutiliser le meme id plus tard.
pub type MaterialId = u8;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub enum WallKind {
    Solid { material: MaterialId },
    /// `step_material` habille la "marche" visible quand le secteur voisin a
    /// un sol plus haut ou un plafond plus bas (la bande verticale entre les
    /// deux, visible au niveau du portail lui-meme).
    Portal { neighbor: usize, step_material: MaterialId },
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Wall {
    pub start: VertexId,
    pub end: VertexId,
    pub kind: WallKind,
}

/// Un secteur est un polygone CONVEXE (sommets dans l'ordre anti-horaire,
/// l'interieur est toujours a gauche de chaque mur dans cet ordre). Toute la
/// geometrie du niveau vit hors des secteurs : l'espace hors de tout secteur
/// est implicitement solide/infranchissable.
#[derive(Debug, Clone, Deserialize)]
pub struct Sector {
    pub walls: Vec<Wall>,
    pub floor_height: f32,
    /// Si `door` est present, c'est la hauteur de plafond FERMEE de depart
    /// (typiquement egale a `floor_height`, pour boucher completement le
    /// passage) ; le runtime (`crate::door::Door`) l'anime ensuite vers
    /// `door.open_height` selon la proximite du joueur.
    pub ceiling_height: f32,
    /// Luminosite ambiante du secteur, 0.0 (noir) a 1.0 (pleine luminosite).
    pub light_level: f32,
    pub floor_material: MaterialId,
    pub ceiling_material: MaterialId,
    /// Fait de ce secteur une porte animee (plafond qui monte/descend selon
    /// la proximite du joueur) plutot qu'un secteur statique.
    #[serde(default)]
    pub door: Option<DoorDef>,
}

/// Configuration statique (donnee de niveau) d'une porte. L'etat d'animation
/// courant (ouverte/fermee/en mouvement) vit dans `crate::door::Door`, pas ici
/// - `Sector` reste la description du niveau, pas l'etat d'execution.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct DoorDef {
    pub open_height: f32,
    /// Distance a laquelle le joueur declenche l'ouverture.
    pub trigger_radius: f32,
}
