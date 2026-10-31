use macroquad::rand::gen_range;
use std::f32::consts::{PI, TAU};

const INPUTS: usize = 11;
const HIDDEN: usize = 8;
const OUTPUTS: usize = 2;
// Kazdy neuron ma wage na kazde wejscie i jedna dodatkowa stala (bias)
const WEIGHT_COUNT: usize = (INPUTS + 1) * HIDDEN + (HIDDEN + 1) * OUTPUTS;
const MAX_TURN: f32 = 0.1;

pub struct Action{
    pub turn: f32,
    pub speed: f32
}

// Wejscia sieci. Kierunki sa w zakresie od -1 do 1 (ujemne po lewej, 0 na wprost),
// odleglosci od 0 do 1, a pola opisuje Terrain::tile_sense.
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

impl Senses{
    fn to_inputs(&self) -> [f32; INPUTS]{
        [
            self.energy,
            self.ready_to_reproduce,
            self.base_direction,
            self.base_distance,
            self.food_direction,
            self.food_distance,
            self.tile_here,
            self.capture_progress,
            self.tile_ahead,
            self.tile_left,
            self.tile_right
        ]
    }
}

pub struct Brain{
    weights: [f32; WEIGHT_COUNT]
}

impl Brain{
    pub fn new() -> Brain{
        let mut weights = [0.0; WEIGHT_COUNT];
        for weight in &mut weights{
            *weight = gen_range(-1.0, 1.0);
        }
        Brain{
            weights
        }
    }

    pub fn decide(&self, senses: &Senses) -> Action{
        let inputs = senses.to_inputs();
        let (hidden_weights, output_weights) = self.weights.split_at((INPUTS + 1) * HIDDEN);

        let mut hidden = [0.0; HIDDEN];
        for (neuron, weights) in hidden.iter_mut().zip(hidden_weights.chunks(INPUTS + 1)){
            *neuron = activate(&inputs, weights);
        }

        let mut outputs = [0.0; OUTPUTS];
        for (neuron, weights) in outputs.iter_mut().zip(output_weights.chunks(HIDDEN + 1)){
            *neuron = activate(&hidden, weights);
        }

        Action{
            turn: outputs[0] * MAX_TURN,
            speed: (outputs[1] + 1.0) / 2.0
        }
    }
}

// Jeden neuron: bias plus wejscia pomnozone przez wagi, sprowadzone do zakresu od -1 do 1
fn activate(inputs: &[f32], weights: &[f32]) -> f32{
    let mut sum = weights[0];
    for (input, weight) in inputs.iter().zip(&weights[1..]){
        sum += input * weight;
    }
    sum.tanh()
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

    fn senses(value: f32) -> Senses{
        Senses{
            energy: value,
            ready_to_reproduce: value,
            base_direction: value,
            base_distance: value,
            food_direction: value,
            food_distance: value,
            tile_here: value,
            capture_progress: value,
            tile_ahead: value,
            tile_left: value,
            tile_right: value
        }
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

    #[test]
    fn network_has_expected_weight_count(){
        assert_eq!(WEIGHT_COUNT, 114);
    }

    #[test]
    fn neuron_adds_bias_and_weighted_inputs(){
        assert_close(activate(&[0.5, 0.2], &[0.1, 2.0, -1.0]), 0.9_f32.tanh());
    }

    #[test]
    fn outputs_stay_in_range_for_random_networks(){
        for _ in 0..200{
            let brain = Brain::new();
            for value in [-1.0, 0.0, 1.0]{
                let action = brain.decide(&senses(value));
                assert!(action.turn.abs() <= MAX_TURN);
                assert!((0.0..=1.0).contains(&action.speed));
            }
        }
    }

    #[test]
    fn same_network_and_senses_give_same_action(){
        let brain = Brain::new();
        let first = brain.decide(&senses(0.5));
        let second = brain.decide(&senses(0.5));
        assert_eq!(first.turn, second.turn);
        assert_eq!(first.speed, second.speed);
    }

    #[test]
    fn network_can_stop_and_go_at_full_speed(){
        let mut brain = Brain{ weights: [0.0; WEIGHT_COUNT] };
        assert_close(brain.decide(&senses(0.5)).speed, 0.5);

        // Bias neuronu wyjsciowego od predkosci
        let speed_bias = WEIGHT_COUNT - (HIDDEN + 1);
        brain.weights[speed_bias] = -10.0;
        assert_close(brain.decide(&senses(0.5)).speed, 0.0);
        brain.weights[speed_bias] = 10.0;
        assert_close(brain.decide(&senses(0.5)).speed, 1.0);
    }
}
