use macroquad::prelude::*;
use std::f32::consts::PI;
pub struct Agent{
    x: f32,
    y: f32,
    angle: f32,
    speed: f32,
    color: Color,
    size: f32
}

impl Agent{
    pub fn new_with_random_position(height: f32, width: f32) -> Agent{
        let size = 10.0;
        let x = rand::gen_range(0.0, width - size);
        let y = rand::gen_range(0.0, height - size);
        Agent{
            x,
            y,
            angle: 0.0,
            speed: 1.0,
            color: BLUE,
            size: size
        }
    }



    pub fn update(&mut self, height: f32, width: f32){
        self.angle += rand::gen_range(-0.1, 0.1);

        let new_x = self.x + self.speed * self.angle.cos();
        let new_y = self.y + self.speed * self.angle.sin();

        if new_x >= width - self.size {
            self.angle = PI - self.angle;
        }
        else if new_x <= 0.0 {
            self.angle = PI - self.angle;
        }
        if new_y >= height - self.size {
            self.angle = -self.angle;
        }
        else if new_y <= 0.0 {
            self.angle = -self.angle;
        }
        self.x += self.speed * self.angle.cos();
        self.y += self.speed * self.angle.sin();

    }



    pub fn draw(&self){
        draw_circle(self.x, self.y, self.size, self.color)
    }
}