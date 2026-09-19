use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq)]

enum TileType{
    Grass,
    Water,
    Sand,
    Rock
}

impl TileType{
    fn get_color(&self) -> Color{
        match self{
            TileType::Grass => GREEN,
            TileType::Water => BLUE,
            TileType::Sand => YELLOW,
            TileType::Rock => GRAY
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
    pub fn new(rows: usize, cols: usize, tile_size: f32) -> Terrain{
        let mut tiles = Vec::new();
        for _ in 0..rows{
            let mut row = Vec::new();
            for _ in 0..cols{
                let tile_type = TileType::Grass;
                row.push(tile_type);
            }
            tiles.push(row);
        }

        Terrain{
            rows,
            cols,
            tile_size,
            tiles
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