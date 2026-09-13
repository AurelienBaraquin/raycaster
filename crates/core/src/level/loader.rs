use glam::Vec2;
use serde::Deserialize;

use super::{Level, Sector};
use crate::entity::Entity;

#[derive(Deserialize)]
struct LevelData {
    vertices: Vec<Vec2>,
    sectors: Vec<Sector>,
    #[serde(default)]
    entities: Vec<Entity>,
}

/// Parse un niveau au format RON. Panique si le texte est invalide : un
/// niveau embarque au build (`include_str!`) qui ne parse pas est une erreur
/// de programmation, pas une entree utilisateur a valider en douceur.
pub fn load(source: &str) -> Level {
    let data: LevelData = ron::from_str(source).expect("niveau RON invalide");
    Level::new(data.vertices, data.sectors, data.entities)
}
