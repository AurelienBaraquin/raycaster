use glam::Vec2;

use crate::level::{Level, SectorId};

/// Vitesse d'ouverture/fermeture (fraction 0.0..1.0 par seconde).
const DOOR_SPEED: f32 = 1.5;
/// Temps que la porte reste ouverte une fois le joueur hors de portee, avant
/// de commencer a se refermer.
const STAY_OPEN_SECONDS: f32 = 2.0;

/// Etat d'execution d'une porte : anime `ceiling_height` du secteur associe
/// entre sa hauteur fermee (figee dans les donnees du niveau) et sa hauteur
/// ouverte, selon la proximite du joueur. La configuration statique
/// (`DoorDef`) vit dans `level::Sector` ; cette struct est le runtime mutable
/// construit a partir de cette config (voir `World::new`).
pub struct Door {
    sector_id: SectorId,
    trigger_point: Vec2,
    trigger_radius: f32,
    closed_height: f32,
    open_height: f32,
    /// 0.0 = completement fermee, 1.0 = completement ouverte.
    openness: f32,
    /// Temps restant avant de commencer a se refermer, une fois le joueur
    /// hors de la zone de declenchement.
    stay_open_timer: f32,
}

impl Door {
    pub fn new(sector_id: SectorId, trigger_point: Vec2, trigger_radius: f32, closed_height: f32, open_height: f32) -> Self {
        Self {
            sector_id,
            trigger_point,
            trigger_radius,
            closed_height,
            open_height,
            openness: 0.0,
            stay_open_timer: 0.0,
        }
    }

    /// Fait avancer l'animation d'un pas de temps `dt` et ecrit la hauteur de
    /// plafond resultante directement dans le secteur du niveau. `level` est
    /// la source de verite courante (l'etat "actuel" du jeu), c'est pour ca
    /// qu'on y ecrit plutot que de garder la hauteur dans `Door` elle-meme.
    pub fn update(&mut self, level: &mut Level, player_position: Vec2, dt: f32) {
        let near = player_position.distance_squared(self.trigger_point) <= self.trigger_radius * self.trigger_radius;

        if near {
            self.stay_open_timer = STAY_OPEN_SECONDS;
        } else {
            self.stay_open_timer = (self.stay_open_timer - dt).max(0.0);
        }

        let target = if self.stay_open_timer > 0.0 { 1.0 } else { 0.0 };
        let max_step = DOOR_SPEED * dt;
        self.openness = if self.openness < target {
            (self.openness + max_step).min(target)
        } else {
            (self.openness - max_step).max(target)
        };

        level.sectors[self.sector_id].ceiling_height =
            self.closed_height + (self.open_height - self.closed_height) * self.openness;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::fixtures::single_room;

    #[test]
    fn opens_when_player_is_within_trigger_radius() {
        let mut level = single_room();
        let mut door = Door::new(0, Vec2::new(5.0, 5.0), 2.0, 0.0, 2.0);

        door.update(&mut level, Vec2::new(5.0, 5.0), 1.0); // joueur juste au centre du declencheur

        assert!(level.sectors[0].ceiling_height > 0.0, "la porte doit avoir commence a s'ouvrir");
    }

    #[test]
    fn stays_closed_when_player_is_far() {
        let mut level = single_room();
        let mut door = Door::new(0, Vec2::new(5.0, 5.0), 2.0, 0.0, 2.0);

        door.update(&mut level, Vec2::new(500.0, 500.0), 1.0);

        assert_eq!(level.sectors[0].ceiling_height, 0.0, "la porte ne doit pas s'ouvrir si le joueur est loin");
    }

    #[test]
    fn fully_opens_given_enough_time_near_the_trigger() {
        let mut level = single_room();
        let mut door = Door::new(0, Vec2::new(5.0, 5.0), 2.0, 0.0, 2.0);

        for _ in 0..20 {
            door.update(&mut level, Vec2::new(5.0, 5.0), 0.5);
        }

        approx::assert_relative_eq!(level.sectors[0].ceiling_height, 2.0, epsilon = 1e-4);
    }

    #[test]
    fn closes_again_after_the_player_leaves_and_the_stay_open_timer_expires() {
        let mut level = single_room();
        let mut door = Door::new(0, Vec2::new(5.0, 5.0), 2.0, 0.0, 2.0);

        // ouvre completement...
        for _ in 0..10 {
            door.update(&mut level, Vec2::new(5.0, 5.0), 1.0);
        }
        assert!(level.sectors[0].ceiling_height > 1.9);

        // ...le joueur s'eloigne, la porte doit finir par se refermer
        for _ in 0..20 {
            door.update(&mut level, Vec2::new(500.0, 500.0), 1.0);
        }
        approx::assert_relative_eq!(level.sectors[0].ceiling_height, 0.0, epsilon = 1e-4);
    }
}
