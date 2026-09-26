use macroquad::prelude::*;
use noise::{NoiseFn, Perlin};
use std::{collections::VecDeque, vec};
use crate::tribe::TRIBE_COLORS;

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
            TileType::Base(tribe) => mix(TRIBE_COLORS[*tribe as usize], Color::new(0.15, 0.15, 0.17, 1.0), 0.35)
        }
    }

    fn is_walkable(&self) -> bool{
        match self{
            TileType::Rock => false,
            _ => true
        }
    }
}

pub struct TerrainConfig{
    pub rows: usize,
    pub cols: usize,
    pub base_size: usize,
    pub tile_size: f32,
    pub seed: u32,
}

pub struct Terrain{
    config: TerrainConfig,
    tiles: Vec<Vec<TileType>>
}

impl Terrain{
    pub fn new(rows: usize, cols: usize, tile_size: f32, base_size: usize, seed: u32) -> Terrain{
        let config = TerrainConfig {
            rows,
            cols,
            tile_size,
            base_size,
            seed,
        };
        
        let mut tiles = Vec::new();
        let mut correct_map = false;
        for attempt in 0..100{
            tiles = Terrain::generate(&config, attempt);
            if Terrain::validate_map(&config, &tiles){
                correct_map = true;
                break
            }
        }
        if !correct_map{
            panic!("Nie udało się wygenerować poprawnej mapy po 100 próbach");
        }

        

        Terrain{
            config,
            tiles
        }
    }

    fn generate(config: &TerrainConfig, attempt: u32) -> Vec<Vec<TileType>>{
        let perlin = Perlin::new(config.seed + attempt);

        
        let mut tiles = Vec::new();
        for i in 0..config.rows{
            let mut row = Vec::new();
            for j in 0..config.cols{
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



        for i in 0..config.base_size{
            for j in 0..config.base_size{
                tiles[i][j] = TileType::Base(0);
                tiles[i][config.cols - 1 - j] = TileType::Base(1);
                tiles[config.rows - 1 - i][j] = TileType::Base(2);
                tiles[config.rows - 1 - i][config.cols - 1 - j] = TileType::Base(3);
            }
        }
        tiles
    }

    fn validate_map(config: &TerrainConfig, tiles: &Vec<Vec<TileType>>) -> bool{
        let mut visited_tiles = vec![vec![false; config.cols]; config.rows];
        let neighbors = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        let mut queue = VecDeque::new();
        visited_tiles[0][0] = true;
        queue.push_back((0usize, 0usize));

        while let Some((row,col)) = queue.pop_front(){
            for (dr, dc) in neighbors {
                let new_row = row as i32 + dr;
                let new_col = col as i32 + dc;
                if new_row >= 0 && new_row < config.rows as i32 && new_col >= 0 && new_col < config.cols as i32 {
                    let new_row = new_row as usize;
                    let new_col = new_col as usize;
                    if tiles[new_row][new_col].is_walkable() && !visited_tiles[new_row][new_col] {
                        visited_tiles[new_row][new_col] = true;
                        queue.push_back((new_row, new_col));
                    }           
                }
            }
        }
        if !visited_tiles[0][config.cols - 1]
            || !visited_tiles[config.rows - 1][0]
            || !visited_tiles[config.rows - 1][config.cols - 1] {
            return false
        }
        true
    }

    pub fn is_walkable_at(&self, x: f32, y: f32) -> bool{
        if x < 0.0 || x >= self.get_width() || y < 0.0 || y >= self.get_height(){
            return false
        }

        let row = (y / self.config.tile_size) as usize;
        let col = (x / self.config.tile_size) as usize;

        self.tiles[row][col].is_walkable()
    }

    pub fn can_walk(&self, x: f32, y: f32, radius: f32) -> bool {
        self.is_walkable_at(x - radius, y)
        && self.is_walkable_at(x + radius, y)
        && self.is_walkable_at(x, y - radius)
        && self.is_walkable_at(x, y + radius)
    }

    pub fn draw(&self){
        for row in 0..self.config.rows{
            for col in 0..self.config.cols{
                let tile_type = self.tiles[row][col];
                let color = tile_type.get_color();
                let x = col as f32 * self.config.tile_size;
                let y = row as f32 * self.config.tile_size;
                draw_rectangle(x, y, self.config.tile_size, self.config.tile_size, color);
                draw_rectangle_lines(x, y, self.config.tile_size, self.config.tile_size, 1.0, BLACK);
            }
        }
    }

    pub fn get_width(&self) -> f32{
        self.config.cols as f32 * self.config.tile_size
    }

    pub fn get_height(&self) -> f32{
        self.config.rows as f32 * self.config.tile_size
    }

    pub fn get_tile_size(&self) -> f32{
        self.config.tile_size
    }
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r * t + b.r * (1.0 - t),
        a.g * t + b.g * (1.0 - t),
        a.b * t + b.b * (1.0 - t),
        1.0,
    )
}