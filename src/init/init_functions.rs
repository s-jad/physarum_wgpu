use rand::Rng;
use wgpu::util::DeviceExt;

use crate::{
    vertices_as_bytes, BindGroups, Buffers, ConstUniforms, Params, PheremoneParams, Pipelines,
    ShaderModules, SlimeParams, Textures, TimeUniform, AGENT_TEX_HEIGHT, AGENT_TEX_WIDTH,
    PHM_TEX_BUF_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH, VERTICES,
};

pub(crate) fn init_shader_modules(device: &wgpu::Device) -> ShaderModules {
    let vdesc = wgpu::ShaderModuleDescriptor {
        label: Some("Vertex Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/vertex/v2.wgsl").into()),
    };
    let v_shader = device.create_shader_module(vdesc);

    let fdesc = wgpu::ShaderModuleDescriptor {
        label: Some("Fragment Shader"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("../shaders/fragment/slime_frag.wgsl").into(),
        ),
    };
    let f_shader = device.create_shader_module(fdesc);

    let update_slime_desc = wgpu::ShaderModuleDescriptor {
        label: Some("Update Slime Movement Shader"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("../shaders/compute/slime_movement.wgsl").into(),
        ),
    };
    let update_slime_shader = device.create_shader_module(update_slime_desc);

    let update_phm_desc = wgpu::ShaderModuleDescriptor {
        label: Some("Update Pheremone HeatMap Shader"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("../shaders/compute/update_pheremone_texture.wgsl").into(),
        ),
    };
    let update_phm_shader = device.create_shader_module(update_phm_desc);

    let update_food_desc = wgpu::ShaderModuleDescriptor {
        label: Some("Update Food Shader"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("../shaders/compute/update_food.wgsl").into(),
        ),
    };
    let update_food_shader = device.create_shader_module(update_food_desc);

    ShaderModules {
        v_shader,
        f_shader,
        update_slime_shader,
        update_phm_shader,
        update_food_shader,
    }
}

pub(crate) fn init_params() -> Params {
    let slime_params = SlimeParams {
        max_velocity: 0.003,
        min_velocity: -0.003,
        turn_factor: 0.002,
        avoid_factor: -0.003,
        sensor_dist: 0.01,
        sensor_offset: 1.0472, // 60degrees in Radians
        sensor_radius: 0.001,
        brownian_offset: 0.000005,
    };

    let pheremone_params = PheremoneParams {
        deposition_amount: 0.001,
        diffusion_factor: 0.1,
        decay_factor: 0.985,
    };

    Params {
        slime_params,
        pheremone_params,
    }
}

pub(crate) fn init_buffers(device: &wgpu::Device, params: &Params) -> Buffers {
    let vertices_bytes = vertices_as_bytes(&VERTICES[..]);
    let vertex_buf = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: vertices_bytes,
            usage: wgpu::BufferUsages::VERTEX,
        },
    );

    // UNIFORM BUFFERS
    let time_uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Time Uniform Buffer"),
        size: std::mem::size_of::<f32>() as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let const_uniform_buf = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("Texture Extent Buffer"),
            contents: bytemuck::cast_slice(&[ConstUniforms {
                texture_height: SCREEN_HEIGHT as f32,
                texture_width: SCREEN_WIDTH as f32,
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        },
    );

    let food_coords_buf = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("Texture Extent Buffer"),
            contents: bytemuck::cast_slice(&[
                [SCREEN_WIDTH / 3, SCREEN_HEIGHT / 4],
                [SCREEN_WIDTH / 5 * 2, SCREEN_HEIGHT / 4 * 3],
                [SCREEN_WIDTH / 4 * 2, SCREEN_HEIGHT / 3 * 2],
                [SCREEN_WIDTH / 7 * 3, SCREEN_HEIGHT / 11 * 2],
            ]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        },
    );

    // PARAMETER BUFFERS
    let slime_params_buf = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("Slime Parameters Storage Buffer"),
            contents: bytemuck::cast_slice(&[
                params.slime_params.max_velocity,
                params.slime_params.min_velocity,
                params.slime_params.turn_factor,
                params.slime_params.avoid_factor,
                params.slime_params.sensor_dist,
                params.slime_params.sensor_offset,
                params.slime_params.sensor_radius,
                params.slime_params.brownian_offset,
            ]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        },
    );

    let pheremone_params_buf = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("Pheremone Parameters Storage Buffer"),
            contents: bytemuck::cast_slice(&[
                params.pheremone_params.deposition_amount,
                params.pheremone_params.diffusion_factor,
                params.pheremone_params.decay_factor,
            ]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        },
    );

    // STORAGE/CPU-READABLE BUFFER PAIRS
    let generic_debug_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Debug Shaders Buffer"),
        size: (std::mem::size_of::<[f32; 4]>()) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let cpu_read_generic_debug_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("CPU Readable Buffer - Debug Shaders"),
        size: (std::mem::size_of::<[f32; 4]>()) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    Buffers {
        vertex_buf,
        time_uniform_buf,
        const_uniform_buf,
        food_coords_buf,
        generic_debug_buf,
        cpu_read_generic_debug_buf,
        slime_params_buf,
        pheremone_params_buf,
    }
}

