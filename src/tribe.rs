use macroquad::prelude::*;
use crate::terrain::TerrainConfig;
use macroquad::rand::gen_range;


pub const TRIBE_COLORS: [Color; 4] = [
    Color::new(0.922, 0.294, 0.275, 1.0),
    Color::new(0.275, 0.784, 0.882, 1.0),
    Color::new(0.941, 0.784, 0.255, 1.0),
    Color::new(0.804, 0.373, 0.784, 1.0)
];

pub struct Tribe {
    color: Color,
    base_min: (usize, usize),
    base_max: (usize, usize)
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
        }   
    }

    pub fn create_tribes(config: &TerrainConfig) -> Vec<Tribe> {
        vec![
            Tribe::new(TRIBE_COLORS[0], (0, 0), config),
            Tribe::new(TRIBE_COLORS[1], (0, config.cols - 1), config),
            Tribe::new(TRIBE_COLORS[2], (config.rows - 1, 0), config),
            Tribe::new(TRIBE_COLORS[3], (config.rows - 1, config.cols - 1), config)
        ]
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn random_base_tile(&self) -> (usize, usize) {
        let x = gen_range(self.base_min.0, self.base_max.0 + 1);
        let y = gen_range(self.base_min.1, self.base_max.1 + 1);
        (x, y)
    }
}