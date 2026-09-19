use macroquad::prelude::*;
mod agent;

use agent::Agent;

#[macroquad::main("Ewolucja")]
async fn main() {

    let mut agents =Vec::new();

    for _ in 0..5{
        agents.push(Agent::new_with_random_position(screen_height(), screen_width(), 1.0));
    }


    loop{
        
        clear_background(BLACK);
        
        for a in &mut agents{
            a.update(screen_height(), screen_width());
            a.draw();
        }

        next_frame().await
    }
}
