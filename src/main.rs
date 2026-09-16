use macroquad::prelude::*;

#[macroquad::main("Ewolucja")]
async fn main() {
    let mut x = 10.0;
    let mut y = 10.0;
    let mut i = 1;
    let mut j = 1;

    loop{
        if x >= screen_width() - 60.0 {
            i = -1;
        }
        else if x <= 0.0 {
            i = 1;
        }
        if y >= screen_height() - 40.0 {
            j = -1;
        }
        else if y <= 0.0 {
            j = 1;
        }
        
        
        x += 1.0 * i as f32;
        y += 1.0 * j as f32;




        clear_background(BLACK);
        draw_rectangle(x, y, 60.0, 40.0, BLUE);
        //draw_circle(x, y, 15.0, RED);
        next_frame().await
    }
}
