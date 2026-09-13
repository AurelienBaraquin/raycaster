mod loader;
mod sector;

pub use sector::{DoorDef, MaterialId, Sector, Wall, WallKind};

use crate::entity::Entity;
use glam::Vec2;

pub type VertexId = usize;
pub type SectorId = usize;

#[derive(Debug, Clone)]
pub struct Level {
    pub vertices: Vec<Vec2>,
    pub sectors: Vec<Sector>,
    pub entities: Vec<Entity>,
}

impl Level {
    pub fn new(vertices: Vec<Vec2>, sectors: Vec<Sector>, entities: Vec<Entity>) -> Self {
        Self { vertices, sectors, entities }
    }

    pub fn load_ron(source: &str) -> Self {
        loader::load(source)
    }

    /// Le secteur convexe contenant ce point, s'il y en a un. `None` = point
    /// hors de tout secteur, donc en zone solide/infranchissable.
    pub fn sector_containing(&self, point: Vec2) -> Option<SectorId> {
        self.sectors
            .iter()
            .position(|sector| self.contains_point(sector, point))
    }

    fn contains_point(&self, sector: &Sector, point: Vec2) -> bool {
        // Point dans un polygone convexe : il doit etre du cote "interieur"
        // (gauche, convention CCW) de CHAQUE mur du secteur.
        sector.walls.iter().all(|wall| {
            let a = self.vertices[wall.start];
            let b = self.vertices[wall.end];
            (b - a).perp_dot(point - a) >= 0.0
        })
    }

    /// Centre approximatif d'un secteur (moyenne de ses sommets). Utilise
    /// par `crate::door` comme point de reference pour la distance de
    /// declenchement d'une porte.
    pub fn sector_centroid(&self, sector_id: SectorId) -> Vec2 {
        let sector = &self.sectors[sector_id];
        let sum = sector
            .walls
            .iter()
            .fold(Vec2::ZERO, |acc, wall| acc + self.vertices[wall.start]);
        sum / sector.walls.len() as f32
    }
}

/// Niveau de demonstration embarque au build (`include_str!`, donc compatible
/// wasm sans fetch async). Utilise par `native`, `web`, et les tests.
pub fn demo_level() -> Level {
    Level::load_ron(include_str!("../../assets/levels/demo.ron"))
}

#[cfg(test)]
pub(crate) mod fixtures {
    use super::{demo_level as demo_level_impl, Level, Sector, Wall, WallKind};
    use glam::Vec2;

    /// Le niveau de demo (grande salle + petite piece surelevee reliees par
    /// un portail), partage par les tests de `level`, `render` et `world`.
    pub(crate) fn demo_level() -> Level {
        demo_level_impl()
    }

    /// Une seule piece carree 10x10 (CCW), sans portail. Utile pour tester
    /// la geometrie/le rendu/les collisions isolement, sans la complexite
    /// des portails.
    pub(crate) fn single_room() -> Level {
        let vertices = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ];
        let sector = Sector {
            walls: vec![
                Wall { start: 0, end: 1, kind: WallKind::Solid { material: 3 } },
                Wall { start: 1, end: 2, kind: WallKind::Solid { material: 3 } },
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
        Level::new(vertices, vec![sector], Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::demo_level;
    use glam::Vec2;

    #[test]
    fn loads_three_sectors() {
        // 2 salles + 1 secteur-porte entre les deux.
        let level = demo_level();
        assert_eq!(level.sectors.len(), 3);
        assert_eq!(level.vertices.len(), 10);
    }

    #[test]
    fn finds_point_in_main_room() {
        let level = demo_level();
        assert_eq!(level.sector_containing(Vec2::new(5.0, 4.0)), Some(0));
    }

    #[test]
    fn finds_point_in_small_room() {
        let level = demo_level();
        assert_eq!(level.sector_containing(Vec2::new(12.0, 4.0)), Some(1));
    }

    #[test]
    fn point_outside_every_sector_is_none() {
        let level = demo_level();
        assert_eq!(level.sector_containing(Vec2::new(-1.0, -1.0)), None);
    }
}
