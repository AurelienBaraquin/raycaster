/// Une texture carree en memoire : pixels `0x00RRGGBB` (ou `None` = pixel
/// transparent, pour les sprites), echantillonnee par coordonnees UV
/// normalisees (0.0..1.0, avec bouclage/tiling au-dela).
pub struct Texture {
    width: usize,
    height: usize,
    pixels: Vec<Option<u32>>,
}

impl Texture {
    fn from_fn(width: usize, height: usize, mut pixel_at: impl FnMut(usize, usize) -> Option<u32>) -> Self {
        let mut pixels = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                pixels.push(pixel_at(x, y));
            }
        }
        Self { width, height, pixels }
    }

    /// Echantillonne la texture a des coordonnees UV quelconques : `u`/`v`
    /// bouclent automatiquement (tiling), donc une valeur hors de 0.0..1.0
    /// est parfaitement valide (c'est meme le but, pour repeter la texture
    /// le long d'un mur). `None` = pixel transparent (utilise par les sprites,
    /// jamais produit par les textures de mur qui remplissent tout le carre).
    pub fn sample(&self, u: f32, v: f32) -> Option<u32> {
        let x = (u.rem_euclid(1.0) * self.width as f32) as usize;
        let y = (v.rem_euclid(1.0) * self.height as f32) as usize;
        let x = x.min(self.width - 1);
        let y = y.min(self.height - 1);
        self.pixels[y * self.width + x]
    }
}

const SIZE: usize = 32;

/// Motif de briques : rangees de briques avec joints de mortier, rangees
/// alternees decalees d'une demi-brique (appareillage classique).
pub fn brick() -> Texture {
    const BRICK_HEIGHT: usize = 8;
    const BRICK_WIDTH: usize = 16;
    const MORTAR: u32 = 0x00_80_78_70;
    const BRICK: u32 = 0x00_A8_50_38;

    Texture::from_fn(SIZE, SIZE, |x, y| {
        let row = y / BRICK_HEIGHT;
        let offset = if row.is_multiple_of(2) { 0 } else { BRICK_WIDTH / 2 };
        let brick_x = (x + offset) % SIZE;

        let on_horizontal_joint = y % BRICK_HEIGHT == 0;
        let on_vertical_joint = brick_x.is_multiple_of(BRICK_WIDTH);
        Some(if on_horizontal_joint || on_vertical_joint { MORTAR } else { BRICK })
    })
}

/// Motif de carrelage : dalles carrees avec un fin joint clair.
pub fn tile() -> Texture {
    const TILE_SIZE: usize = 16;
    const JOINT: u32 = 0x00_C8_C8_D0;
    const SLAB: u32 = 0x00_88_90_98;

    Texture::from_fn(SIZE, SIZE, |x, y| {
        let on_joint = x % TILE_SIZE == 0 || y % TILE_SIZE == 0;
        Some(if on_joint { JOINT } else { SLAB })
    })
}

/// Sprite : un tonneau en bois vu de face, silhouette ovale (le reste du
/// carre est transparent) avec des cerclages et un ombrage cylindrique simple
/// (plus sombre sur les bords, plus clair au centre).
pub fn barrel() -> Texture {
    const CENTER_X: f32 = SIZE as f32 / 2.0;
    const RADIUS: f32 = SIZE as f32 * 0.36;
    const TOP_MARGIN: usize = 2;
    const WOOD: u32 = 0x00_8B_5A_2B;
    const WOOD_DARK: u32 = 0x00_5C_3A_1B;
    const HOOP: u32 = 0x00_30_28_20;

    Texture::from_fn(SIZE, SIZE, |x, y| {
        if !(TOP_MARGIN..SIZE - TOP_MARGIN).contains(&y) {
            return None;
        }

        let dx = (x as f32 + 0.5) - CENTER_X;
        if dx.abs() > RADIUS {
            return None;
        }

        let edge_factor = dx.abs() / RADIUS; // 0.0 au centre, 1.0 au bord
        let is_hoop = (y - TOP_MARGIN) % 10 < 2;

        Some(if is_hoop {
            HOOP
        } else if edge_factor > 0.7 {
            WOOD_DARK
        } else {
            WOOD
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_wraps_past_the_edge() {
        let texture = brick();
        // u/v=1.0 doit boucler sur le meme pixel que 0.0, pas paniquer/deborder.
        assert_eq!(texture.sample(1.0, 1.0), texture.sample(0.0, 0.0));
    }

    #[test]
    fn sample_handles_negative_and_large_coordinates() {
        let texture = tile();
        // ne doit pas paniquer sur des UV hors [0,1] (tiling le long d'un mur long)
        let _ = texture.sample(-3.7, 12.25);
    }

    #[test]
    fn brick_and_tile_are_visually_distinct() {
        // patrons differents -> au moins un pixel doit differer entre les deux
        let brick = brick();
        let tile = tile();
        let differs = (0..SIZE)
            .any(|i| brick.sample(i as f32 / SIZE as f32, 0.0) != tile.sample(i as f32 / SIZE as f32, 0.0));
        assert!(differs, "brick() et tile() ne devraient pas etre des textures identiques");
    }

    #[test]
    fn wall_textures_are_fully_opaque() {
        for texture in [brick(), tile()] {
            for i in 0..SIZE {
                for j in 0..SIZE {
                    let u = i as f32 / SIZE as f32;
                    let v = j as f32 / SIZE as f32;
                    assert!(texture.sample(u, v).is_some(), "une texture de mur ne doit avoir aucun pixel transparent");
                }
            }
        }
    }

    #[test]
    fn barrel_has_transparent_corners_and_opaque_center() {
        let barrel = barrel();
        assert_eq!(barrel.sample(0.02, 0.02), None, "le coin doit etre transparent (hors silhouette)");
        assert!(barrel.sample(0.5, 0.5).is_some(), "le centre du tonneau doit etre opaque");
    }
}
