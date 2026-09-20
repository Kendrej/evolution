use macroquad::prelude::*;
use crate::agent::Agent;
use crate::terrain::Terrain;

pub struct World{
    agents: Vec<Agent>,
    food: Vec<Food>,
    height: f32,
    width: f32,
    camera: Camera2D,
    terrain: Terrain
}

impl World{
    pub fn new_with_random_agents(num_agents: usize, num_food: usize) -> World{
        let mut agents = Vec::new();
        let mut food = Vec::new();
        let terrain = Terrain::new(60, 60, 25.0, 5, 2);
        let height = terrain.get_height();
        let width = terrain.get_width();
        let cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, width, height));

        for _ in 0..num_agents{
            agents.push(Agent::new_with_random_position(height, width));
        }

        for _ in 0..num_food{
            food.push(Food::new_with_random_position(height, width));
        }

        World{
            agents,
            food,
            height,
            width,
            camera: cam,
            terrain
        }
    }

    pub fn update(&mut self){
        for a in &mut self.agents{
            a.update(self.height, self.width);
        }

        for a in &mut self.agents{
            for f in &mut self.food{
                let distance_sq = (a.get_x() - f.x).powi(2) + (a.get_y() - f.y).powi(2);
                let radius_sum = a.get_size() + f.size;
                if distance_sq < radius_sum * radius_sum {
                    f.eaten = true;
                    a.eat_food(f.size);
                }
            }
        }

        self.food.retain(|f| !f.eaten);

    }

    pub fn draw(&self){
        set_camera(&self.camera);

        self.terrain.draw();

        for a in &self.agents{
            a.draw();
        }

        for f in &self.food{
            f.draw();
        }

        set_default_camera();

        draw_text(&format!("Agenci: {}", self.agents.len()), 10.0, 20.0, 20.0, Color::from_rgba(255, 255, 255, 255));
        draw_text(&format!("Jedzenie: {}", self.food.len()), 10.0, 40.0, 20.0, Color::from_rgba(255, 255, 255, 255));
    }
}

struct Food{
    x: f32,
    y: f32,
    size: f32,
    eaten: bool
}

impl Food{
    fn new_with_random_position(height: f32, width: f32) -> Food{
        let size = 5.0;
        let x = rand::gen_range(10.0, width - 10.0);
        let y = rand::gen_range(10.0, height - 10.0);
        Food{
            x,
            y,
            size,
            eaten: false
        }
    }

    fn draw(&self){
        draw_circle(self.x, self.y, self.size, ORANGE);
    }
}