use macroquad::prelude::*;
use std::f32::consts::{FRAC_PI_2, TAU};
use crate::model::{Model, ModelEntry, ModelStore, format_steps};

const BUTTON_WIDTH: f32 = 660.0;
const BUTTON_HEIGHT: f32 = 32.0;
const BUTTON_GAP: f32 = 6.0;
const STAR_WIDTH: f32 = 32.0;
const EDIT_WIDTH: f32 = 80.0;
const LEFT: f32 = 20.0;
const MAX_NAME_LENGTH: usize = 32;

pub enum Choice{
    New{
        name: String
    },
    Saved{
        model: Model,
        id: String,
        name: String,
        demo: bool
    }
}

enum Screen{
    Start,
    List{ demo: bool },
    NewName,
    Rename{ index: usize, demo: bool }
}

enum RowAction{
    Choose(usize),
    ToggleStar(usize),
    Rename(usize)
}

fn button_at(x: f32, y: f32, width: f32, label: &str) -> bool{
    let (mouse_x, mouse_y) = mouse_position();
    let hovered = mouse_x >= x && mouse_x <= x + width && mouse_y >= y && mouse_y <= y + BUTTON_HEIGHT;
    let color = if hovered { Color::from_rgba(70, 80, 100, 255) } else { Color::from_rgba(45, 50, 62, 255) };

    draw_rectangle(x, y, width, BUTTON_HEIGHT, color);
    draw_text(label, x + 10.0, y + 22.0, 22.0, WHITE);

    hovered && is_mouse_button_pressed(MouseButton::Left)
}

fn button(y: f32, label: &str) -> bool{
    button_at(LEFT, y, BUTTON_WIDTH, label)
}

fn draw_star(center_x: f32, center_y: f32, radius: f32, color: Color){
    let center = vec2(center_x, center_y);
    let point = |index: usize, distance: f32| {
        let angle = index as f32 * TAU / 10.0 - FRAC_PI_2;
        center + vec2(angle.cos(), angle.sin()) * distance
    };

    for tip in (0..10).step_by(2){
        draw_triangle(center, point(tip, radius), point(tip + 1, radius * 0.4), color);
        draw_triangle(center, point(tip, radius), point(tip + 9, radius * 0.4), color);
    }
}

fn star_button(y: f32, starred: bool) -> bool{
    let clicked = button_at(LEFT, y, STAR_WIDTH, "");
    let color = if starred { Color::from_rgba(240, 200, 65, 255) } else { Color::from_rgba(110, 110, 120, 255) };
    draw_star(LEFT + STAR_WIDTH / 2.0, y + BUTTON_HEIGHT / 2.0, 11.0, color);
    clicked
}

fn clear_typed_characters(){
    while get_char_pressed().is_some() {}
}

// Domyslna czcionka ma tylko znaki ASCII, wiec tylko takie mozna wpisac
fn read_name(name: &mut String){
    while let Some(character) = get_char_pressed(){
        if name.len() < MAX_NAME_LENGTH && (character.is_ascii_alphanumeric() || " -_".contains(character)){
            name.push(character);
        }
    }
    if is_key_pressed(KeyCode::Backspace){
        name.pop();
    }
}

fn draw_name_input(title: &str, name: &str){
    draw_text(title, LEFT, 45.0, 28.0, WHITE);
    draw_rectangle(LEFT, 70.0, BUTTON_WIDTH, BUTTON_HEIGHT, Color::from_rgba(45, 50, 62, 255));
    draw_text(&format!("{}_", name), LEFT + 10.0, 92.0, 22.0, WHITE);
    draw_text("Enter: zatwierdz, Esc: anuluj", LEFT, 130.0, 20.0, GRAY);
}

// Zwraca Some(true) po potwierdzeniu, Some(false) po rezygnacji i None, gdy nie ma jeszcze decyzji
pub fn confirm_exit(saves_model: bool) -> Option<bool>{
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::from_rgba(0, 0, 0, 200));
    draw_text("Wrocic do menu?", LEFT, 60.0, 28.0, WHITE);
    let details = if saves_model { "Model zostanie zapisany, a ten przebieg sie zakonczy." } else { "To demo sie zakonczy." };
    draw_text(details, LEFT, 90.0, 22.0, WHITE);

    let confirmed = button(110.0, "Tak (Enter)") || is_key_pressed(KeyCode::Enter);
    let cancelled = button(110.0 + BUTTON_HEIGHT + BUTTON_GAP, "Nie (Esc)") || is_key_pressed(KeyCode::Escape);

    if confirmed{
        Some(true)
    }
    else if cancelled{
        Some(false)
    }
    else{
        None
    }
}

