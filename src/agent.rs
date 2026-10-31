use macroquad::prelude::*;
use crate::terrain::Terrain;
use crate::brain::{Brain, Senses};
use macroquad::rand::gen_range;
use crate::terrain::mix;
use std::f32::consts::TAU;
pub struct Agent{
    x: f32,
    y: f32,
    angle: f32,
    size: f32,
    tribe: usize,
    energy: f32,
    max_energy: f32,
    alive: bool,
    timer_of_death: i32,
    reproduction_cooldown: i32,
    age: u32,
    brain: Brain
}

const TIME_OF_DYING: i32 = 500;
const TIME_OF_DISAPPEARING: f32 = 300.0;
const REPRODUCTION_MIN_ENERGY: f32 = 60.0;
const REPRODUCTION_COST: f32 = 25.0;
const REPRODUCTION_COOLDOWN: i32 = 600;

impl Agent{
    fn new(x: f32, y: f32, tribe: usize, energy: f32, tile_size: f32, brain: Brain) -> Agent{
        Agent{
            x,
            y,
            angle: gen_range(0.0, TAU),
            size: tile_size * 0.4,
            tribe,
            energy,
            max_energy: 100.0,
            alive: true,
            timer_of_death: 0,
            reproduction_cooldown: REPRODUCTION_COOLDOWN,
            age: 0,
            brain
        }
    }

    pub fn spawn_at_tile(tile: (usize, usize), tribe: usize, tile_size: f32, brain: Brain) -> Agent{
        let x = (tile.0 as f32 * tile_size) + tile_size/2.0;
        let y = (tile.1 as f32 * tile_size) + tile_size/2.0;
        Agent::new(x, y, tribe, 100.0, tile_size, brain)
    }

    pub fn is_ready_to_reproduce(&self) -> bool{
        self.alive && self.energy >= REPRODUCTION_MIN_ENERGY && self.reproduction_cooldown == 0
    }

    pub fn can_reproduce_with(&self, other: &Agent, terrain: &Terrain) -> bool{
        if self.tribe != other.tribe || !self.is_ready_to_reproduce() || !other.is_ready_to_reproduce(){
            return false
        }

        if !terrain.is_base_of(self.x, self.y, self.tribe) || !terrain.is_base_of(other.x, other.y, other.tribe){
            return false
        }

        let distance_sq = (self.x - other.x).powi(2) + (self.y - other.y).powi(2);
        let radius_sum = self.size + other.size;
        distance_sq < radius_sum * radius_sum
    }

    pub fn reproduce_with(&mut self, other: &mut Agent, tile_size: f32, brain: Brain) -> Agent{
        self.energy -= REPRODUCTION_COST;
        other.energy -= REPRODUCTION_COST;
        self.reproduction_cooldown = REPRODUCTION_COOLDOWN;
        other.reproduction_cooldown = REPRODUCTION_COOLDOWN;
        Agent::new(self.x, self.y, self.tribe, REPRODUCTION_COST * 2.0, tile_size, brain)
    }



    pub fn update(&mut self, terrain: &Terrain, senses: &Senses){
        if self.alive{
            let action = self.brain.decide(senses);
            self.angle += action.turn;
        
            let new_x = self.x + action.speed * self.angle.cos();
            let new_y = self.y + action.speed * self.angle.sin();

            if terrain.can_walk(new_x, self.y, self.size){
                self.x = new_x;
            }
            if terrain.can_walk(self.x, new_y, self.size){
                self.y = new_y;
            }

            self.age += 1;

            if self.reproduction_cooldown > 0 {
                self.reproduction_cooldown -= 1;
            }

            self.energy -= 0.05;
            if self.energy <= 0.0 {
                self.energy = 0.0;
                self.timer_of_death += 1;
                self.alive = false;
            }
        }
        else {
            if self.timer_of_death > TIME_OF_DYING {
                self.size -= 0.05;
                if self.size <= 0.0 {
                    self.size = 0.0;
                }
            }
            else{
                self.timer_of_death += 1;
            }
        }
        
    }

    pub fn draw(&self, color: Color){
        if self.alive {
            let energy_ratio = self.energy / self.max_energy;
            let new_color = mix(color, GRAY, energy_ratio);
            draw_circle(self.x, self.y, self.size, new_color)  
        }
        else {
            let time_ratio = (self.timer_of_death as f32 / TIME_OF_DISAPPEARING).min(1.0);
            let new_color = mix(BLACK, GRAY, time_ratio);
            draw_circle(self.x, self.y, self.size, new_color) 
        }
    }

    pub fn eat_food(&mut self, food_size: f32, terrain: &Terrain){
        let max_size = terrain.config().tile_size/2.0;
        let new_size = (self.size + food_size * 0.2).min(max_size);

        if terrain.can_walk(self.x, self.y, new_size) {
            self.size = new_size;
        }

        if self.energy + food_size * 5.0 <= self.max_energy {
            self.energy += food_size * 5.0;
        }
        else {
            self.energy = self.max_energy;
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

    pub fn is_gone(&self) -> bool{
        self.size == 0.0 && !self.alive
    }

    pub fn brain(&self) -> &Brain{
        &self.brain
    }

    pub fn angle(&self) -> f32{
        self.angle
    }

    pub fn energy_ratio(&self) -> f32{
        self.energy / self.max_energy
    }

    pub fn age(&self) -> u32{
        self.age
    }

    pub fn is_alive(&self) -> bool{
        self.alive
    }
}