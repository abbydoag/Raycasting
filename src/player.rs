use std::f32::consts::PI;
use gilrs::{Button, Gamepad};
use nalgebra_glm::Vec2;
use minifb::{Key, Window};

pub struct Player {
    pub pos: Vec2,
    pub a: f32, // angulo vista
    pub fov: f32 // campo vista
}

const MOV_SPEED: f32 = 7.0;
const ROTATION_SPEED: f32 = PI / 20.0;
//keyboard
pub fn process_events(window: &Window, player: &mut Player){  
    let cos_a = player.a.cos();
    let sin_a = player.a.sin();  
    if window.is_key_down(Key::Left){
        player.a -= ROTATION_SPEED
    }
    if window.is_key_down(Key::Right){
        player.a += ROTATION_SPEED
    }
    if window.is_key_down(Key::Up) {
        player.pos.x += MOV_SPEED * cos_a;
        player.pos.y += MOV_SPEED * sin_a;
    }
    if window.is_key_down(Key::Down) {
        player.pos.x -= MOV_SPEED * cos_a;
        player.pos.y -= MOV_SPEED * sin_a;
    }
}
//gamepad (en teoría, no tengo)
pub fn process_gamepad_events(gamepad: &Gamepad, player: &mut Player) {
    let cos_a = player.a.cos();
    let sin_a = player.a.sin();
    if gamepad.is_pressed(Button::DPadLeft) {
        player.a -= ROTATION_SPEED;
    }
    if gamepad.is_pressed(Button::DPadRight) {
        player.a += ROTATION_SPEED;
    }
    if gamepad.is_pressed(Button::DPadUp) {
        player.pos.x += MOV_SPEED * cos_a;
        player.pos.y += MOV_SPEED * sin_a;
    }
    if gamepad.is_pressed(Button::DPadDown) {
        player.pos.x -= MOV_SPEED * cos_a;
        player.pos.y -= MOV_SPEED * sin_a;
    }
}