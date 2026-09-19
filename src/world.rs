use macroquad::prelude::*;
use crate::agent::Agent;
pub struct World{
    agents: Vec<Agent>,
    food: Vec<Food>,
    height: f32,
    width: f32,
    camera: Camera2D
}

impl World{
    pub fn new_with_random_agents(num_agents: usize) -> World{
        let mut agents = Vec::new();
        let height = 1500.0;
        let width = 1500.0;
        let cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, width, height));

        for _ in 0..num_agents{
            agents.push(Agent::new_with_random_position(height, width));
        }

        World{
            agents,
            food: Vec::new(),
            height,
            width,
            camera: cam
        }
    }

    pub fn update(&mut self){
        for a in &mut self.agents{
            a.update(self.height, self.width);
        }
    }

    pub fn draw(&self){
        set_camera(&self.camera);

        for a in &self.agents{
            a.draw();
        }
    }
}

struct Food{
    x: f32,
    y: f32
}

impl Food{

}