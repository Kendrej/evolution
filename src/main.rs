use macroquad::prelude::*;

#[macroquad::main("Ewolucja")]
async fn main() {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut i = 1.0;

    loop{
        if x<screen_width()-5.0 {
            x += 1.0;
        }
        if y<screen_height()-5.0 {
            y += 1.0;
        }
        clear_background(BLACK);
        draw_circle(x, y, 15.0, RED);
        next_frame().await
    }
}
