use std::{thread, time::Duration};

use winit::keyboard::{KeyCode, PhysicalKey};

use crate::{
    state::{
        app_state::State,
        control_state::{print_gpu_data, KeyboardMode},
    },
    PheremoneParams, Slime, SlimeParams, ViewParams, NUM_AGENTS, SCREEN_HEIGHT, SCREEN_WIDTH,
};

pub(crate) fn update_view_params_buffer(state: &State) {
    let new_view_params = ViewParams {
        shift_modifier: state.params.view_params.shift_modifier,
        x_shift: state.params.view_params.x_shift,
        y_shift: state.params.view_params.y_shift,
        zoom: state.params.view_params.zoom,
        time_modifier: state.params.view_params.time_modifier,
    };

    state.queue.write_buffer(
        &state.buffers.view_params_buf,
        0,
        bytemuck::cast_slice(&[new_view_params]),
    );
}

pub(crate) fn update_slime_params_buffer(state: &State) {
    let new_slime_params = SlimeParams {
        max_velocity: state.params.slime_params.max_velocity,
        min_velocity: state.params.slime_params.min_velocity,
        turn_factor: state.params.slime_params.turn_factor,
        sensor_dist: state.params.slime_params.sensor_dist,
        sensor_offset: state.params.slime_params.sensor_offset,
        sensor_radius: state.params.slime_params.sensor_radius,
    };

    state.queue.write_buffer(
        &state.buffers.slime_params_buf,
        0,
        bytemuck::cast_slice(&[new_slime_params]),
    );
}

pub(crate) fn update_pheremone_params_buffer(state: &State) {
    let new_pheremone_params = PheremoneParams {
        deposition_amount: state.params.pheremone_params.deposition_amount,
        diffusion_factor: state.params.pheremone_params.diffusion_factor,
        decay_factor: state.params.pheremone_params.decay_factor,
    };

    state.queue.write_buffer(
        &state.buffers.pheremone_params_buf,
        0,
        bytemuck::cast_slice(&[new_pheremone_params]),
    );
}

pub(crate) fn update_cpu_read_buffers(state: &State) {
    let mut encoder = state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("update_cpu_read_buffers encoder"),
        });

    encoder.copy_buffer_to_buffer(
        &state.buffers.slime_pos_buf,
        0,
        &state.buffers.cpu_read_slime_pos_buf,
        0,
        (std::mem::size_of::<[Slime; NUM_AGENTS]>()) as wgpu::BufferAddress,
    );

    encoder.copy_buffer_to_buffer(
        &state.buffers.generic_debug_buf,
        0,
        &state.buffers.cpu_read_generic_debug_buf,
        0,
        (std::mem::size_of::<[f32; 4]>()) as wgpu::BufferAddress,
    );

    encoder.copy_buffer_to_buffer(
        &state.buffers.generic_debug_array_buf,
        0,
        &state.buffers.cpu_read_generic_debug_array_buf,
        0,
        (std::mem::size_of::<[[f32; 4]; NUM_AGENTS]>()) as wgpu::BufferAddress,
    );

    state.queue.submit(Some(encoder.finish()));
}

pub(crate) fn update_agent_position(state: &State) {
    let mut encoder = state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("update_slime_position encoder"),
        });

    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Slime Moves Compute Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&state.pipelines.update_slime);
        compute_pass.set_bind_group(0, &state.bind_groups.compute_bg, &[]);
        compute_pass.set_bind_group(1, &state.bind_groups.uniform_bg, &[]);
        compute_pass.set_bind_group(2, &state.bind_groups.phm_bg, &[]);
        compute_pass.dispatch_workgroups(16, 16, 1); // Adjust workgroup size as needed
    }

    state.queue.submit(Some(encoder.finish()));
}

