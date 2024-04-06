pub(crate) const NUM_AGENTS: usize = 256;
pub(crate) const SCREEN_WIDTH: u32 = 1376;
pub(crate) const SCREEN_HEIGHT: u32 = 768;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Vertex {
    pub(crate) position: [f32; 2],
}

pub(crate) const VERTICES: &[Vertex; 6] = &[
    // Bottom left triangle
    Vertex {
        position: [-1.0, -1.0],
    },
    Vertex {
        position: [1.0, -1.0],
    },
    Vertex {
        position: [-1.0, 1.0],
    },
    // Top right triangle
    Vertex {
        position: [1.0, -1.0],
    },
    Vertex {
        position: [1.0, 1.0],
    },
    Vertex {
        position: [-1.0, 1.0],
    },
];

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct TimeUniform {
    pub(crate) time: f32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ConstUniforms {
    pub(crate) phm_height: f32,
    pub(crate) phm_width: f32,
}

#[derive(Debug)]
pub(crate) struct Buffers {
    pub(crate) vertex_buf: wgpu::Buffer,
    pub(crate) time_uniform_buf: wgpu::Buffer,
    pub(crate) const_uniform_buf: wgpu::Buffer,
    pub(crate) view_params_buf: wgpu::Buffer,
    pub(crate) slime_pos_buf: wgpu::Buffer,
    pub(crate) cpu_read_slime_pos_buf: wgpu::Buffer,
    pub(crate) generic_debug_buf: wgpu::Buffer,
    pub(crate) cpu_read_generic_debug_buf: wgpu::Buffer,
    pub(crate) generic_debug_array_buf: wgpu::Buffer,
    pub(crate) cpu_read_generic_debug_array_buf: wgpu::Buffer,
    pub(crate) slime_params_buf: wgpu::Buffer,
    pub(crate) pheremone_params_buf: wgpu::Buffer,
}
#[derive(Debug)]
pub(crate) struct BindGroups {
    pub(crate) uniform_bg: wgpu::BindGroup,
    pub(crate) uniform_bgl: wgpu::BindGroupLayout,
    pub(crate) param_bg: wgpu::BindGroup,
    pub(crate) param_bgl: wgpu::BindGroupLayout,
    pub(crate) compute_bg: wgpu::BindGroup,
    pub(crate) compute_bgl: wgpu::BindGroupLayout,
    pub(crate) phm_bg: wgpu::BindGroup,
    pub(crate) phm_bgl: wgpu::BindGroupLayout,
    pub(crate) sampled_phm_bg: wgpu::BindGroup,
    pub(crate) sampled_phm_bgl: wgpu::BindGroupLayout,
}

#[derive(Debug)]
pub(crate) struct ShaderModules {
    pub(crate) v_shader: wgpu::ShaderModule,
    pub(crate) f_shader: wgpu::ShaderModule,
    pub(crate) init_slime_shader: wgpu::ShaderModule,
    pub(crate) update_slime_shader: wgpu::ShaderModule,
    pub(crate) update_phm_shader: wgpu::ShaderModule,
}

#[derive(Debug)]
pub(crate) struct Pipelines {
    pub(crate) render: wgpu::RenderPipeline,
    pub(crate) init_slime: wgpu::ComputePipeline,
    pub(crate) update_slime: wgpu::ComputePipeline,
    pub(crate) update_phm: wgpu::ComputePipeline,
}

#[derive(Debug)]
pub(crate) struct Textures {
    pub(crate) phm: wgpu::Texture,
    pub(crate) phm_sampler: wgpu::Sampler,
    pub(crate) phm_view: wgpu::TextureView,
    pub(crate) phm_extent: wgpu::Extent3d,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Slime {
    pub(crate) pos: [f32; 2],
    pub(crate) vel: [f32; 2],
    pub(crate) s1_pos: [f32; 2],
    pub(crate) s2_pos: [f32; 2],
    pub(crate) s3_pos: [f32; 2],
}

// PARAMETERS
#[derive(Debug)]
pub(crate) struct Params {
    pub(crate) view_params: ViewParams,
    pub(crate) slime_params: SlimeParams,
    pub(crate) pheremone_params: PheremoneParams,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct SlimeParams {
    pub(crate) max_velocity: f32,
    pub(crate) min_velocity: f32,
    pub(crate) turn_factor: f32,
    pub(crate) sensor_dist: f32,
    pub(crate) sensor_offset: f32,
    pub(crate) sensor_radius: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct PheremoneParams {
    pub(crate) deposition_amount: f32,
    pub(crate) diffusion_factor: f32,
    pub(crate) decay_factor: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ViewParams {
    pub(crate) shift_modifier: f32,
    pub(crate) x_shift: f32,
    pub(crate) y_shift: f32,
    pub(crate) zoom: f32,
    pub(crate) time_modifier: f32,
}