pub(crate) fn init_bind_groups(
    device: &wgpu::Device,
    buffers: &Buffers,
    textures: &Textures,
) -> BindGroups {
    let uniform_bgl =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<TimeUniform>() as _
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<ConstUniforms>() as _,
                        ),
                    },
                    count: None,
                },
            ],
            label: Some("uniform_bind_group_layout"),
        });

    let uniform_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &uniform_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: buffers.time_uniform_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: buffers.const_uniform_buf.as_entire_binding(),
            },
        ],
        label: Some("uniforms_bind_group"),
    });

    let compute_bgl =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<SlimeParams>() as _
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<PheremoneParams>() as _,
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 9,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<[f32; 4]>() as _
                        ),
                    },
                    count: None,
                },
            ],
            label: Some("compute_bind_group_layout"),
        });

    let compute_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &compute_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 1,
                resource: buffers.slime_params_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: buffers.pheremone_params_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 9,
                resource: buffers.generic_debug_buf.as_entire_binding(),
            },
        ],
        label: Some("compute_bind_group"),
    });

    let texture_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::ReadWrite,
                    format: wgpu::TextureFormat::Rgba32Float,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::ReadWrite,
                    format: wgpu::TextureFormat::Rgba32Float,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
        ],
        label: Some("texture_bgl"),
    });

    let texture_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &texture_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&textures.phm_tex_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&textures.agent_tex_view),
            },
        ],
        label: Some("texture_bg"),
    });

    let sampled_texture_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::ReadWrite,
                    format: wgpu::TextureFormat::Rgba32Float,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
        ],
        label: Some("sampled_texture_bgl"),
    });

    let sampled_texture_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &sampled_texture_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&textures.phm_tex_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&textures.phm_tex_sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&textures.agent_tex_view),
            },
        ],
        label: Some("sampled_texture_bg"),
    });

    let food_bgl =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::Rgba32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<[u32; 2]>() as _
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<TimeUniform>() as _
                        ),
                    },
                    count: None,
                },
            ],
            label: Some("food_bgl"),
        });

    let food_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &food_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&textures.phm_tex_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: buffers.food_coords_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: buffers.time_uniform_buf.as_entire_binding(),
            },
        ],
        label: Some("food_bg"),
    });

    BindGroups {
        uniform_bg,
        uniform_bgl,
        compute_bg,
        compute_bgl,
        texture_bg,
        texture_bgl,
        sampled_texture_bg,
        sampled_texture_bgl,
        food_bg,
        food_bgl,
    }
}

