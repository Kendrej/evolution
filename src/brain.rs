use macroquad::rand::gen_range;
use std::f32::consts::{PI, TAU};

pub struct Action{
    pub turn: f32,
    pub speed: f32
}

// Wejscia przyszlej sieci. Kierunki sa w zakresie od -1 do 1 (ujemne po lewej, 0 na wprost),
// odleglosci od 0 do 1, a pola opisuje Terrain::tile_sense.
#[allow(dead_code)]
pub struct Senses{
    pub energy: f32,
    pub ready_to_reproduce: f32,
    pub base_direction: f32,
    pub base_distance: f32,
    pub food_direction: f32,
    pub food_distance: f32,
    pub tile_here: f32,
    pub capture_progress: f32,
    pub tile_ahead: f32,
    pub tile_left: f32,
    pub tile_right: f32
}

pub struct Brain;

impl Brain{
    pub fn new() -> Brain{
        Brain
    }

    pub fn decide(&self, _senses: &Senses) -> Action{
        Action{
            turn: gen_range(-0.1, 0.1),
            speed: 1.0
        }
    }
}

pub fn relative_direction(x: f32, y: f32, angle: f32, target_x: f32, target_y: f32) -> f32{
    let target_angle = (target_y - y).atan2(target_x - x);
    ((target_angle - angle + PI).rem_euclid(TAU) - PI) / PI
}

#[cfg(test)]
mod tests{
    use super::*;

    fn assert_close(a: f32, b: f32){
        assert!((a - b).abs() < 0.001, "{} != {}", a, b);
    }

    #[test]
    fn target_straight_ahead_is_zero(){
        assert_close(relative_direction(0.0, 0.0, 0.0, 10.0, 0.0), 0.0);
    }

    #[test]
    fn target_on_the_sides_has_opposite_signs(){
        assert_close(relative_direction(0.0, 0.0, 0.0, 0.0, 10.0), 0.5);
        assert_close(relative_direction(0.0, 0.0, 0.0, 0.0, -10.0), -0.5);
    }

    #[test]
    fn target_behind_is_at_the_end_of_the_range(){
        assert_close(relative_direction(0.0, 0.0, 0.0, -10.0, 0.0).abs(), 1.0);
    }

    #[test]
    fn large_angle_is_wrapped(){
        assert_close(relative_direction(0.0, 0.0, TAU * 5.0, 10.0, 0.0), 0.0);
    }
}
