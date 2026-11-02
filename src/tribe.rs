use macroquad::prelude::*;
use crate::terrain::TerrainConfig;
use macroquad::rand::gen_range;


pub const TRIBE_COLORS: [Color; 4] = [
    Color::new(0.922, 0.294, 0.275, 1.0),
    Color::new(0.275, 0.784, 0.882, 1.0),
    Color::new(0.941, 0.784, 0.255, 1.0),
    Color::new(0.804, 0.373, 0.784, 1.0)
];

// Tyle krokow wrogowie musza stac w bazie bez obroncow, zeby ja zdobyc
const BASE_FALL_STEPS: u32 = 600;

pub struct Tribe {
    color: Color,
    base_min: (usize, usize),
    base_max: (usize, usize),
    siege_progress: u32,
    besieged: bool,
    extinct: bool
}

impl Tribe {
    pub fn new(color: Color, base_position: (usize, usize), config: &TerrainConfig) -> Tribe {
        let size = config.base_size;
        let (bx, by) = base_position;

        let min_x = bx.min(config.cols - size);
        let min_y = by.min(config.rows - size);

        let max_x = min_x + size - 1;
        let max_y = min_y + size - 1;

        Tribe {
            color,
            base_min: (min_x, min_y),
            base_max: (max_x, max_y),
            siege_progress: 0,
            besieged: false,
            extinct: false
        }
    }

    pub fn create_tribes(config: &TerrainConfig) -> Vec<Tribe> {
        vec![
            Tribe::new(TRIBE_COLORS[0], (0, 0), config),
            Tribe::new(TRIBE_COLORS[1], (config.cols - 1, 0), config),
            Tribe::new(TRIBE_COLORS[2], (0, config.rows - 1), config),
            Tribe::new(TRIBE_COLORS[3], (config.rows - 1, config.cols - 1), config)
        ]
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn base_center(&self, tile_size: f32) -> (f32, f32) {
        let x = (self.base_min.0 + self.base_max.0 + 1) as f32 * tile_size / 2.0;
        let y = (self.base_min.1 + self.base_max.1 + 1) as f32 * tile_size / 2.0;
        (x, y)
    }

    pub fn random_base_tile(&self) -> (usize, usize) {
        let x = gen_range(self.base_min.0, self.base_max.0 + 1);
        let y = gen_range(self.base_min.1, self.base_max.1 + 1);
        (x, y)
    }

    pub fn base_rect(&self, tile_size: f32) -> Rect {
        let width = (self.base_max.0 - self.base_min.0 + 1) as f32 * tile_size;
        let height = (self.base_max.1 - self.base_min.1 + 1) as f32 * tile_size;
        Rect::new(self.base_min.0 as f32 * tile_size, self.base_min.1 as f32 * tile_size, width, height)
    }

    // Wrogowie bez obroncow w bazie zblizaja ja do upadku, a w kazdej innej sytuacji
    // postep oblezenia spada. Zwraca true, gdy baza wlasnie upadla.
    pub fn update_siege(&mut self, enemies_in_base: bool, defenders_in_base: bool) -> bool {
        if self.extinct {
            return false
        }

        self.besieged = enemies_in_base && !defenders_in_base;
        if self.besieged {
            self.siege_progress += 1;
        } else {
            self.siege_progress = self.siege_progress.saturating_sub(1);
        }

        if self.siege_progress >= BASE_FALL_STEPS {
            self.fall();
            return true
        }
        false
    }

    pub fn fall(&mut self) {
        self.extinct = true;
        self.besieged = false;
        self.siege_progress = 0;
    }

    // Nowa runda przywraca plemie do gry
    pub fn restore(&mut self) {
        self.extinct = false;
        self.besieged = false;
        self.siege_progress = 0;
    }

    pub fn is_extinct(&self) -> bool {
        self.extinct
    }

    pub fn is_besieged(&self) -> bool {
        self.besieged
    }

    pub fn siege_ratio(&self) -> f32 {
        self.siege_progress as f32 / BASE_FALL_STEPS as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::Terrain;

    fn tribe() -> Tribe {
        Tribe::create_tribes(Terrain::new(10, 10, 25.0, 2, 1).config()).remove(0)
    }

    #[test]
    fn base_falls_after_enough_steps_without_defenders() {
        let mut tribe = tribe();
        for _ in 0..BASE_FALL_STEPS - 1 {
            assert!(!tribe.update_siege(true, false));
        }

        assert!(tribe.update_siege(true, false));
        assert!(tribe.is_extinct());
    }

    #[test]
    fn defender_in_base_pushes_the_siege_back() {
        let mut tribe = tribe();
        for _ in 0..10 {
            tribe.update_siege(true, false);
        }
        for _ in 0..4 {
            tribe.update_siege(true, true);
        }

        assert!(!tribe.is_besieged());
        assert_eq!(tribe.siege_ratio(), 6.0 / BASE_FALL_STEPS as f32);
    }

    #[test]
    fn restored_tribe_is_back_in_the_game() {
        let mut tribe = tribe();
        tribe.fall();
        assert!(!tribe.update_siege(true, false));

        tribe.restore();
        assert!(!tribe.is_extinct());
    }
}