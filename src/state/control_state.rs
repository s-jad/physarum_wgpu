use std::collections::HashSet;

pub(crate) enum KeyboardMode {
    DEBUG,
    SLIME,
    PHEREMONES,
    VIEW,
}

pub(crate) struct KeyboardState {
    keys: HashSet<winit::keyboard::PhysicalKey>,
    mode: KeyboardMode,
}

impl KeyboardState {
    pub(crate) fn new() -> Self {
        Self {
            keys: HashSet::new(),
            mode: KeyboardMode::VIEW,
        }
    }

    pub(crate) fn key_pressed(&self, key: winit::keyboard::PhysicalKey) -> bool {
        self.keys.contains(&key)
    }

    pub(crate) fn handle_keyboard_input(&mut self, input: &winit::event::KeyEvent) {
        let key = input.physical_key;
        if input.state == winit::event::ElementState::Pressed {
            self.keys.insert(key);
            println!("KeyboardState.keys: {:?}", self.keys);
        } else {
            self.keys.remove(&key);
            println!("KeyboardState.keys: {:?}", self.keys);
        }
    }

    pub(crate) fn clear_keys(&mut self) {
        self.keys.clear();
    }
}
