use macroquad::prelude::*;


pub const TRIBE_COLORS: [Color; 4] = [
    Color::new(0.922, 0.294, 0.275, 1.0),
    Color::new(0.275, 0.784, 0.882, 1.0),
    Color::new(0.941, 0.784, 0.255, 1.0),
    Color::new(0.804, 0.373, 0.784, 1.0)
];

pub struct Tribe {
    color: Color,
    base_position: (usize, usize)
}

impl Tribe {
    pub fn new(color: Color, base_position: (usize, usize)) -> Tribe {
        Tribe {
            color,
            base_position
        }
    }

    pub fn create_tribes(rows: usize, cols: usize) -> Vec<Tribe> {
        vec![
            Tribe::new(TRIBE_COLORS[0], (0, 0)),
            Tribe::new(TRIBE_COLORS[1], (0, cols - 1)),
            Tribe::new(TRIBE_COLORS[2], (rows - 1, 0)),
            Tribe::new(TRIBE_COLORS[3], (rows - 1, cols - 1))
        ]
    }

    pub fn get_color(&self) -> Color {
        self.color
    }
}