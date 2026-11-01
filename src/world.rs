use macroquad::prelude::*;
use crate::agent::Agent;
use crate::terrain::Terrain;
use crate::tribe::Tribe;
use crate::stats::Stats;
use crate::brain::{Brain, Senses, relative_direction};
use crate::model::{Model, ModelSaver, format_steps};
use std::f32::consts::FRAC_PI_2;

const FOOD_SPAWN_ATTEMPTS: usize = 3;
const FOOD_CHANCE_NEUTRAL: f32 = 0.02;
const FOOD_CHANCE_OWNED: f32 = 0.05;
const MAX_FOOD: usize = 2000;
const MAX_AGENTS: usize = 300;
const MIN_TRIBE_POPULATION: usize = 5;
const SIMULATION_SPEEDS: [usize; 3] = [1, 10, 50];
const STATS_BUTTON_WIDTH: f32 = 130.0;
const STATS_BUTTON_HEIGHT: f32 = 25.0;
const MODEL_SAVE_INTERVAL: u64 = 50_000;
const PARENT_POOL_SIZE: usize = 20;
const KILLER_PARENT_CHANCE: f32 = 0.25;

#[derive(Clone, Copy, PartialEq)]
pub enum Mode{
    Inherit,
    Random,
    Demo
}

impl Mode{
    // Nazwa przebiegu, ktory zapisuje statystyki i modele. Demo niczego nie zapisuje.
    pub fn name(self) -> Option<&'static str>{
        match self{
            Mode::Inherit => Some("inherit"),
            Mode::Random => Some("random"),
            Mode::Demo => None
        }
    }

    fn child_brain(self, first: &Brain, second: &Brain) -> Brain{
        match self{
            Mode::Inherit => Brain::child_of(first, second),
            Mode::Random => Brain::new(),
            Mode::Demo => Brain::crossover(first, second)
        }
    }

    fn respawn_brain(self, parent: Option<&Brain>) -> Brain{
        match (self, parent){
            (Mode::Inherit, Some(brain)) => brain.mutated(),
            (Mode::Demo, Some(brain)) => brain.clone(),
            _ => Brain::new()
        }
    }
}

pub struct World{
    agents: Vec<Agent>,
    food: Vec<Food>,
    camera: Camera2D,
    terrain: Terrain,
    tribes: Vec<Tribe>,
    stats: Stats,
    stats_open: bool,
    speed_index: usize,
    mode: Mode,
    saved_brains: Vec<Vec<Brain>>,
    parents: Vec<Vec<Brain>>,
    saver: Option<ModelSaver>,
    seed: u32,
    start_step: u64
}

