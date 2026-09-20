use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq)]

enum TileType{
    Grass,
    Water,
    Sand,
    Rock,
    Base(u8)
}

impl TileType{
    fn get_color(&self) -> Color{
        match self{
            TileType::Grass => Color::from_rgba(62, 102, 62, 255),
            TileType::Water => Color::from_rgba(38, 56, 84, 255),
            TileType::Sand => Color::from_rgba(150, 134, 96, 255),
            TileType::Rock => Color::from_rgba(88, 88, 92, 255),
            TileType::Base(tribe) => {
                match tribe{
                    0 => Color::from_rgba(235, 75, 70, 255),
                    1 => Color::from_rgba(70, 200, 225, 255),
                    2 => Color::from_rgba(240, 200, 65, 255),
                    3 => Color::from_rgba(205, 95, 200, 255),
                    _ => Color::from_rgba(255, 255, 255, 255)
                }
            }
        }
    }
}

pub struct Terrain{
    rows: usize,
    cols: usize,
    tile_size: f32,
    tiles: Vec<Vec<TileType>>
}

impl Terrain{
    pub fn new(rows: usize, cols: usize, tile_size: f32, base_size: usize) -> Terrain{
        let mut tiles = Vec::new();
        for _ in 0..rows{
            let mut row = Vec::new();
            for _ in 0..cols{
                let tile_type = TileType::Grass;
                row.push(tile_type);
            }
            tiles.push(row);
        }

        for i in 0..base_size{
            for j in 0..base_size{
                tiles[i][j] = TileType::Base(0);
                tiles[i][cols - 1 - j] = TileType::Base(1);
                tiles[rows - 1 - i][j] = TileType::Base(2);
                tiles[rows - 1 - i][cols - 1 - j] = TileType::Base(3);
            }
        }

        Terrain{
            rows,
            cols,
            tile_size,
            tiles,
        }
    }

    pub fn draw(&self){
        for row in 0..self.rows{
            for col in 0..self.cols{
                let tile_type = self.tiles[row][col];
                let color = tile_type.get_color();
                let x = col as f32 * self.tile_size;
                let y = row as f32 * self.tile_size;
                draw_rectangle(x, y, self.tile_size, self.tile_size, color);
                draw_rectangle_lines(x, y, self.tile_size, self.tile_size, 1.0, BLACK);
            }
        }
    }

    pub fn get_width(&self) -> f32{
        self.cols as f32 * self.tile_size
    }

    pub fn get_height(&self) -> f32{
        self.rows as f32 * self.tile_size
    }
}