use macroquad::prelude::*;
mod agent;

use agent::Agent;

#[macroquad::main("Ewolucja")]
async fn main() {

    let mut agent =  Agent::new_with_position(10.0, 10.0, 5.0);
    let mut agent1 = Agent::new_with_position(100.0, 50.0, 7.5);
    let mut agent2 = Agent::new_with_position(20.0, 200.0, 10.0);

    let mut agents = vec![agent, agent1, agent2];

    
    


    loop{
        
        clear_background(BLACK);

        for a in &mut agents{
            a.update(screen_height(), screen_width());
            a.draw();
        }

        next_frame().await
    }
}