pub(crate) fn update_pheremone_trails(state: &State) {
    let mut encoder = state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("update_pheremone_trails encoder"),
        });

    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Pheremone Trails Compute Pass"),
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&state.pipelines.update_phm);
        compute_pass.set_bind_group(0, &state.bind_groups.compute_bg, &[]);
        compute_pass.set_bind_group(1, &state.bind_groups.uniform_bg, &[]);
        compute_pass.set_bind_group(2, &state.bind_groups.phm_bg, &[]);

        // Calculate the dispatch size based on the texture dimensions and workgroup size
        let dispatch_size_x = ((SCREEN_WIDTH as u32).saturating_add(32)) / 32;
        let dispatch_size_y = ((SCREEN_HEIGHT as u32).saturating_add(28)) / 28;
        compute_pass.dispatch_workgroups(dispatch_size_x, dispatch_size_y, 1); // Adjust workgroup size as needed
    }

    state.queue.submit(Some(encoder.finish()));
}

pub(crate) fn update_controls(state: &mut State) {
    if state.controls.key_pressed(PhysicalKey::Code(KeyCode::KeyD)) {
        state.controls.set_mode(KeyboardMode::DEBUG);
    } else if state
        .controls
        .key_pressed(PhysicalKey::Code(KeyCode::Digit1))
    {
        state.controls.set_mode(KeyboardMode::VIEW);
    } else if state
        .controls
        .key_pressed(PhysicalKey::Code(KeyCode::Digit2))
    {
        state.controls.set_mode(KeyboardMode::SLIME);
    } else if state
        .controls
        .key_pressed(PhysicalKey::Code(KeyCode::Digit3))
    {
        state.controls.set_mode(KeyboardMode::PHEREMONES);
    } else if state
        .controls
        .key_pressed(PhysicalKey::Code(KeyCode::Digit4))
    {
        state.controls.set_mode(KeyboardMode::PRINT);
    }

    match state.controls.get_mode() {
        KeyboardMode::DEBUG => debug_controls(state),
        KeyboardMode::SLIME => slime_controls(state),
        KeyboardMode::PHEREMONES => pheremone_controls(state),
        KeyboardMode::VIEW => view_controls(state),
        KeyboardMode::PRINT => print_controls(state),
    }
}

fn debug_controls(state: &mut State) {
    let pressed = state.controls.get_keys();

    if pressed.contains(&PhysicalKey::Code(KeyCode::KeyS)) {
        print_gpu_data::<[f32; 4]>(
            &state.device,
            &state.buffers.cpu_read_generic_debug_buf,
            "Debug",
        );
        thread::sleep(Duration::from_millis(50));
        state.controls.set_mode(KeyboardMode::VIEW);
    } else if pressed.contains(&PhysicalKey::Code(KeyCode::KeyA)) {
        print_gpu_data::<[[f32; 4]; NUM_AGENTS]>(
            &state.device,
            &state.buffers.cpu_read_generic_debug_array_buf,
            "Debug",
        );
        thread::sleep(Duration::from_millis(50));
        state.controls.set_mode(KeyboardMode::VIEW);
    }
}