impl World{
    pub fn new(num_agents: usize, num_food: usize, seed: u32, mode: Mode, model: Option<Model>, saver: Option<ModelSaver>, is_comparison: bool) -> World{
        let mut agents = Vec::new();
        let mut food = Vec::new();
        let rows = 60;
        let cols = 60;
        let tile_size = 25.0;
        let base_size = 5;
        let terrain = Terrain::new(rows, cols, tile_size, base_size, seed);
        let tribes = Tribe::create_tribes(terrain.config());
        let stats = Stats::new(seed, tribes.len(), mode.name(), is_comparison);
        let height = terrain.height();
        let width = terrain.width();
        let cam: Camera2D = Camera2D::from_display_rect(Rect::new(0.0, 0.0, width, height));

        let start_step = model.as_ref().map_or(0, |m| m.step);
        let saved_brains: Vec<Vec<Brain>> = model.iter()
            .flat_map(|m| &m.tribes)
            .map(|tribe| tribe.iter().map(|weights| Brain::from_weights(weights)).collect())
            .collect();
        let mut parents: Vec<Vec<Brain>> = vec![Vec::new(); tribes.len()];
        for (pool, saved) in parents.iter_mut().zip(model.iter().flat_map(|m| &m.parents)){
            *pool = saved.iter().map(|weights| Brain::from_weights(weights)).collect();
        }

        if model.is_some(){
            for (tribe, brains) in saved_brains.iter().enumerate(){
                for brain in brains{
                    agents.push(Agent::spawn_at_tile(tribes[tribe].random_base_tile(), tribe, terrain.config().tile_size, brain.clone()));
                }
            }
        }
        else{
            for i in 0..num_agents{
                let tribe = i % 4;
                agents.push(Agent::spawn_at_tile(tribes[tribe].random_base_tile(), tribe, terrain.config().tile_size, Brain::new()));
            }
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
            mode,
            saved_brains,
            parents,
            saver,
            seed,
            start_step
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

        let ready_agents: Vec<ReadyAgent> = self.agents.iter().enumerate()
            .filter(|(_, a)| a.is_ready_to_reproduce())
            .map(|(index, a)| ReadyAgent{ index, tribe: a.tribe(), x: a.get_x(), y: a.get_y() })
            .collect();

        let living_agents: Vec<LivingAgent> = self.agents.iter().enumerate()
            .filter(|(_, a)| a.is_alive())
            .map(|(index, a)| LivingAgent{ index, tribe: a.tribe(), x: a.get_x(), y: a.get_y(), energy: a.energy_ratio() })
            .collect();
        let mut attacks = Vec::new();

        for (index, a) in self.agents.iter_mut().enumerate(){
            let was_alive = a.is_alive();
            let partner = nearest_partner(index, a.tribe(), a.get_x(), a.get_y(), &ready_agents);
            let enemy = nearest_enemy(a.tribe(), a.get_x(), a.get_y(), &living_agents);
            let senses = sense(a, &self.terrain, &self.tribes[a.tribe()], &self.food, partner, enemy);
            if a.update(&self.terrain, &senses){
                if let Some(enemy) = enemy{
                    attacks.push((index, enemy.index));
                }
            }
            if a.is_alive(){
                self.terrain.capture(a.get_x(), a.get_y(), a.tribe());
            }
            else if was_alive{
                self.stats.add_death(a.tribe(), a.age());
            }
        }
        self.terrain.update_capture_status();
        self.resolve_attacks(&attacks);

        self.food.retain(|f| !f.eaten);
        self.agents.retain(|a| !a.is_gone());

        self.reproduce_agents();
        self.respawn_tribes();
        self.spawn_food();
        self.record_stats();

        if self.stats.step() % MODEL_SAVE_INTERVAL == 0{
            self.save_latest();
        }
    }

    fn resolve_attacks(&mut self, attacks: &[(usize, usize)]){
        for &(attacker, target) in attacks{
            let Ok([attacker, target]) = self.agents.get_disjoint_mut([attacker, target]) else {
                continue;
            };
            self.stats.add_attack(attacker.tribe());
            if !attacker.can_hit(target){
                continue;
            }
            self.stats.add_hit(attacker.tribe());
            if attacker.hit(target){
                self.stats.add_death(target.tribe(), target.age());
                self.stats.add_kill(attacker.tribe());
                // Zabojca czasem przekazuje wagi dalej jak rodzic. Przy kazdym zabojstwie
                // lista zapelniala sie zabojcami i agenci przestawali sie rozmnazac.
                if rand::gen_range(0.0, 1.0) < KILLER_PARENT_CHANCE{
                    remember_parent(&mut self.parents[attacker.tribe()], attacker.brain());
                }
            }
        }
    }

    // Kolejnosc plemion od najlepszego. Rozstrzygaja zajete pola, a przy remisie liczba zywych.
    pub fn ranking(&self) -> Vec<usize>{
        let scores: Vec<(usize, usize)> = (0..self.tribes.len())
            .map(|tribe| (self.terrain.owned_tiles(tribe), self.alive_in_tribe(tribe)))
            .collect();
        rank_tribes(&scores)
    }

    pub fn print_ranking(&self){
        for (place, tribe) in self.ranking().into_iter().enumerate(){
            println!("Miejsce {}: plemie {}, pola {}, zywi {}, zabojstwa {}", place + 1, tribe + 1, self.terrain.owned_tiles(tribe), self.alive_in_tribe(tribe), self.stats.total_kills(tribe));
        }
    }

    fn alive_in_tribe(&self, tribe: usize) -> usize{
        self.agents.iter().filter(|a| a.is_alive() && a.tribe() == tribe).count()
    }

    fn to_model(&self) -> Model{
        let tribes = (0..self.tribes.len())
            .map(|tribe| self.agents.iter()
                .filter(|a| a.is_alive() && a.tribe() == tribe)
                .map(|a| a.brain().weights().to_vec())
                .collect())
            .collect();
        let parents = self.parents.iter()
            .map(|pool| pool.iter().map(|brain| brain.weights().to_vec()).collect())
            .collect();

        Model{
            seed: self.seed,
            step: self.start_step + self.stats.step(),
            weight_count: crate::brain::WEIGHT_COUNT,
            tribes,
            parents,
            name: String::new(),
            source: String::new()
        }
    }

    pub fn save_latest(&self){
        if let Some(saver) = &self.saver{
            saver.save_latest(&self.to_model());
        }
    }

    pub fn save_final(&self){
        if let Some(saver) = &self.saver{
            saver.save_final(&self.to_model());
        }
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

        draw_rectangle(STATS_BUTTON_WIDTH + 60.0, 0.0, 90.0, STATS_BUTTON_HEIGHT, panel_color);
        draw_text(if self.mode == Mode::Demo { "Demo" } else { "Trening" }, STATS_BUTTON_WIDTH + 70.0, 18.0, 20.0, WHITE);

        // Demo niczego nie uczy, wiec pokazuje stala liczbe krokow modelu
        let steps = if self.mode == Mode::Demo {
            format!("Model: {} krokow", format_steps(self.start_step))
        } else {
            format!("Kroki: {}", format_steps(self.start_step + self.stats.step()))
        };
        draw_rectangle(STATS_BUTTON_WIDTH + 150.0, 0.0, 230.0, STATS_BUTTON_HEIGHT, panel_color);
        draw_text(&steps, STATS_BUTTON_WIDTH + 160.0, 18.0, 20.0, WHITE);

        if !self.stats_open{
            return;
        }

        draw_rectangle(0.0, STATS_BUTTON_HEIGHT, 800.0, 150.0, panel_color);
        draw_text(&format!("Agenci: {}", self.agents.len()), 10.0, 45.0, 20.0, WHITE);
        draw_text(&format!("Jedzenie: {}", self.food.len()), 10.0, 65.0, 20.0, WHITE);

        for (place, i) in self.ranking().into_iter().enumerate(){
            let text = format!("{}. Plemie {}: pola {}, zywi {}, narodziny {}, smierci {}, odrodzenia {}, zabojstwa {}", place + 1, i + 1, self.terrain.owned_tiles(i), self.alive_in_tribe(i), self.stats.total_births(i), self.stats.total_deaths(i), self.stats.total_respawns(i), self.stats.total_kills(i));
            draw_text(&text, 10.0, 85.0 + place as f32 * 20.0, 20.0, self.tribes[i].color());
        }

        draw_text(&format!("Ziarno: {}", self.stats.seed()), 10.0, 165.0, 20.0, WHITE);
    }

    fn record_stats(&mut self){
        if !self.stats.tick(){
            return;
        }

        for i in 0..self.tribes.len(){
            let alive = self.agents.iter().filter(|a| a.is_alive() && a.tribe() == i).count();
            let in_base: Vec<&Agent> = self.agents.iter()
                .filter(|a| a.is_alive() && a.tribe() == i && self.terrain.is_base_of(a.get_x(), a.get_y(), i))
                .collect();
            let ready_in_base = in_base.iter().filter(|a| a.is_ready_to_reproduce()).count();
            let ready = self.agents.iter().filter(|a| a.tribe() == i && a.is_ready_to_reproduce()).count();
            self.stats.write_row(i, alive, self.terrain.owned_tiles(i), in_base.len(), ready_in_base, ready);
        }
    }

    fn respawn_tribes(&mut self){
        let tile_size = self.terrain.config().tile_size;

        for (i, tribe) in self.tribes.iter().enumerate(){
            let alive: Vec<usize> = (0..self.agents.len())
                .filter(|&index| self.agents[index].is_alive() && self.agents[index].tribe() == i)
                .collect();
            for _ in alive.len()..MIN_TRIBE_POPULATION{
                // Gdy plemie wymarlo, rodzicem moze byc jeszcze ktos z wczytanego modelu
                let saved = self.saved_brains.get(i).map_or(&[][..], |brains| &brains[..]);
                let parent = if self.mode == Mode::Inherit{
                    let pool = &self.parents[i];
                    if pool.is_empty() { None } else { Some(&pool[rand::gen_range(0, pool.len())]) }
                }
                else if !alive.is_empty(){
                    Some(self.agents[alive[rand::gen_range(0, alive.len())]].brain())
                }
                else if !saved.is_empty(){
                    Some(&saved[rand::gen_range(0, saved.len())])
                }
                else{
                    None
                };
                let brain = self.mode.respawn_brain(parent);
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
                    let brain = self.mode.child_brain(a.brain(), b.brain());
                    let pool = &mut self.parents[a.tribe()];
                    remember_parent(pool, a.brain());
                    remember_parent(pool, b.brain());
                    children.push(a.reproduce_with(b, tile_size, brain));
                }
            }
        }

        for child in &children{
            self.stats.add_birth(child.tribe());
        }

        self.agents.extend(children);
    }

