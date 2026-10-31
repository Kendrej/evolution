use macroquad::prelude::*;
mod world;
mod agent;
mod terrain;
mod tribe;
mod stats;
mod brain;
use world::World;
use std::time::{SystemTime, UNIX_EPOCH};

// Wpisz Some(ziarno), zeby powtorzyc konkretny przebieg
const FIXED_SEED: Option<u32> = None;
const INHERITANCE: bool = true;

fn random_seed() -> u32{
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |t| t.as_nanos() as u32)
}

async fn run_window() {

    let seed = FIXED_SEED.unwrap_or_else(random_seed);
    println!("Ziarno: {}", seed);
    rand::srand(seed as u64);

    let mut world = World::new(40, 100, seed, INHERITANCE);


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

// Przebieg bez okna do porownan: --headless <ziarno> <kroki> <inherit|random>
fn run_headless(args: &[String]) -> Result<(), String>{
    let [seed, steps, mode] = args else {
        return Err("Uzycie: --headless <ziarno> <kroki> <inherit|random>".to_string());
    };
    let seed: u32 = seed.parse().map_err(|_| format!("Bledne ziarno: {}", seed))?;
    let steps: u64 = steps.parse().map_err(|_| format!("Bledna liczba krokow: {}", steps))?;
    let inheritance = match mode.as_str(){
        "inherit" => true,
        "random" => false,
        _ => return Err(format!("Bledny tryb: {}", mode))
    };

    rand::srand(seed as u64);
    let mut world = World::new(40, 100, seed, inheritance);
    for _ in 0..steps{
        world.update();
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.first().is_some_and(|arg| arg == "--headless"){
        if let Err(message) = run_headless(&args[1..]){
            eprintln!("{}", message);
            std::process::exit(1);
        }
    }
    else{
        macroquad::Window::new("Ewolucja", run_window());
    }
}