fn slime_controls(state: &mut State) {
    let pressed = state.controls.get_keys();
    let mut dval = 0.0f32;

    if pressed.contains(&PhysicalKey::Code(KeyCode::ArrowUp)) {
        dval = 1.0f32;
    }

    if pressed.contains(&PhysicalKey::Code(KeyCode::ArrowDown)) {
        dval = -1.0f32;
    }

    // MOVEMENT
    if pressed.contains(&PhysicalKey::Code(KeyCode::Period)) {
        let maxv = &mut state.params.slime_params.max_velocity;
        *maxv = f32::max(0.1, *maxv + (0.003 * dval));
        update_slime_params_buffer(state);
    } else if pressed.contains(&PhysicalKey::Code(KeyCode::Comma)) {
        let minv = &mut state.params.slime_params.min_velocity;
        *minv = f32::max(0.0, *minv + (0.003 * dval));
        update_slime_params_buffer(state);
    } else if pressed.contains(&PhysicalKey::Code(KeyCode::KeyT)) {
        let tf = &mut state.params.slime_params.turn_factor;
        *tf = f32::max(0.0, *tf + (0.003 * dval));
        update_slime_params_buffer(state);
    // SENSORS
    } else if pressed.contains(&PhysicalKey::Code(KeyCode::KeyS))
        && pressed.contains(&PhysicalKey::Code(KeyCode::KeyD))
    {
        let tf = &mut state.params.slime_params.sensor_dist;
        *tf = f32::max(0.0, *tf + (0.1 * dval));
        update_slime_params_buffer(state);
    } else if pressed.contains(&PhysicalKey::Code(KeyCode::KeyS))
        && pressed.contains(&PhysicalKey::Code(KeyCode::KeyO))
    {
        let tf = &mut state.params.slime_params.sensor_offset;
        *tf = f32::max(0.0, *tf + (0.1 * dval));
        update_slime_params_buffer(state);
    } else if pressed.contains(&PhysicalKey::Code(KeyCode::KeyS))
        && pressed.contains(&PhysicalKey::Code(KeyCode::KeyR))
    {
        let tf = &mut state.params.slime_params.sensor_radius;
        *tf = f32::max(0.0, *tf + (0.1 * dval));
        update_slime_params_buffer(state);
    }
}

fn pheremone_controls(state: &mut State) {
    let pressed = state.controls.get_keys();
    let mut dval = 0.0f32;

    if pressed.contains(&PhysicalKey::Code(KeyCode::ArrowUp)) {
        dval = 1.0f32;
    }

    if pressed.contains(&PhysicalKey::Code(KeyCode::ArrowDown)) {
        dval = -1.0f32;
    }

    if pressed.contains(&PhysicalKey::Code(KeyCode::KeyA)) {
        let maxv = &mut state.params.pheremone_params.deposition_amount;
        *maxv = f32::max(0.1, *maxv + (0.003 * dval));
        update_pheremone_params_buffer(state);
    } else if pressed.contains(&PhysicalKey::Code(KeyCode::KeyS)) {
        let minv = &mut state.params.pheremone_params.diffusion_factor;
        *minv = f32::max(0.0, *minv + (0.003 * dval));
        update_pheremone_params_buffer(state);
    } else if pressed.contains(&PhysicalKey::Code(KeyCode::KeyD)) {
        let tf = &mut state.params.pheremone_params.decay_factor;
        *tf = f32::max(0.0, *tf + (0.003 * dval));
        update_pheremone_params_buffer(state);
    }
}

fn view_controls(state: &mut State) {
    let pressed = state.controls.get_keys();

    if pressed.contains(&PhysicalKey::Code(KeyCode::ArrowLeft)) {
        state.params.view_params.x_shift -=
            (0.01 * state.params.view_params.shift_modifier) / state.params.view_params.zoom;
        update_view_params_buffer(state);
    } else if pressed.contains(&PhysicalKey::Code(KeyCode::ArrowRight)) {
        state.params.view_params.x_shift +=
            (0.01 * state.params.view_params.shift_modifier) / state.params.view_params.zoom;
        update_view_params_buffer(state);
    }

    if pressed.contains(&PhysicalKey::Code(KeyCode::ArrowUp)) {
        state.params.view_params.y_shift +=
            (0.01 * state.params.view_params.shift_modifier) / state.params.view_params.zoom;
        update_view_params_buffer(state);
    } else if pressed.contains(&PhysicalKey::Code(KeyCode::ArrowDown)) {
        state.params.view_params.y_shift -=
            (0.01 * state.params.view_params.shift_modifier) / state.params.view_params.zoom;
        update_view_params_buffer(state);
    }

    if pressed.contains(&PhysicalKey::Code(KeyCode::PageDown)) {
        state.params.view_params.shift_modifier -= 0.1;
        update_view_params_buffer(state);
    } else if pressed.contains(&PhysicalKey::Code(KeyCode::PageUp)) {
        state.params.view_params.shift_modifier += 0.1;
        update_view_params_buffer(state);
    } else if pressed.contains(&PhysicalKey::Code(KeyCode::KeyX)) {
        let mz = state.params.view_params.zoom;
        state.params.view_params.zoom -= 0.1 * mz;
        update_view_params_buffer(state);
    } else if pressed.contains(&PhysicalKey::Code(KeyCode::KeyZ)) {
        let mz = state.params.view_params.zoom;
        state.params.view_params.zoom += 0.1 * mz;
        update_view_params_buffer(state);
    }
}