fn draw_rows(entries: &[ModelEntry], first_y: f32) -> Option<RowAction>{
    let row_height = BUTTON_HEIGHT + BUTTON_GAP;
    let rows = ((screen_height() - first_y - 40.0) / row_height).max(1.0) as usize;
    let name_x = LEFT + STAR_WIDTH + BUTTON_GAP;
    let name_width = BUTTON_WIDTH - STAR_WIDTH - EDIT_WIDTH - BUTTON_GAP * 2.0;
    let mut action = None;

    for (index, entry) in entries.iter().take(rows).enumerate(){
        let y = first_y + index as f32 * row_height;
        if star_button(y, entry.starred){
            action = Some(RowAction::ToggleStar(index));
        }
        if button_at(name_x, y, name_width, &entry.name){
            action = Some(RowAction::Choose(index));
        }
        let steps = entry.step.map_or("? krokow".to_string(), |step| format!("{} krokow", format_steps(step)));
        let steps_width = measure_text(&steps, None, 20, 1.0).width;
        draw_text(&steps, name_x + name_width - steps_width - 10.0, y + 22.0, 20.0, LIGHTGRAY);
        if button_at(name_x + name_width + BUTTON_GAP, y, EDIT_WIDTH, "Nazwa"){
            action = Some(RowAction::Rename(index));
        }
    }
    action
}

// Zwraca None, gdy uzytkownik zamknie okno
pub async fn choose(store: &ModelStore) -> Option<Choice>{
    let mut screen = Screen::Start;
    let mut entries = store.list();
    let mut name = String::new();
    let mut error: Option<String> = None;

    loop{
        if is_quit_requested(){
            return None;
        }

        clear_background(Color::from_rgba(20, 20, 24, 255));
        let mut next_screen = None;

        match &screen{
            Screen::Start => {
                draw_text("Ewolucja: co chcesz zrobic?", LEFT, 45.0, 28.0, WHITE);
                if button(70.0, "Trening"){
                    next_screen = Some(Screen::List{ demo: false });
                }
                if button(70.0 + BUTTON_HEIGHT + BUTTON_GAP, "Demo (ogladanie modelu bez nauki)"){
                    next_screen = Some(Screen::List{ demo: true });
                }
            }
            Screen::List{ demo } => {
                let demo = *demo;
                draw_text(if demo { "Demo: wybierz model" } else { "Trening: od czego zaczac?" }, LEFT, 45.0, 28.0, WHITE);

                let mut y = 70.0;
                if button(y, "< Wstecz") || is_key_pressed(KeyCode::Escape){
                    next_screen = Some(Screen::Start);
                }
                y += BUTTON_HEIGHT + BUTTON_GAP;

                if !demo{
                    if button(y, "Nowa populacja"){
                        name.clear();
                        clear_typed_characters();
                        next_screen = Some(Screen::NewName);
                    }
                    y += BUTTON_HEIGHT + BUTTON_GAP;
                }

                if entries.is_empty(){
                    draw_text("Brak zapisanych modeli", LEFT, y + 22.0, 22.0, GRAY);
                }
                else{
                    draw_text("Gwiazdka: model glowny, ten trafia do gita", LEFT, screen_height() - 42.0, 20.0, GRAY);
                }

                match draw_rows(&entries, y){
                    Some(RowAction::Choose(index)) => {
                        let entry = &entries[index];
                        match Model::load(&entry.path){
                            Ok(model) => return Some(Choice::Saved{ model, id: entry.id.clone(), name: entry.name.clone(), demo }),
                            Err(e) => error = Some(format!("Nie udalo sie wczytac modelu: {}", e))
                        }
                    }
                    Some(RowAction::ToggleStar(index)) => {
                        let entry = &entries[index];
                        let result = if entry.starred { store.clear_main(entry) } else { store.set_main(entry) };
                        error = result.err();
                        entries = store.list();
                    }
                    Some(RowAction::Rename(index)) => {
                        name = entries[index].name.clone();
                        clear_typed_characters();
                        next_screen = Some(Screen::Rename{ index, demo });
                    }
                    None => {}
                }
            }
            Screen::NewName => {
                draw_name_input("Nazwa nowej populacji", &name);
                read_name(&mut name);
                if is_key_pressed(KeyCode::Enter) && !name.trim().is_empty(){
                    return Some(Choice::New{ name: name.trim().to_string() });
                }
                if is_key_pressed(KeyCode::Escape){
                    next_screen = Some(Screen::List{ demo: false });
                }
            }
            Screen::Rename{ index, demo } => {
                draw_name_input("Nowa nazwa", &name);
                read_name(&mut name);
                if is_key_pressed(KeyCode::Enter) && !name.trim().is_empty(){
                    error = store.rename(&entries[*index], name.trim()).err();
                    entries = store.list();
                    next_screen = Some(Screen::List{ demo: *demo });
                }
                if is_key_pressed(KeyCode::Escape){
                    next_screen = Some(Screen::List{ demo: *demo });
                }
            }
        }

        if let Some(message) = &error{
            draw_text(message, LEFT, screen_height() - 18.0, 20.0, RED);
        }

        if let Some(new_screen) = next_screen{
            screen = new_screen;
        }

        next_frame().await
    }
}
