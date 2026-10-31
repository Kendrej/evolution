use macroquad::prelude::*;
use noise::{NoiseFn, Perlin};
use std::{collections::VecDeque, vec};
use crate::tribe::TRIBE_COLORS;

const CAPTURE_TIME_NEUTRAL: f32 = 60.0;
const CAPTURE_TIME_ENEMY: f32 = 180.0;

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

struct Tile{
    tile_type: TileType,
    owner: Option<usize>,
    capturer: Option<usize>,
    capture_progress: f32,
    capturer_present: bool
}

impl Tile{
    fn new(tile_type: TileType, owner: Option<usize>, capturer: Option<usize>) -> Tile{
        Tile{
            tile_type,
            owner,
            capturer,
            capture_progress: 0.0,
            capturer_present: false
        }
    }

    fn is_walkable(&self) -> bool{
        self.tile_type.is_walkable()
    }

    fn capture_time(&self) -> f32{
        if self.owner.is_none() {
            CAPTURE_TIME_NEUTRAL
        } else {
            CAPTURE_TIME_ENEMY
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
    tiles: Vec<Vec<Tile>>
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

    fn generate(config: &TerrainConfig, attempt: u32) -> Vec<Vec<Tile>>{
        let perlin = Perlin::new(config.seed.wrapping_add(attempt));

        
        let mut tiles= Vec::new();
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
                let tile = Tile::new(tile_type, None, None);
                row.push(tile);
            }
            tiles.push(row);
        }



        for i in 0..config.base_size{
            for j in 0..config.base_size{
                tiles[i][j].tile_type = TileType::Base(0);
                tiles[i][j].owner = Some(0);
                tiles[i][config.cols - 1 - j].tile_type = TileType::Base(1);
                tiles[i][config.cols - 1 - j].owner = Some(1);
                tiles[config.rows - 1 - i][j].tile_type = TileType::Base(2);
                tiles[config.rows - 1 - i][j].owner = Some(2);
                tiles[config.rows - 1 - i][config.cols - 1 - j].tile_type = TileType::Base(3);
                tiles[config.rows - 1 - i][config.cols - 1 - j].owner = Some(3);
            }
        }
        tiles
    }

    fn validate_map(config: &TerrainConfig, tiles: &Vec<Vec<Tile>>) -> bool{
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
        if x < 0.0 || x >= self.width() || y < 0.0 || y >= self.height(){
            return false
        }

        let row = (y / self.config.tile_size) as usize;
        let col = (x / self.config.tile_size) as usize;

        self.tiles[row][col].is_walkable()
    }

    pub fn can_grow_food_at(&self, x: f32, y: f32) -> bool{
        if !self.is_walkable_at(x, y){
            return false
        }

        let row = (y / self.config.tile_size) as usize;
        let col = (x / self.config.tile_size) as usize;

        !matches!(self.tiles[row][col].tile_type, TileType::Base(_))
    }

    pub fn is_base_of(&self, x: f32, y: f32, tribe: usize) -> bool{
        let row = (y / self.config.tile_size) as usize;
        let col = (x / self.config.tile_size) as usize;

        matches!(self.tiles[row][col].tile_type, TileType::Base(t) if t as usize == tribe)
    }

    // Pole widziane przez agenta: wlasne 1, neutralne 0, wroga -0.5, nie do przejscia -1
    pub fn tile_sense(&self, x: f32, y: f32, tribe: usize) -> f32{
        if !self.is_walkable_at(x, y){
            return -1.0
        }

        match self.owner_at(x, y){
            Some(owner) if owner == tribe => 1.0,
            Some(_) => -0.5,
            None => 0.0
        }
    }

    pub fn capture_progress_at(&self, x: f32, y: f32, tribe: usize) -> f32{
        let row = (y / self.config.tile_size) as usize;
        let col = (x / self.config.tile_size) as usize;

        let tile = &self.tiles[row][col];
        if tile.capturer == Some(tribe) {
            tile.capture_progress / tile.capture_time()
        } else {
            0.0
        }
    }

    pub fn owned_tiles(&self, tribe: usize) -> usize{
        self.tiles.iter().flatten().filter(|tile| tile.owner == Some(tribe)).count()
    }

