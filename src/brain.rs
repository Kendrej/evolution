use macroquad::rand::gen_range;

pub struct Action{
    pub turn: f32
}

pub struct Brain;

impl Brain{
    pub fn new() -> Brain{
        Brain
    }

    pub fn decide(&self) -> Action{
        Action{
            turn: gen_range(-0.1, 0.1)
        }
    }
}
