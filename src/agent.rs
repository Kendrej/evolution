use macroquad::prelude::*;

pub struct Agent{
    x: f32,
    y: f32,
    i: i32,
    j: i32,
    speed: f32,
    color: Color
}

impl Agent{
    pub fn new_with_position(x: f32, y: f32, speed: f32) -> Agent{
        Agent{
            x,
            y,
            i: 1,
            j: 1,
            speed,
            color: BLUE
        }
    }

    pub fn new_with_random_position(height: f32, width: f32, speed: f32) -> Agent{
        let x = rand::gen_range(0.0, width - 60.0);
        let y = rand::gen_range(0.0, height - 40.0);
        Agent{
            x,
            y,
            i: 1,
            j: 1,
            speed,
            color: BLUE
        }
    }

    pub fn update(&mut self, height: f32, width: f32){
        if self.x >= width - 60.0 {
            self.i = -1;
        }
        else if self.x <= 0.0 {
            self.i = 1;
        }
        if self.y >= height - 40.0 {
            self.j = -1;
        }
        else if self.y <= 0.0 {
            self.j = 1;
        }
        
        if self.check_if_corner(height, width) {
            self.color = RED;
            self.speed = 0.0;
        }
        
        self.x += self.speed * self.i as f32;
        self.y += self.speed * self.j as f32;
    }

    pub fn draw(&self){
        draw_rectangle(self.x, self.y, 60.0, 40.0, self.color);
    }

    fn check_if_corner(&self, height: f32, width: f32) -> bool{
        if (self.x >= width - 60.0 && self.y >= height - 40.0) || (self.x <= 0.0 && self.y <= 0.0) || (self.x >= width - 60.0 && self.y <= 0.0) || (self.x <= 0.0 && self.y >= height - 40.0){
            return true;
        }
        return false
    }
}