pub(crate) fn init_pipelines(
    device: &wgpu::Device,
    bind_groups: &BindGroups,
    shader_modules: &ShaderModules,
) -> Pipelines {
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[
            &bind_groups.compute_bgl,
            &bind_groups.uniform_bgl,
            &bind_groups.sampled_texture_bgl,
        ],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_modules.v_shader,
            entry_point: "main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: 8, // 2 * 4byte float
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![
                    0 => Float32x2,
                    1 => Float32x2,
                ],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_modules.f_shader,
            entry_point: "main",
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Compute Agent Pipeline Layout"),
        bind_group_layouts: &[
            &bind_groups.compute_bgl,
            &bind_groups.uniform_bgl,
            &bind_groups.texture_bgl,
        ],
        push_constant_ranges: &[],
    });

    let update_slime_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Update Agent Position Pipeline"),
        layout: Some(&compute_pipeline_layout),
        module: &shader_modules.update_slime_shader,
        entry_point: "update_slime_positions",
    });

    let update_phm_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Update Pheremone HeatMap Pipeline"),
        layout: Some(&compute_pipeline_layout),
        module: &shader_modules.update_phm_shader,
        entry_point: "update_pheremone_heatmap",
    });

    let food_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Compute Food Layout"),
        bind_group_layouts: &[&bind_groups.food_bgl],
        push_constant_ranges: &[],
    });

    let update_food_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Update Food Pipeline"),
        layout: Some(&food_pipeline_layout),
        module: &shader_modules.update_food_shader,
        entry_point: "update_food",
    });

    Pipelines {
        render: render_pipeline,
        update_slime: update_slime_pipeline,
        update_phm: update_phm_pipeline,
        update_food: update_food_pipeline,
    }
}

fn init_agents_data(num_agents: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let mut agents_data = Vec::with_capacity(num_agents * 4); // each agent has 4 params

    for i in 0..num_agents {
        let (pos_x, pos_y) = match i % 5 {
            0 => (0.89, 0.73),
            1 => (0.18, 0.86),
            2 => (0.54, 0.47),
            3 => (0.73, 0.17),
            4 => (0.27, 0.33),
            _ => (0.5, 0.5),
        };

        // Generate random velocity between sp.min_velocity and sp.max_velocity
        let vel_x = rng.gen_range(-0.002..=0.002);
        let vel_y = rng.gen_range(-0.004..=0.004);

        // Encode position and velocity into a vec4
        let agent_data = [pos_x, pos_y, vel_x, vel_y];

        agents_data.push(agent_data);
    }

    agents_data.into_iter().flatten().collect::<Vec<f32>>()
}

pub(crate) fn init_textures(device: &wgpu::Device, queue: &wgpu::Queue) -> Textures {
    let phm_tex_view_desc = wgpu::TextureViewDescriptor {
        label: Some("Phermone - View Descriptor"),
        format: Some(wgpu::TextureFormat::Rgba32Float),
        dimension: Some(wgpu::TextureViewDimension::D2),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: Some(1),
        base_array_layer: 0,
        array_layer_count: None,
    };

    let phm_tex_extent = wgpu::Extent3d {
        width: SCREEN_WIDTH,
        height: SCREEN_HEIGHT,
        depth_or_array_layers: 1,
    };

    let phm_tex = device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label: Some("Pheremone - Read-Write Storage Texture"),
            size: phm_tex_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Rgba32Float],
        },
        wgpu::util::TextureDataOrder::default(),
        &[0; PHM_TEX_BUF_SIZE],
    );

    let phm_tex_view = phm_tex.create_view(&phm_tex_view_desc);

    let phm_tex_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Pheremone - Sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    let agent_tex_view_desc = wgpu::TextureViewDescriptor {
        label: Some("Agent - Texture View Descriptor"),
        format: Some(wgpu::TextureFormat::Rgba32Float),
        dimension: Some(wgpu::TextureViewDimension::D2),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: Some(1),
        base_array_layer: 0,
        array_layer_count: None,
    };

    let agent_tex_extent = wgpu::Extent3d {
        width: AGENT_TEX_WIDTH as u32,
        height: AGENT_TEX_HEIGHT as u32,
        depth_or_array_layers: 1,
    };

    let init_agent_tex = init_agents_data(AGENT_TEX_WIDTH * AGENT_TEX_HEIGHT);

    let agent_tex = device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label: Some("Agent - Read-Write Storage Texture"),
            size: agent_tex_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Rgba32Float],
        },
        wgpu::util::TextureDataOrder::default(),
        bytemuck::cast_slice(&init_agent_tex),
    );

    let agent_tex_view = agent_tex.create_view(&agent_tex_view_desc);

    let agent_tex_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Agent - Sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    Textures {
        phm_tex,
        phm_tex_sampler,
        phm_tex_view,
        phm_tex_extent,
        agent_tex,
        agent_tex_sampler,
        agent_tex_view,
        agent_tex_extent,
    }
}
