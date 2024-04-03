use std::collections::HashSet;

#[derive(Debug, Copy, Clone)]
pub(crate) enum KeyboardMode {
    DEBUG,
    SLIME,
    PHEREMONES,
    VIEW,
}

#[derive(Debug, Clone)]
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
        } else {
            self.keys.remove(&key);
        }
    }

    pub(crate) fn clear_keys(&mut self) {
        self.keys.clear();
    }

    pub(crate) fn get_mode(&self) -> KeyboardMode {
        self.mode
    }

    pub(crate) fn set_mode(&mut self, new_mode: KeyboardMode) {
        self.mode = new_mode;
    }
}

pub(crate) fn print_gpu_data<T: bytemuck::Pod + std::fmt::Debug>(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    obj_label: &str,
) {
    // Map the buffer for reading
    let buffer_slice = buffer.slice(..);
    let (tx, rx) = futures::channel::oneshot::channel();

    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).unwrap();
    });

    // Wait for the GPU to finish executing the commands
    device.poll(wgpu::Maintain::Wait);
    // Wait for the buffer to be mapped
    let result = futures::executor::block_on(rx);

    match result {
        Ok(_) => {
            let buf_view = buffer_slice.get_mapped_range();
            let data: &[T] = bytemuck::cast_slice(&buf_view);

            // Print the boids current properties
            for (i, obj) in data.iter().enumerate() {
                println!("{} {}:\n{:?}", obj_label, i, obj);
            }

            drop(buf_view);
            buffer.unmap();
        }
        Err(e) => eprintln!("Error retrieving gpu data: {:?}", e),
    }
}