fn print_controls(state: &State) {
    let pressed = state.controls.get_keys();

    // PRINT CURRENT FRAME --------------------------------------------------------
    if pressed.contains(&PhysicalKey::Code(KeyCode::Space)) {
        capture_frame_and_save(&state.device, &state.queue, &state.surface);
    }

    // PRINT CURRENT PARAMETER VALUES ----------------------------------------------
    if pressed.contains(&PhysicalKey::Code(KeyCode::KeyI)) {
        println!("\nview_params:\n{:#?}\n", state.params.view_params);
        thread::sleep(Duration::from_millis(50));
    } else if pressed.contains(&PhysicalKey::Code(KeyCode::KeyS)) {
        println!("\nslime_params:\n{:#?}", state.params.slime_params);
        thread::sleep(Duration::from_millis(50));
    } else if pressed.contains(&PhysicalKey::Code(KeyCode::KeyP)) {
        println!("\npheremone_params:\n{:#?}", state.params.pheremone_params);
        thread::sleep(Duration::from_millis(50));
    } else if pressed.contains(&PhysicalKey::Code(KeyCode::Comma)) {
        print_gpu_data::<Slime>(
            &state.device,
            &state.buffers.cpu_read_slime_pos_buf,
            "Slime",
        );
        thread::sleep(Duration::from_millis(50));
    }
}

// TODO! FIX ME!
fn capture_frame_and_save(device: &wgpu::Device, queue: &wgpu::Queue, surface: &wgpu::Surface) {
    // Capture the current frame
    let output = surface
        .get_current_texture()
        .expect("Failed to acquire next swap chain texture");

    // Ensure bytes per row is multiple of 256 as per wgpu standard
    let output_width = ((output.texture.size().width + 255) / 256) * 256;
    let output_height = ((output.texture.size().height + 255) / 256) * 256;

    println!("output_width: {:?}", output_width);
    println!("output_height: {:?}", output_height);
    // Create a buffer to store the frame data
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        size: output_width as wgpu::BufferAddress * output_height as wgpu::BufferAddress * 4, // Assuming RGBA8 format
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        label: Some("Frame Data Buffer"),
        mapped_at_creation: false,
    });

    let render_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Render Texture"),
        size: wgpu::Extent3d {
            width: output_width,
            height: output_height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    // Create a view for the texture
    let render_texture_view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Capture Frame Encoder"),
    });

    {
        // Set up a render pass that targets your texture
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &render_texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });
    }

    // Copy the texture data to the buffer
    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            texture: &render_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyBuffer {
            buffer: &buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(output_width * 4),
                rows_per_image: Some(output_height),
            },
        },
        output.texture.size(),
    );

    queue.submit(Some(encoder.finish()));

    // Wait for the GPU to finish copying the data
    device.poll(wgpu::Maintain::Wait);

    // Map the buffer's memory to the CPU
    let frame_data = buffer
        .slice(..)
        .get_mapped_range()
        .iter()
        .map(|b| *b)
        .collect::<Vec<u8>>();

    // Create an ImageBuffer from the frame data
    let img = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
        output_width,
        output_height,
        &frame_data[..],
    )
    .unwrap();

    // Save the image as a PNG file
    let screenshot_path =
        std::path::Path::new("~/Pictures/wgpu_screenshots").join(format!("{:0}.png", output_width));

    img.save(screenshot_path)
        .expect("Failed to save screenshot");

    // Unmap the buffer's memory
    drop(frame_data);
    buffer.unmap();
}
