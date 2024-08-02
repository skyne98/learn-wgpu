use std::collections::HashSet;

use winit::{
    event::{ElementState, KeyEvent, MouseButton},
    keyboard::{KeyCode, PhysicalKey},
};

pub struct InputState {
    pub pressed_keys: HashSet<KeyCode>,
    pub mouse_pressed_keys: HashSet<MouseButton>,
    pub mouse_position: (f64, f64),
}

impl InputState {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            mouse_pressed_keys: HashSet::new(),
            mouse_position: (0.0, 0.0),
        }
    }

    pub fn update(&mut self, input: &KeyEvent) {
        let code = match input.physical_key {
            PhysicalKey::Code(key) => key,
            _ => return,
        };
        match input.state {
            ElementState::Pressed => {
                self.pressed_keys.insert(code);
            }
            ElementState::Released => {
                self.pressed_keys.remove(&code);
            }
        }
    }
    pub fn update_mouse(&mut self, input: &winit::event::MouseButton, state: ElementState) {
        match state {
            ElementState::Pressed => {
                self.mouse_pressed_keys.insert(input.clone());
            }
            ElementState::Released => {
                self.mouse_pressed_keys.remove(input);
            }
        }
    }
    pub fn update_mouse_position(&mut self, x: f64, y: f64) {
        self.mouse_position = (x, y);
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }
    pub fn is_mouse_pressed(&self, key: MouseButton) -> bool {
        self.mouse_pressed_keys.contains(&key)
    }
}
