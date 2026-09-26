use macroquad::prelude::*;
use crate::terrain::Terrain;
use macroquad::rand::gen_range;

pub struct Agent{
    x: f32,
    y: f32,
    angle: f32,
    speed: f32,
    size: f32,
    tribe: usize
}

impl Agent{
    pub fn spawn_at_tile(tile: (usize, usize), tribe: usize, tile_size: f32) -> Agent{
        let x = (tile.0 as f32 * tile_size) + tile_size/2.0;
        let y = (tile.1 as f32 * tile_size) + tile_size/2.0;
        Agent{
            x,
            y,
            angle: gen_range(0.0, 360.0),
            speed: 1.0,
            size: tile_size * 0.4,
            tribe
        }
    }



    pub fn update(&mut self, terrain: &Terrain){
        self.angle += rand::gen_range(-0.1, 0.1);
        
        let new_x = self.x + self.speed * self.angle.cos();
        let new_y = self.y + self.speed * self.angle.sin();

        if terrain.can_walk(new_x, self.y, self.size){
            self.x = new_x;
        }
        if terrain.can_walk(self.x, new_y, self.size){
            self.y = new_y;
        }
    }

    pub fn draw(&self, color: Color){
        draw_circle(self.x, self.y, self.size, color)
    }

    pub fn eat_food(&mut self, food_size: f32, tile_size: f32){
        let new_size = self.size + food_size * 0.2;
        
        if new_size <= tile_size/2.0 {
            self.size = new_size;
        }
        else {
            self.size = tile_size/2.0;
        }
    }


    pub fn get_x(&self) -> f32{
        self.x
    }

    pub fn get_y(&self) -> f32{
        self.y
    }

    pub fn get_size(&self) -> f32{
        self.size
    }
    
    pub fn tribe(&self) -> usize{
        self.tribe
    }
}