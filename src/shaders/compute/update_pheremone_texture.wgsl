const NUM_AGENTS: u32 = 256u;
const SCREEN_WIDTH: f32 = 1376.0;
const SCREEN_HEIGHT: f32 = 768.0;

struct TimeUniform {
    time: f32,
}
struct ConstsUniform {
    phm_height: f32,
    phm_width: f32,
}
struct Slime {
  pos: vec2<f32>,
  vel: vec2<f32>,
  s1_pos: vec2<f32>,
  s2_pos: vec2<f32>,
  s3_pos: vec2<f32>,
}
struct SlimeParams {
  max_velocity: f32,
  min_velocity: f32,
  turn_factor: f32,
  sensor_dist: f32,
  sensor_offset: f32,
  sensor_radius: f32,
}
struct PheremoneParams {
  deposition_amount: f32,
  diffusion_factor: f32,
  decay_factor: f32,
}

// COMPUTE GROUP
@group(0) @binding(0) var<storage, read_write> agents: array<Slime>;
@group(0) @binding(1) var<storage, read_write> sp: SlimeParams;
@group(0) @binding(2) var<storage, read_write> pp: PheremoneParams;
@group(0) @binding(8) var<storage, read_write> debug_arr: array<vec4<f32>>;
@group(0) @binding(9) var<storage, read_write> debug: vec4<f32>;

@group(1) @binding(0) var<uniform> tu: TimeUniform;
@group(1) @binding(1) var<uniform> cu: ConstsUniform;

// TEXTURE GROUP
@group(2) @binding(0) var phm: texture_storage_2d<rgba32float, read_write>;

// ASPECT RATIO
const SCREEN: vec2<f32> = vec2(SCREEN_WIDTH, SCREEN_HEIGHT);
fn scale_tex_aspect(fc: vec2<f32>) -> vec2<f32> {
  // Scale from screen dimensions to 0.0 --> 1.0
  var uv: vec2<f32> = fc / SCREEN;
  return uv;
}

@compute 
@workgroup_size(36, 28, 1) 
fn update_pheremone_heatmap(@builtin(global_invocation_id) id: vec3<u32>) {
    let tcf: vec2<f32> = vec2<f32>(f32(id.x), f32(id.y)); 
    var tex_uv: vec2<f32> = scale_tex_aspect(tcf);
    debug_arr[id.x + id.y] = vec4(tex_uv, tex_uv);
    let range: f32 = 0.01; // Example range
    var populated = 0.0;

    for (var i: u32 = 0u; i < NUM_AGENTS; i++) {
      // Calculate the distance from the agent's position to the current pixel
      let dst: f32 = distance(agents[i].pos, tex_uv);

      // Check if the distance is within the range
      if (dst < range) {
        populated = 1.0;
      }
    }

    // Update the texel value to red
    textureStore(phm, id.xy, vec4(1.0*populated, 0.0, 0.0, 1.0));
}
