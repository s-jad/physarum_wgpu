const NUM_AGENTS: u32 = 256u;
const SCREEN_WIDTH: f32 = 1376.0;
const SCREEN_HEIGHT: f32 = 738.0;

struct TimeUniform {
    time: f32,
}
struct ResolutionUniform {
    xy: vec2<f32>,
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
@group(0) @binding(9) var<storage, read_write> debug: vec4<f32>;

@group(1) @binding(0) var<storage, read_write> tu: TimeUniform;

// TEXTURE GROUP
@group(2) @binding(0) var phm: texture_storage_2d<rgba32float, read_write>;

// ASPECT RATIO
const screen: vec2<f32> = vec2(1366.4, 768.0);

fn scale_aspect(fc: vec2<f32>) -> vec2<f32> {
  // Scale from 0.0 --> 1.0 to -1.0 --> 1.0 
  var uv: vec2<f32> = ((fc * 2.0) - screen) / max(screen.x, screen.y);
  uv.y *= -1.0;
  return uv;
}

@compute 
@workgroup_size(16, 16, 1) 
fn update_pheremone_heatmap(@builtin(global_invocation_id) pos: vec3<u32>) {
    // Calculate the distance from the agent's position to the current pixel
    let texel_coord = pos.xy;
    let tcf: vec2<f32> = vec2<f32>(f32(pos.x), f32(pos.y)); 
    let uv = scale_aspect(tcf);

    for (var i: u32 = 0u; i < NUM_AGENTS; i++) {
      let distance = 1.0; //distance(agents[i].pos, uv);

      let range = 1.0; // Example range

      // Check if the distance is within the range
      if (distance < range) {
        // Update the texture value to 1.0
      //  textureStore(phm, texel_coord, vec4<f32>(1.0, 0.0, 0.0, 1.0));
      }
    }
}
