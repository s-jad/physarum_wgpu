use crate::{
    state::app_state::State, PheremoneParams, Slime, SlimeParams, ViewParams, DISPATCH_SIZE_X,
    DISPATCH_SIZE_Y, NUM_AGENTS,
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
        avoid_factor: state.params.slime_params.avoid_factor,
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
        deposition_range: state.params.pheremone_params.deposition_range,
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
        compute_pass.dispatch_workgroups(DISPATCH_SIZE_X, DISPATCH_SIZE_Y, 1); // Adjust workgroup size as needed
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
        compute_pass.dispatch_workgroups(DISPATCH_SIZE_X, DISPATCH_SIZE_Y, 1); // Adjust workgroup size as needed
    }

    state.queue.submit(Some(encoder.finish()));
}
