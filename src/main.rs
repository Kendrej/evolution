use macroquad::prelude::*;
mod world;
mod agent;
mod terrain;
mod tribe;
mod stats;
mod brain;
mod model;
mod menu;
use world::{Mode, ROUND_STEPS, World};
use model::{Model, ModelStore};
use menu::Choice;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

// Wpisz Some(ziarno), zeby powtorzyc konkretny przebieg
const FIXED_SEED: Option<u32> = None;
const INHERITANCE: bool = true;
const PLOT_SCRIPT: &str = "tools/plot_run.py";

fn random_seed() -> u32{
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |t| t.as_nanos() as u32)
}

async fn run_window() {
    prevent_quit();

    let store = ModelStore::new();
    while let Some(choice) = menu::choose(&store).await {
        if !run_simulation(choice, &store).await {
            return;
        }
    }
}

fn plot_latest_run(mode: Mode){
    if mode == Mode::Demo{
        return;
    }
    if let Err(e) = Command::new("python").arg(PLOT_SCRIPT).spawn(){
        eprintln!("Nie udalo sie narysowac wykresow: {}", e);
    }
}

// Zwraca true, gdy uzytkownik chce wrocic do menu, a false, gdy zamyka okno
async fn run_simulation(choice: Choice, store: &ModelStore) -> bool {
    let seed = FIXED_SEED.unwrap_or_else(random_seed);
    println!("Ziarno: {}", seed);
    rand::srand(seed as u64);

    let training = if INHERITANCE { Mode::Inherit } else { Mode::Random };
    let (mode, model, saver) = match choice {
        Choice::New{ name } => {
            let saver = training.name().map(|mode_name| store.new_population(seed, mode_name, &name));
            (training, None, saver)
        }
        Choice::Saved{ model, id, name, demo: false } => (training, Some(model), Some(store.resume_population(&id, &name))),
        Choice::Saved{ model, demo: true, .. } => (Mode::Demo, Some(model), None)
    };
    let mut world = World::new(40, 100, seed, mode, model, saver, false);
    let mut confirming_exit = false;


    loop{
        if is_quit_requested(){
            world.save_latest();
            plot_latest_run(mode);
            return false;
        }

        clear_background(Color::from_rgba(20, 20, 24, 255));

        if !confirming_exit{
            world.handle_input();
            for _ in 0..world.steps_per_frame(){
                world.update();
            }
        }
        world.draw();

        if confirming_exit{
            match menu::confirm_exit(mode != Mode::Demo){
                Some(true) => {
                    world.save_latest();
                    plot_latest_run(mode);
                    return true;
                }
                Some(false) => confirming_exit = false,
                None => {}
            }
        }
        else if is_key_pressed(KeyCode::Escape){
            confirming_exit = true;
        }

        next_frame().await
    }
}

// Przebieg bez okna do porownan: --headless <ziarno> <kroki> <inherit|random> [kroki_rundy, 0 to bez rund]
fn run_headless(args: &[String]) -> Result<(), String>{
    let [seed, steps, mode, round_steps @ ..] = args else {
        return Err("Uzycie: --headless <ziarno> <kroki> <inherit|random> [kroki_rundy]".to_string());
    };
    let round_steps: u64 = match round_steps{
        [] => ROUND_STEPS,
        [value] => value.parse().map_err(|_| format!("Bledne kroki rundy: {}", value))?,
        _ => return Err("Za duzo argumentow".to_string())
    };
    let seed: u32 = seed.parse().map_err(|_| format!("Bledne ziarno: {}", seed))?;
    let steps: u64 = steps.parse().map_err(|_| format!("Bledna liczba krokow: {}", steps))?;
    let mode = match mode.as_str(){
        "inherit" => Mode::Inherit,
        "random" => Mode::Random,
        _ => return Err(format!("Bledny tryb: {}", mode))
    };

    rand::srand(seed as u64);
    let saver = mode.name().map(|name| ModelStore::new().comparison(seed, name));
    if let Some(saver) = &saver{
        println!("Model: {}", saver.final_path().display());
    }
    let mut world = World::new(40, 100, seed, mode, None, saver, true);
    world.set_round_steps(round_steps);
    for _ in 0..steps{
        world.update();
    }
    world.save_final();
    world.print_ranking();
    Ok(())
}

// Mecz bez nauki: --match <model_1> <model_2> <ziarno> <kroki>. Plemiona 1 i 2 to pierwszy model, 3 i 4 drugi.
fn run_match(args: &[String]) -> Result<(), String>{
    let [first, second, seed, steps] = args else {
        return Err("Uzycie: --match <model_1> <model_2> <ziarno> <kroki>".to_string());
    };
    let seed: u32 = seed.parse().map_err(|_| format!("Bledne ziarno: {}", seed))?;
    let steps: u64 = steps.parse().map_err(|_| format!("Bledna liczba krokow: {}", steps))?;
    let first = Model::load(Path::new(first))?;
    let second = Model::load(Path::new(second))?;

    rand::srand(seed as u64);
    let duel = Model::for_match(&first, &second);
    let mut world = World::new(40, 100, seed, Mode::Demo, Some(duel), None, true);
    for _ in 0..steps{
        world.update();
    }
    world.print_ranking();
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let result = match args.first().map(String::as_str){
        Some("--headless") => Some(run_headless(&args[1..])),
        Some("--match") => Some(run_match(&args[1..])),
        _ => None
    };

    if let Some(result) = result{
        if let Err(message) = result{
            eprintln!("{}", message);
            std::process::exit(1);
        }
    }
    else{
        macroquad::Window::new("Ewolucja", run_window());
    }
}
