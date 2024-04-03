use crate::{
    vertices_as_bytes, BindGroups, Buffers, Params, PheremoneParams, Pipelines, ShaderModules,
    Slime, SlimeParams, Textures, TimeUniform, ViewParams, NUM_AGENTS, SCREEN_HEIGHT, SCREEN_WIDTH,
    VERTICES,
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

    let init_slime_desc = wgpu::ShaderModuleDescriptor {
        label: Some("Initial Slime Position Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/compute/init_slime.wgsl").into()),
    };
    let init_slime_shader = device.create_shader_module(init_slime_desc);

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

    ShaderModules {
        v_shader,
        f_shader,
        init_slime_shader,
        update_slime_shader,
        update_phm_shader,
    }
}

pub(crate) fn init_params() -> Params {
    let view_params = ViewParams {
        x_shift: 0.0,
        y_shift: 0.0,
        zoom: 0.01,
        time_modifier: 0.01,
    };

    let slime_params = SlimeParams {
        max_velocity: 0.25,
        min_velocity: -0.25,
        turn_factor: 0.005,
        sensor_dist: 5.0,
        sensor_offset: 1.0472, // 60degrees in Radians
        sensor_radius: 4.0,
    };

    let pheremone_params = PheremoneParams {
        deposition_amount: 1.0,
        diffusion_factor: 0.1,
        decay_factor: 0.1,
    };

    Params {
        view_params,
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

    // PARAMETER BUFFERS
    let view_params_buf = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("Parameters Storage Buffer"),
            contents: bytemuck::cast_slice(&[
                params.view_params.x_shift,
                params.view_params.y_shift,
                params.view_params.zoom,
                params.view_params.time_modifier,
            ]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        },
    );

    let slime_params_buf = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("Slime Parameters Storage Buffer"),
            contents: bytemuck::cast_slice(&[
                params.slime_params.max_velocity,
                params.slime_params.min_velocity,
                params.slime_params.turn_factor,
                params.slime_params.sensor_dist,
                params.slime_params.sensor_offset,
                params.slime_params.sensor_radius,
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
    let slime_pos_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Slimes Positions Buffer"),
        size: (std::mem::size_of::<[Slime; NUM_AGENTS]>()) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let cpu_read_slime_pos_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("CPU Readable Buffer - Slimes"),
        size: (std::mem::size_of::<[Slime; NUM_AGENTS]>()) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

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
        view_params_buf,
        slime_pos_buf,
        cpu_read_slime_pos_buf,
        generic_debug_buf,
        cpu_read_generic_debug_buf,
        slime_params_buf,
        pheremone_params_buf,
    }
}

pub(crate) fn init_bind_groups(
    device: &wgpu::Device,
    buffers: &Buffers,
    texture_view: &wgpu::TextureView,
) -> BindGroups {
    let uniform_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<TimeUniform>() as _),
            },
            count: None,
        }],
        label: Some("uniform_bind_group_layout"),
    });

    let uniform_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &uniform_bgl,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffers.time_uniform_buf.as_entire_binding(),
        }],
        label: Some("uniforms_bind_group"),
    });

    let param_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<ViewParams>() as _),
            },
            count: None,
        }],
        label: Some("variable_bind_group_layout"),
    });

    let param_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &param_bgl,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffers.view_params_buf.as_entire_binding(),
        }],
        label: Some("view_params_bind_group"),
    });

    let compute_bgl =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<Slime>() as _),
                    },
                    count: None,
                },
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
                binding: 0,
                resource: buffers.slime_pos_buf.as_entire_binding(),
            },
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

    let phm_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::ReadWrite,
                    format: wgpu::TextureFormat::Rgba32Float,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
            //       wgpu::BindGroupLayoutEntry {
            //           binding: 1,
            //           visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
            //           ty: BindingType::Sampler(SamplerBindingType::Filtering),
            //           count: None,
            //       },
        ],
        label: Some("phm_bgl"),
    });

    let phm_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &phm_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            //    wgpu::BindGroupEntry {
            //        binding: 1,
            //        resource: wgpu::BindingResource::Sampler(&phm_sampler),
            //    },
        ],
        label: Some("phm_bg"),
    });

    BindGroups {
        uniform_bg,
        uniform_bgl,
        param_bg,
        param_bgl,
        compute_bg,
        compute_bgl,
        phm_bg,
        phm_bgl,
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
            &bind_groups.param_bgl,
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

    let compute_slime_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Agent Pipeline Layout"),
            bind_group_layouts: &[
                &bind_groups.compute_bgl,
                &bind_groups.uniform_bgl,
                &bind_groups.phm_bgl,
            ],
            push_constant_ranges: &[],
        });

    let init_slime_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Slime Initial Position Pipeline"),
        layout: Some(&compute_slime_pipeline_layout),
        module: &shader_modules.init_slime_shader,
        entry_point: "compute_slime_positions",
    });

    let update_slime_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Update Slime Position Pipeline"),
        layout: Some(&compute_slime_pipeline_layout),
        module: &shader_modules.update_slime_shader,
        entry_point: "update_slime_positions",
    });

    let compute_phm_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute PHM Pipeline Layout"),
            bind_group_layouts: &[
                &bind_groups.compute_bgl,
                &bind_groups.uniform_bgl,
                &bind_groups.phm_bgl,
            ],
            push_constant_ranges: &[],
        });

    let update_phm_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Update Pheremone HeatMap Pipeline"),
        layout: Some(&compute_phm_pipeline_layout),
        module: &shader_modules.update_phm_shader,
        entry_point: "update_pheremone_heatmap",
    });

    Pipelines {
        render: render_pipeline,
        init_slime: init_slime_pipeline,
        update_slime: update_slime_pipeline,
        update_phm: update_phm_pipeline,
    }
}

pub(crate) fn init_textures(device: &wgpu::Device) -> Textures {
    let phm_texture_view_desc = wgpu::TextureViewDescriptor {
        label: Some("Phermone Heatmap Texture View"),
        format: Some(wgpu::TextureFormat::Rgba32Float),
        dimension: Some(wgpu::TextureViewDimension::D2),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: Some(1),
        base_array_layer: 0,
        array_layer_count: None,
    };

    let texture_extent = wgpu::Extent3d {
        width: SCREEN_WIDTH,
        height: SCREEN_HEIGHT,
        depth_or_array_layers: 1,
    };

    let pheremone_heat_map = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Read-Write Storage Texture"),
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[wgpu::TextureFormat::Rgba32Float],
    });

    let phm_view = pheremone_heat_map.create_view(&phm_texture_view_desc);

    let phm_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Pheremone Heat Map Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    Textures {
        pheremone_heat_map,
        phm_sampler,
        phm_view,
    }
}
