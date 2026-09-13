pub struct Map {
    width: usize,
    height: usize,
    tiles: Vec<u8>,
}

impl Map {
    pub fn from_layout(layout: &str) -> Self {
        let rows: Vec<&str> = layout.lines().filter(|l| !l.is_empty()).collect();
        let height = rows.len();
        let width = rows[0].len();

        let mut tiles = Vec::with_capacity(width * height);
        for row in &rows {
            for ch in row.chars() {
                tiles.push(if ch == '#' { 1 } else { 0 });
            }
        }

        Self { width, height, tiles }
    }

    pub fn is_wall(&self, x: i32, y: i32) -> bool {
        self.wall_id(x, y).map_or(true, |id| id != 0)
    }

    pub fn wall_id(&self, x: i32, y: i32) -> Option<u8> {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return None; // hors limites = mur, empêche de sortir de la carte
        }
        Some(self.tiles[y as usize * self.width + x as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_map() -> Map {
        Map::from_layout("###\n#.#\n###")
    }

    #[test]
    fn detects_wall() {
        let map = sample_map();
        assert!(map.is_wall(0, 0));
        assert!(!map.is_wall(1, 1));
    }

    #[test]
    fn out_of_bounds_is_a_wall() {
        let map = sample_map();
        assert!(map.is_wall(-1, 0));
        assert!(map.is_wall(100, 100));
    }
}
