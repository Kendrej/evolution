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
    pub fn new() -> Agent{
        Agent{
            x: 10.0,
            y: 10.0,
            i: 1,
            j: 1,
            speed: 1.0,
            color: BLUE
        }
    }
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

    pub fn update(&mut self, high: f32, width: f32){
        if self.x >= width - 60.0 {
            self.i = -1;
        }
        else if self.x <= 0.0 {
            self.i = 1;
        }
        if self.y >= high - 40.0 {
            self.j = -1;
        }
        else if self.y <= 0.0 {
            self.j = 1;
        }
        
        if self.check_if_corner(high, width) {
            self.color = RED;
            self.speed = 0.0;
        }
        
        self.x += self.speed * self.i as f32;
        self.y += self.speed * self.j as f32;
    }

    pub fn draw(&self){
        draw_rectangle(self.x, self.y, 60.0, 40.0, self.color);
    }

    fn check_if_corner(&self, high: f32, width: f32) -> bool{
        if (self.x >= width - 60.0 && self.y >= high - 40.0) || (self.x <= 0.0 && self.y <= 0.0) || (self.x >= width - 60.0 && self.y <= 0.0) || (self.x <= 0.0 && self.y >= high - 40.0){
            return true;
        }
        return false
    }
}