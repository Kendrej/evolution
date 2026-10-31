use macroquad::prelude::*;
use crate::agent::Agent;
use crate::terrain::Terrain;
use crate::tribe::Tribe;

const FOOD_SPAWN_ATTEMPTS: usize = 3;
const FOOD_CHANCE_NEUTRAL: f32 = 0.05;
const FOOD_CHANCE_OWNED: f32 = 0.15;
const MAX_AGENTS: usize = 300;

pub struct World{
    agents: Vec<Agent>,
    food: Vec<Food>,
    camera: Camera2D,
    terrain: Terrain,
    tribes: Vec<Tribe>
}

impl World{
    pub fn new(num_agents: usize, num_food: usize) -> World{
        let mut agents = Vec::new();
        let mut food = Vec::new();
        let rows = 60;
        let cols = 60;
        let tile_size = 25.0;
        let base_size = 5;
        let terrain = Terrain::new(rows, cols, tile_size, base_size, 200);
        let tribes = Tribe::create_tribes(terrain.config());
        let height = terrain.height();
        let width = terrain.width();
        let cam: Camera2D = Camera2D::from_display_rect(Rect::new(0.0, 0.0, width, height));

        for i in 0..num_agents{
            let tribe = i % 4;
            agents.push(Agent::spawn_at_tile(tribes[tribe].random_base_tile(), tribe, terrain.config().tile_size));
        }

        for _ in 0..num_food * 10{
            if food.len() >= num_food{
                break;
            }
            if let Some((x, y)) = Food::random_free_spot(&terrain, &food){
                food.push(Food::new(x, y));
            }
        }

        World{
            agents,
            food,
            camera: cam,
            terrain,
            tribes
        }
    }

    pub fn update(&mut self){
        for a in &mut self.agents{
            for f in &mut self.food{
                let distance_sq = (a.get_x() - f.x).powi(2) + (a.get_y() - f.y).powi(2);
                let radius_sum = a.get_size() + f.size;
                if (distance_sq < radius_sum * radius_sum) && (a.is_alive()) && (!f.eaten)
                    && self.terrain.owner_at(f.x, f.y).is_none_or(|owner| owner == a.tribe()){
                    f.eaten = true;
                    a.eat_food(f.size, &self.terrain);
                }
            }
        }

        for a in &mut self.agents{
            a.update(&self.terrain);
            if a.is_alive(){
                self.terrain.capture(a.get_x(), a.get_y(), a.tribe());
            }
        }
        self.terrain.update_capture_status();

        self.food.retain(|f| !f.eaten);
        self.agents.retain(|a| !a.is_gone());

        self.reproduce_agents();
        self.spawn_food();
    }

    fn reproduce_agents(&mut self){
        let tile_size = self.terrain.config().tile_size;
        let agents_count = self.agents.len();
        let mut children = Vec::new();

        for j in 1..agents_count{
            let (left, right) = self.agents.split_at_mut(j);
            let b = &mut right[0];
            for a in left{
                if agents_count + children.len() >= MAX_AGENTS{
                    break;
                }
                if a.can_reproduce_with(b, &self.terrain){
                    children.push(a.reproduce_with(b, tile_size));
                }
            }
        }

        self.agents.extend(children);
    }

    fn spawn_food(&mut self){
        for _ in 0..FOOD_SPAWN_ATTEMPTS{
            if let Some((x, y)) = Food::random_free_spot(&self.terrain, &self.food){
                let chance = if self.terrain.owner_at(x, y).is_some(){
                    FOOD_CHANCE_OWNED
                }
                else{
                    FOOD_CHANCE_NEUTRAL
                };
                if rand::gen_range(0.0, 1.0) < chance{
                    self.food.push(Food::new(x, y));
                }
            }
        }
    }

    pub fn draw(&self){
        set_camera(&self.camera);

        self.terrain.draw();

        for a in &self.agents{
            a.draw(self.tribes[a.tribe()].color());
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
    fn new(x: f32, y: f32) -> Food{
        Food{
            x,
            y,
            size: 5.0,
            eaten: false
        }
    }

    fn random_free_spot(terrain: &Terrain, food: &[Food]) -> Option<(f32, f32)>{
        let config = terrain.config();
        let col = rand::gen_range(0, config.cols);
        let row = rand::gen_range(0, config.rows);
        let x = (col as f32 * config.tile_size) + config.tile_size/2.0;
        let y = (row as f32 * config.tile_size) + config.tile_size/2.0;

        if terrain.can_grow_food_at(x, y) && !food.iter().any(|f| f.x == x && f.y == y){
            Some((x, y))
        }
        else{
            None
        }
    }

    fn draw(&self){
        draw_circle(self.x, self.y, self.size, ORANGE);
    }
}