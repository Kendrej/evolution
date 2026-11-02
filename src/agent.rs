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
    attack_cooldown: i32,
    age: u32,
    brain: Brain
}

const TIME_OF_DYING: i32 = 500;
const TIME_OF_DISAPPEARING: f32 = 300.0;
const REPRODUCTION_MIN_ENERGY: f32 = 60.0;
const REPRODUCTION_COST: f32 = 25.0;
const REPRODUCTION_COOLDOWN: i32 = 600;
const MAX_AGE: u32 = 15_000;
const ATTACK_COST: f32 = 0.25;
const ATTACK_COOLDOWN: i32 = 30;
const ATTACK_DAMAGE: f32 = 25.0;
const ATTACK_STOLEN_PART: f32 = 0.5;
const ATTACK_REACH: f32 = 5.0;
const FOOD_ENERGY_PER_SIZE: f32 = 8.0;

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
            attack_cooldown: 0,
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

        // Baza nalezy do plemienia na stale, wiec tez sie tu liczy
        if terrain.owner_at(self.x, self.y) != Some(self.tribe) || terrain.owner_at(other.x, other.y) != Some(other.tribe){
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



    pub fn can_hit(&self, other: &Agent) -> bool{
        if self.tribe == other.tribe || !self.alive || !other.alive{
            return false
        }

        let distance_sq = (self.x - other.x).powi(2) + (self.y - other.y).powi(2);
        let reach = self.size + other.size + ATTACK_REACH;
        distance_sq < reach * reach
    }

    // Atakujacy zabiera czesc odebranej energii. Zwraca true, gdy cel zginal.
    pub fn hit(&mut self, target: &mut Agent) -> bool{
        let damage = ATTACK_DAMAGE.min(target.energy);
        target.energy -= damage;
        self.energy = (self.energy + damage * ATTACK_STOLEN_PART).min(self.max_energy);

        if target.energy <= 0.0{
            target.die();
            return true
        }
        false
    }

    pub fn die(&mut self){
        self.energy = 0.0;
        self.timer_of_death += 1;
        self.alive = false;
    }

    // Zwraca true, gdy agent wykonal w tym kroku atak
    pub fn update(&mut self, terrain: &Terrain, senses: &Senses) -> bool{
        let mut attacked = false;

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

            if self.attack_cooldown > 0 {
                self.attack_cooldown -= 1;
            }
            else if action.attack {
                // Pudlo tez kosztuje, zeby nie oplacalo sie atakowac bez przerwy
                self.energy -= ATTACK_COST;
                self.attack_cooldown = ATTACK_COOLDOWN;
                attacked = true;
            }

            self.energy -= 0.05;
            if self.should_die() {
                self.die();
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

        attacked && self.alive
    }

    fn should_die(&self) -> bool{
        self.energy <= 0.0 || self.age >= MAX_AGE
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

        if self.energy + food_size * FOOD_ENERGY_PER_SIZE <= self.max_energy {
            self.energy += food_size * FOOD_ENERGY_PER_SIZE;
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

#[cfg(test)]
mod tests{
    use super::*;

    fn agent() -> Agent{
        Agent::new(0.0, 0.0, 0, 100.0, 25.0, Brain::new())
    }

    #[test]
    fn agent_with_energy_below_the_age_limit_survives(){
        let mut agent = agent();
        agent.age = MAX_AGE - 1;
        assert!(!agent.should_die());
    }

    #[test]
    fn agent_dies_at_the_age_limit(){
        let mut agent = agent();
        agent.age = MAX_AGE;
        assert!(agent.should_die());
    }

    #[test]
    fn agent_without_energy_dies(){
        let mut agent = agent();
        agent.energy = 0.0;
        assert!(agent.should_die());
    }

    fn ready_pair_at(x: f32, y: f32, tribe: usize) -> (Agent, Agent){
        let mut first = Agent::new(x, y, tribe, 100.0, 25.0, Brain::new());
        let mut second = Agent::new(x, y, tribe, 100.0, 25.0, Brain::new());
        first.reproduction_cooldown = 0;
        second.reproduction_cooldown = 0;
        (first, second)
    }

    #[test]
    fn agents_reproduce_only_on_tiles_of_their_tribe(){
        let mut terrain = Terrain::new(20, 20, 25.0, 3, 1);
        let (first, second) = ready_pair_at(30.0, 30.0, 0);
        assert!(first.can_reproduce_with(&second, &terrain));

        let (first, second) = ready_pair_at(30.0, 30.0, 1);
        assert!(!first.can_reproduce_with(&second, &terrain));

        let (x, y) = (250.0, 250.0);
        let (first, second) = ready_pair_at(x, y, 0);
        assert_eq!(terrain.owner_at(x, y), None);
        assert!(!first.can_reproduce_with(&second, &terrain));

        terrain.set_owner(x, y, 0);
        assert!(first.can_reproduce_with(&second, &terrain));
    }

    #[test]
    fn hit_moves_part_of_the_energy_to_the_attacker(){
        let mut attacker = agent();
        attacker.energy = 50.0;
        let mut target = agent();

        assert!(!attacker.hit(&mut target));
        assert_eq!(target.energy, 100.0 - ATTACK_DAMAGE);
        assert_eq!(attacker.energy, 50.0 + ATTACK_DAMAGE * ATTACK_STOLEN_PART);
    }

    #[test]
    fn hit_kills_a_weak_target_and_steals_only_what_it_had(){
        let mut attacker = agent();
        attacker.energy = 50.0;
        let mut target = agent();
        target.energy = 10.0;

        assert!(attacker.hit(&mut target));
        assert!(!target.is_alive());
        assert_eq!(attacker.energy, 50.0 + 10.0 * ATTACK_STOLEN_PART);
    }

    #[test]
    fn stolen_energy_does_not_exceed_the_maximum(){
        let mut attacker = agent();
        let mut target = agent();
        attacker.hit(&mut target);
        assert_eq!(attacker.energy, attacker.max_energy);
    }

    #[test]
    fn only_a_close_living_enemy_can_be_hit(){
        let attacker = agent();
        let mut enemy = Agent::new(10.0, 0.0, 1, 100.0, 25.0, Brain::new());
        assert!(attacker.can_hit(&enemy));

        enemy.x = 500.0;
        assert!(!attacker.can_hit(&enemy));

        enemy.x = 10.0;
        enemy.die();
        assert!(!attacker.can_hit(&enemy));

        let friend = Agent::new(10.0, 0.0, 0, 100.0, 25.0, Brain::new());
        assert!(!attacker.can_hit(&friend));
    }
}