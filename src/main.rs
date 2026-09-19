use macroquad::prelude::*;
mod world;
mod agent;
mod terrain;

use world::World;

#[macroquad::main("Ewolucja")]
async fn main() {

    let mut world = World::new_with_random_agents(10, 100);


    loop{
        
        clear_background(BLACK);
        
        world.update();
        world.draw();

        next_frame().await
    }
}
