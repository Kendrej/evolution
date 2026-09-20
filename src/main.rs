use macroquad::prelude::*;
mod world;
mod agent;
mod terrain;

use world::World;

#[macroquad::main("Ewolucja")]
async fn main() {

    let mut world = World::new_with_random_agents(10, 100);


    loop{
        
        clear_background(Color::from_rgba(20, 20, 24, 255));
        
        world.update();
        world.draw();

        next_frame().await
    }
}
