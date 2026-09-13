/// Distance (en unites monde) a partir de laquelle le brouillard de distance
/// atteint son assombrissement maximal.
const FOG_DISTANCE: f32 = 20.0;
/// Facteur de luminosite minimal impose par le brouillard, meme tres loin
/// (evite un noir total qui masquerait completement la geometrie lointaine).
const FOG_FLOOR: f32 = 0.2;

/// Assombrit une couleur `0x00RRGGBB` selon la luminosite ambiante du secteur
/// (`light_level`, 0.0..1.0) et un brouillard de distance simple (plus loin
/// de la camera -> plus sombre). Fonction pure, independante de tout etat de
/// rendu : facile a tester isolement.
pub fn shade(color: u32, light_level: f32, distance: f32) -> u32 {
    let fog = (1.0 - distance / FOG_DISTANCE).clamp(FOG_FLOOR, 1.0);
    let factor = (light_level * fog).clamp(0.0, 1.0);

    let r = (((color >> 16) & 0xFF) as f32 * factor) as u32;
    let g = (((color >> 8) & 0xFF) as f32 * factor) as u32;
    let b = ((color & 0xFF) as f32 * factor) as u32;

    (r << 16) | (g << 8) | b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_light_and_zero_distance_is_unchanged() {
        assert_eq!(shade(0x00_A0_A0_A0, 1.0, 0.0), 0x00_A0_A0_A0);
    }

    #[test]
    fn half_light_level_halves_each_channel() {
        // 0xA0 = 160 ; 160 * 0.5 = 80 = 0x50 (troncature exacte, pas d'arrondi surprise)
        assert_eq!(shade(0x00_A0_A0_A0, 0.5, 0.0), 0x00_50_50_50);
    }

    #[test]
    fn distant_geometry_never_goes_fully_black() {
        let shaded = shade(0x00_A0_A0_A0, 1.0, 1000.0);
        assert!(shaded > 0, "le plancher de brouillard doit laisser un peu de lumiere");
    }
}
