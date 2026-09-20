use macroquad::prelude::*;
use noise::{NoiseFn, Perlin, Seedable, core::{perlin, value}};

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

    fn is_walkable(&self) -> bool{
        match self{
            TileType::Rock => false,
            _ => true
        }
    }
}

pub struct Terrain{
    rows: usize,
    cols: usize,
    tile_size: f32,
    base_size: usize,
    tiles: Vec<Vec<TileType>>
}

impl Terrain{
    pub fn new(rows: usize, cols: usize, tile_size: f32, base_size: usize) -> Terrain{

        let tiles = Terrain::generate(rows, cols, base_size, 234234);

        Terrain{
            rows,
            cols,
            tile_size,
            base_size,
            tiles
        }
    }

    fn generate(rows: usize, cols: usize, base_size: usize, seed: u32) -> Vec<Vec<TileType>>{
        let perlin = Perlin::new(seed);

        
        let mut tiles = Vec::new();
        for i in 0..rows{
            let mut row = Vec::new();
            for j in 0..cols{
                let value = perlin.get([j as f64 * 0.05, i as f64 * 0.05]);
                let tile_type = if value < -0.3{
                    TileType::Water
                }
                else if value < -0.15{
                    TileType::Sand
                }
                else if value < 0.5{
                    TileType::Grass
                }
                else{
                    TileType::Rock
                };
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
        tiles
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