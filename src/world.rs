use macroquad::prelude::*;
use crate::agent::Agent;
use crate::terrain::Terrain;
use crate::tribe::Tribe;
use crate::stats::Stats;
use crate::brain::{Brain, Senses, relative_direction};
use std::f32::consts::FRAC_PI_2;

const FOOD_SPAWN_ATTEMPTS: usize = 3;
const FOOD_CHANCE_NEUTRAL: f32 = 0.05;
const FOOD_CHANCE_OWNED: f32 = 0.15;
const MAX_AGENTS: usize = 300;
const MIN_TRIBE_POPULATION: usize = 5;
const SIMULATION_SPEEDS: [usize; 3] = [1, 10, 50];
const STATS_BUTTON_WIDTH: f32 = 130.0;
const STATS_BUTTON_HEIGHT: f32 = 25.0;

pub struct World{
    agents: Vec<Agent>,
    food: Vec<Food>,
    camera: Camera2D,
    terrain: Terrain,
    tribes: Vec<Tribe>,
    stats: Stats,
    stats_open: bool,
    speed_index: usize,
    inheritance: bool
}

impl World{
    pub fn new(num_agents: usize, num_food: usize, seed: u32, inheritance: bool) -> World{
        let mut agents = Vec::new();
        let mut food = Vec::new();
        let rows = 60;
        let cols = 60;
        let tile_size = 25.0;
        let base_size = 5;
        let terrain = Terrain::new(rows, cols, tile_size, base_size, seed);
        let tribes = Tribe::create_tribes(terrain.config());
        let stats = Stats::new(seed, tribes.len(), inheritance);
        let height = terrain.height();
        let width = terrain.width();
        let cam: Camera2D = Camera2D::from_display_rect(Rect::new(0.0, 0.0, width, height));

        for i in 0..num_agents{
            let tribe = i % 4;
            agents.push(Agent::spawn_at_tile(tribes[tribe].random_base_tile(), tribe, terrain.config().tile_size, Brain::new()));
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
            tribes,
            stats,
            stats_open: false,
            speed_index: 0,
            inheritance
        }
    }

    pub fn handle_input(&mut self){
        self.toggle_stats_on_click();

        if is_key_pressed(KeyCode::Space){
            self.speed_index = (self.speed_index + 1) % SIMULATION_SPEEDS.len();
        }
    }

