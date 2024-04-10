const NUM_AGENTS: u32 = 256u;
const SCREEN_WIDTH: f32 = 1376.0;
const SCREEN_HEIGHT: f32 = 768.0;
const I_SCREEN_WIDTH: i32 = 1376;
const I_SCREEN_HEIGHT: i32 = 768;

struct Debug {
    d1: vec4<f32>,
    d2: vec4<f32>,
    d3: vec4<f32>,
    d4: vec4<f32>,
};
struct TimeUniform {
  time: f32,
}
struct ConstsUniform {
  phm_height: f32,
  phm_width: f32,
}
struct Offset {
  val: vec4<i32>,
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
  avoid_factor: f32,
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
@group(0) @binding(9)
var<storage, read_write> debug: Debug;

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


fn get_neighbour_coords(tex_coords: vec2<i32>, x: i32, y: i32) -> vec2<i32> {
  return vec2<i32>(
      max(0, min(I_SCREEN_WIDTH, tex_coords.x + x)),
      max(0, min(I_SCREEN_HEIGHT, tex_coords.y + y)),
  );
}

fn pheremone_diffusion(tex_coords: vec2<u32>) {
  var total_intensity: f32 = 0.0;
  var total_weight: f32 = 0.0;
  let txc_int = vec2<i32>(i32(tex_coords.x), i32(tex_coords.y));

  // Define the range of neighboring pixels to consider
  let range: i32 = 3; // Expensive

  let tex_color: vec4<f32> = textureLoad(phm, tex_coords);

  // Iterate over a range around the current pixel
  for (var x: i32 = -range; x <= range; x++) {
    for (var y: i32 = -range; y <= range; y++) {
      let neighbor_coords = get_neighbour_coords(txc_int, x, y);
      let neighbor_color: vec4<f32> = textureLoad(phm, neighbor_coords);

      let distance_weight: f32 = 1.0 / (1.0 + distance(vec2<f32>(f32(x), f32(y)), vec2<f32>(0.0, 0.0)));

      total_intensity += neighbor_color.r * pp.diffusion_factor;
      total_weight += pp.diffusion_factor;
    }
  }
    
  let avg_intensity: f32 = total_intensity / total_weight;

  textureStore(phm, tex_coords, vec4(max(tex_color.r, avg_intensity), 0.0, 0.0, 1.0));
}

fn pheremone_decay(tex_coords: vec2<u32>) {
  let current_clr: vec4<f32> = textureLoad(phm, tex_coords);
  let new_clr: f32 = max(0.0, current_clr.r*pp.decay_factor);

  textureStore(phm, tex_coords, vec4(new_clr, 0.0, 0.0, 1.0));
}

@compute 
@workgroup_size(32, 32, 1) 
fn update_pheremone_heatmap(@builtin(global_invocation_id) id: vec3<u32>) {
  let tcf: vec2<f32> = vec2<f32>(f32(id.x), f32(id.y)); 
  var tex_uv: vec2<f32> = scale_tex_aspect(tcf);
  
  pheremone_diffusion(id.xy);
  pheremone_decay(id.xy);
}