    pub fn owner_at(&self, x: f32, y: f32) -> Option<usize>{
        let row = (y / self.config.tile_size) as usize;
        let col = (x / self.config.tile_size) as usize;

        self.tiles[row][col].owner
    }

    pub fn can_walk(&self, x: f32, y: f32, radius: f32) -> bool {
        self.is_walkable_at(x - radius, y)
        && self.is_walkable_at(x + radius, y)
        && self.is_walkable_at(x, y - radius)
        && self.is_walkable_at(x, y + radius)
    }

    pub fn capture(&mut self, x: f32, y: f32, tribe: usize) {
        let row = (y / self.config.tile_size) as usize;
        let col = (x / self.config.tile_size) as usize;

        let tile = &mut self.tiles[row][col];
        if matches!(tile.tile_type, TileType::Base(_)){
            return;
        }

        if tile.owner == Some(tribe) {
            return;
        }

        if tile.capturer == Some(tribe) {
            tile.capture_progress += 1.0;
        } else {
            tile.capturer = Some(tribe);
            tile.capture_progress = 0.0;
        }
        tile.capturer_present = true;

        if tile.capture_progress >= tile.capture_time() {
            tile.owner = Some(tribe);
            tile.capturer = None;
            tile.capture_progress = 0.0;
        }
    }

    pub fn draw(&self){
        for row in 0..self.config.rows{
            for col in 0..self.config.cols{
                let tile_type = self.tiles[row][col].tile_type;
                let mut color = tile_type.get_color();  

                if let Some(t) = self.tiles[row][col].owner {
                    let c = TRIBE_COLORS[t];  
                    color = mix(c, color, 0.4);
                }   
    
                let x = col as f32 * self.config.tile_size;
                let y = row as f32 * self.config.tile_size;
                draw_rectangle(x, y, self.config.tile_size, self.config.tile_size, color);
                draw_rectangle_lines(x, y, self.config.tile_size, self.config.tile_size, 1.0, BLACK);             
            }
        }
    }

    pub fn update_capture_status(&mut self) {
        for row in &mut self.tiles {
            for tile in row {
                if !tile.capturer_present {
                    tile.capturer = None;
                    tile.capture_progress = 0.0;
                }
                tile.capturer_present = false; 
            }
        }
    }

    pub fn width(&self) -> f32{
        self.config.cols as f32 * self.config.tile_size
    }

    pub fn height(&self) -> f32{
        self.config.rows as f32 * self.config.tile_size
    }
    
    pub fn config(&self) -> &TerrainConfig{
        &self.config
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    fn flat_terrain() -> Terrain{
        let mut terrain = Terrain::new(10, 10, 25.0, 2, 1);
        for row in &mut terrain.tiles{
            for tile in row{
                tile.tile_type = TileType::Grass;
                tile.owner = None;
            }
        }
        terrain
    }

    #[test]
    fn tile_sense_tells_tiles_apart(){
        let mut terrain = flat_terrain();
        terrain.tiles[1][0].owner = Some(0);
        terrain.tiles[1][1].owner = Some(1);
        terrain.tiles[1][2].tile_type = TileType::Rock;

        assert_eq!(terrain.tile_sense(12.0, 37.0, 0), 1.0);
        assert_eq!(terrain.tile_sense(37.0, 37.0, 0), -0.5);
        assert_eq!(terrain.tile_sense(62.0, 37.0, 0), -1.0);
        assert_eq!(terrain.tile_sense(87.0, 37.0, 0), 0.0);
        assert_eq!(terrain.tile_sense(-5.0, 37.0, 0), -1.0);
    }

    #[test]
    fn capture_progress_is_a_fraction_for_the_capturer_only(){
        let mut terrain = flat_terrain();
        for _ in 0..31{
            terrain.capture(12.0, 12.0, 0);
        }

        assert_eq!(terrain.capture_progress_at(12.0, 12.0, 0), 0.5);
        assert_eq!(terrain.capture_progress_at(12.0, 12.0, 1), 0.0);
    }
}

pub fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r * t + b.r * (1.0 - t),
        a.g * t + b.g * (1.0 - t),
        a.b * t + b.b * (1.0 - t),
        1.0,
    )
}