    fn spawn_food(&mut self){
        if self.food.len() >= MAX_FOOD{
            return;
        }

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

fn remember_parent(pool: &mut Vec<Brain>, brain: &Brain){
    if pool.len() >= PARENT_POOL_SIZE{
        pool.remove(0);
    }
    pool.push(brain.clone());
}

struct ReadyAgent{
    index: usize,
    tribe: usize,
    x: f32,
    y: f32
}

fn nearest_partner(index: usize, tribe: usize, x: f32, y: f32, ready_agents: &[ReadyAgent]) -> Option<&ReadyAgent>{
    let distance_sq = |other: &ReadyAgent| (other.x - x).powi(2) + (other.y - y).powi(2);
    ready_agents.iter()
        .filter(|other| other.index != index && other.tribe == tribe)
        .min_by(|a, b| distance_sq(a).total_cmp(&distance_sq(b)))
}

struct LivingAgent{
    index: usize,
    tribe: usize,
    x: f32,
    y: f32,
    energy: f32
}

fn nearest_enemy(tribe: usize, x: f32, y: f32, living_agents: &[LivingAgent]) -> Option<&LivingAgent>{
    let distance_sq = |other: &LivingAgent| (other.x - x).powi(2) + (other.y - y).powi(2);
    living_agents.iter()
        .filter(|other| other.tribe != tribe)
        .min_by(|a, b| distance_sq(a).total_cmp(&distance_sq(b)))
}

fn rank_tribes(scores: &[(usize, usize)]) -> Vec<usize>{
    let mut tribes: Vec<usize> = (0..scores.len()).collect();
    tribes.sort_by_key(|&tribe| std::cmp::Reverse(scores[tribe]));
    tribes
}

fn sense(agent: &Agent, terrain: &Terrain, tribe: &Tribe, food: &[Food], partner: Option<&ReadyAgent>, enemy: Option<&LivingAgent>) -> Senses{
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

    let (partner_direction, partner_distance) = match partner{
        Some(p) => (relative_direction(x, y, angle, p.x, p.y), ((p.x - x).powi(2) + (p.y - y).powi(2)).sqrt() / max_distance),
        None => (0.0, 1.0)
    };

    let (enemy_direction, enemy_distance, enemy_energy) = match enemy{
        Some(e) => (relative_direction(x, y, angle, e.x, e.y), ((e.x - x).powi(2) + (e.y - y).powi(2)).sqrt() / max_distance, e.energy),
        None => (0.0, 1.0, 0.0)
    };

    let tile_at_angle = |a: f32| terrain.tile_sense(x + tile_size * a.cos(), y + tile_size * a.sin(), tribe_id);

    Senses{
        energy: agent.energy_ratio(),
        ready_to_reproduce: if agent.is_ready_to_reproduce() { 1.0 } else { 0.0 },
        base_direction: relative_direction(x, y, angle, base_x, base_y),
        base_distance: base_distance / max_distance,
        food_direction,
        food_distance,
        partner_direction,
        partner_distance,
        enemy_direction,
        enemy_distance,
        enemy_energy,
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

#[cfg(test)]
mod tests{
    use super::*;

    fn ready(index: usize, tribe: usize, x: f32) -> ReadyAgent{
        ReadyAgent{ index, tribe, x, y: 0.0 }
    }

    #[test]
    fn parent_pool_drops_the_oldest_parent_when_full(){
        let mut pool = Vec::new();
        let oldest = Brain::new();
        remember_parent(&mut pool, &oldest);
        for _ in 0..PARENT_POOL_SIZE{
            remember_parent(&mut pool, &Brain::new());
        }

        assert_eq!(pool.len(), PARENT_POOL_SIZE);
        assert!(pool.iter().all(|brain| brain.weights() != oldest.weights()));
    }

    #[test]
    fn nearest_partner_is_the_closest_one_from_the_tribe(){
        let agents = [ready(1, 0, 50.0), ready(2, 0, 20.0), ready(3, 1, 5.0)];
        assert_eq!(nearest_partner(0, 0, 0.0, 0.0, &agents).unwrap().index, 2);
    }

    #[test]
    fn agent_is_not_its_own_partner(){
        let agents = [ready(0, 0, 0.0)];
        assert!(nearest_partner(0, 0, 0.0, 0.0, &agents).is_none());
    }

    #[test]
    fn other_tribes_are_not_partners(){
        let agents = [ready(1, 1, 10.0)];
        assert!(nearest_partner(0, 0, 0.0, 0.0, &agents).is_none());
    }

    #[test]
    fn nearest_enemy_is_the_closest_one_from_another_tribe(){
        let living = |index, tribe, x| LivingAgent{ index, tribe, x, y: 0.0, energy: 1.0 };
        let agents = [living(1, 0, 1.0), living(2, 1, 50.0), living(3, 2, 20.0)];
        assert_eq!(nearest_enemy(0, 0.0, 0.0, &agents).unwrap().index, 3);
    }

    #[test]
    fn tribe_without_enemies_sees_nobody(){
        let agents = [LivingAgent{ index: 1, tribe: 0, x: 1.0, y: 0.0, energy: 1.0 }];
        assert!(nearest_enemy(0, 0.0, 0.0, &agents).is_none());
    }

    #[test]
    fn ranking_prefers_tiles_and_breaks_ties_with_living_agents(){
        assert_eq!(rank_tribes(&[(10, 5), (30, 1), (10, 8), (0, 20)]), vec![1, 2, 0, 3]);
    }
}