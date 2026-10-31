use macroquad::prelude::*;
mod world;
mod agent;
mod terrain;
mod tribe;
mod stats;
use world::World;
use std::time::{SystemTime, UNIX_EPOCH};

// Wpisz Some(ziarno), zeby powtorzyc konkretny przebieg
const FIXED_SEED: Option<u32> = None;

fn random_seed() -> u32{
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |t| t.as_nanos() as u32)
}

#[macroquad::main("Ewolucja")]
async fn main() {

    let seed = FIXED_SEED.unwrap_or_else(random_seed);
    println!("Ziarno: {}", seed);
    rand::srand(seed as u64);

    let mut world = World::new(40, 100, seed);


    loop{

        clear_background(Color::from_rgba(20, 20, 24, 255));

        world.handle_input();
        for _ in 0..world.steps_per_frame(){
            world.update();
        }
        world.draw();

        next_frame().await
    }
}