    pub fn steps_per_frame(&self) -> usize{
        SIMULATION_SPEEDS[self.speed_index]
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
                    self.stats.add_food(a.tribe());
                }
            }
        }

        for a in &mut self.agents{
            let was_alive = a.is_alive();
            let senses = sense(a, &self.terrain, &self.tribes[a.tribe()], &self.food);
            a.update(&self.terrain, &senses);
            if a.is_alive(){
                self.terrain.capture(a.get_x(), a.get_y(), a.tribe());
            }
            else if was_alive{
                self.stats.add_death(a.tribe(), a.age());
            }
        }
        self.terrain.update_capture_status();

        self.food.retain(|f| !f.eaten);
        self.agents.retain(|a| !a.is_gone());

        self.reproduce_agents();
        self.respawn_tribes();
        self.spawn_food();
        self.record_stats();
    }

    fn toggle_stats_on_click(&mut self){
        let (mouse_x, mouse_y) = mouse_position();
        if is_mouse_button_pressed(MouseButton::Left) && mouse_x <= STATS_BUTTON_WIDTH && mouse_y <= STATS_BUTTON_HEIGHT{
            self.stats_open = !self.stats_open;
        }
    }

    fn draw_stats(&self){
        let panel_color = Color::from_rgba(0, 0, 0, 200);
        let label = if self.stats_open { "Statystyki -" } else { "Statystyki +" };
        draw_rectangle(0.0, 0.0, STATS_BUTTON_WIDTH, STATS_BUTTON_HEIGHT, panel_color);
        draw_text(label, 10.0, 18.0, 20.0, WHITE);

        draw_rectangle(STATS_BUTTON_WIDTH, 0.0, 60.0, STATS_BUTTON_HEIGHT, panel_color);
        draw_text(&format!("x{}", self.steps_per_frame()), STATS_BUTTON_WIDTH + 10.0, 18.0, 20.0, WHITE);

        if !self.stats_open{
            return;
        }

        draw_rectangle(0.0, STATS_BUTTON_HEIGHT, 520.0, 150.0, panel_color);
        draw_text(&format!("Agenci: {}", self.agents.len()), 10.0, 45.0, 20.0, WHITE);
        draw_text(&format!("Jedzenie: {}", self.food.len()), 10.0, 65.0, 20.0, WHITE);

        for (i, tribe) in self.tribes.iter().enumerate(){
            let alive = self.agents.iter().filter(|a| a.is_alive() && a.tribe() == i).count();
            let text = format!("Plemie {}: zywi {}, narodziny {}, smierci {}, odrodzenia {}", i + 1, alive, self.stats.total_births(i), self.stats.total_deaths(i), self.stats.total_respawns(i));
            draw_text(&text, 10.0, 85.0 + i as f32 * 20.0, 20.0, tribe.color());
        }

        draw_text(&format!("Ziarno: {}", self.stats.seed()), 10.0, 165.0, 20.0, WHITE);
    }

    fn record_stats(&mut self){
        if !self.stats.tick(){
            return;
        }

        for i in 0..self.tribes.len(){
            let alive = self.agents.iter().filter(|a| a.is_alive() && a.tribe() == i).count();
            self.stats.write_row(i, alive, self.terrain.owned_tiles(i));
        }
    }

    fn respawn_tribes(&mut self){
        let tile_size = self.terrain.config().tile_size;

        for (i, tribe) in self.tribes.iter().enumerate(){
            let alive: Vec<usize> = (0..self.agents.len())
                .filter(|&index| self.agents[index].is_alive() && self.agents[index].tribe() == i)
                .collect();
            for _ in alive.len()..MIN_TRIBE_POPULATION{
                let brain = if self.inheritance && !alive.is_empty(){
                    let parent = alive[rand::gen_range(0, alive.len())];
                    self.agents[parent].brain().mutated()
                }
                else{
                    Brain::new()
                };
                self.agents.push(Agent::spawn_at_tile(tribe.random_base_tile(), i, tile_size, brain));
                self.stats.add_respawn(i);
            }
        }
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
                    children.push(a.reproduce_with(b, tile_size, self.inheritance));
                }
            }
        }

        for child in &children{
            self.stats.add_birth(child.tribe());
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

        self.draw_stats();
    }
}

fn sense(agent: &Agent, terrain: &Terrain, tribe: &Tribe, food: &[Food]) -> Senses{
    let (x, y) = (agent.get_x(), agent.get_y());
    let angle = agent.angle();
    let tribe_id = agent.tribe();
    let tile_size = terrain.config().tile_size;
    let max_distance = (terrain.width().powi(2) + terrain.height().powi(2)).sqrt();

    let (base_x, base_y) = tribe.base_center(tile_size);
    let base_distance = ((base_x - x).powi(2) + (base_y - y).powi(2)).sqrt();

    let mut nearest_food = None;
    let mut nearest_distance_sq = f32::MAX;
    for f in food{
        let distance_sq = (f.x - x).powi(2) + (f.y - y).powi(2);
        if distance_sq < nearest_distance_sq && !f.eaten
            && terrain.owner_at(f.x, f.y).is_none_or(|owner| owner == tribe_id){
            nearest_distance_sq = distance_sq;
            nearest_food = Some(f);
        }
    }

    let (food_direction, food_distance) = match nearest_food{
        Some(f) => (relative_direction(x, y, angle, f.x, f.y), nearest_distance_sq.sqrt() / max_distance),
        None => (0.0, 1.0)
    };

    let tile_at_angle = |a: f32| terrain.tile_sense(x + tile_size * a.cos(), y + tile_size * a.sin(), tribe_id);

    Senses{
        energy: agent.energy_ratio(),
        ready_to_reproduce: if agent.is_ready_to_reproduce() { 1.0 } else { 0.0 },
        base_direction: relative_direction(x, y, angle, base_x, base_y),
        base_distance: base_distance / max_distance,
        food_direction,
        food_distance,
        tile_here: terrain.tile_sense(x, y, tribe_id),
        capture_progress: terrain.capture_progress_at(x, y, tribe_id),
        tile_ahead: tile_at_angle(angle),
        tile_left: tile_at_angle(angle - FRAC_PI_2),
        tile_right: tile_at_angle(angle + FRAC_PI_2)
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