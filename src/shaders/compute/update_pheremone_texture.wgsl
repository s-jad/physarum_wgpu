const SCREEN_WIDTH: f32 = 1376.0;
const SCREEN_HEIGHT: f32 = 768.0;
const I_SCREEN_WIDTH: i32 = 1376;
const I_SCREEN_HEIGHT: i32 = 768;
const DEPOSITION_RANGE: f32 = 0.005;
const MIN_POSITIVE_F32: f32 = 0x1.0p-126f;

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
struct SlimeParams {
  max_velocity: f32,
  min_velocity: f32,
  turn_factor: f32,
  avoid_factor: f32,
  sensor_dist: f32,
  sensor_offset: f32,
  sensor_radius: f32,
  brownian_offset: f32,
}
struct PheremoneParams {
  deposition_amount: f32,
  diffusion_factor: f32,
  decay_factor: f32,
}
struct FoodUniform {
  f1: vec2<u32>,
  f2: vec2<u32>,
  f3: vec2<u32>,
}

// COMPUTE GROUP
@group(0) @binding(1) var<storage, read_write> sp: SlimeParams;
@group(0) @binding(2) var<storage, read_write> pp: PheremoneParams;
@group(0) @binding(8) var<storage, read_write> debug_arr: array<vec4<f32>>;
@group(0) @binding(9) var<storage, read_write> debug: vec4<f32>;

@group(1) @binding(0) var<uniform> tu: TimeUniform;
@group(1) @binding(1) var<uniform> cu: ConstsUniform;
@group(1) @binding(2) var<uniform> food: FoodUniform;

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
  var total_intensity: vec3<f32> = vec3(0.0);
  var total_weight: f32 = 0.0;
  let txc_int = vec2<i32>(i32(tex_coords.x), i32(tex_coords.y));

  // Define the range of neighboring pixels to consider
  let range: i32 = 2; // Expensive

  let tex_color: vec4<f32> = textureLoad(phm, tex_coords);

  var fx = 0.0;
  var fy = 0.0;

  // Iterate over a range around the current pixel
  for (var x: i32 = -range; x <= range; x++) {
    for (var y: i32 = -range; y <= range; y++) {
      let neighbor_coords = get_neighbour_coords(txc_int, x, y);
      let neighbor_color: vec4<f32> = textureLoad(phm, neighbor_coords);

      let distance_weight: f32 = 1.0 / (1.0 + distance(vec2<f32>(fx, fy), vec2<f32>(0.0, 0.0)));

      total_intensity += neighbor_color.rgb * pp.diffusion_factor;
      total_weight += pp.diffusion_factor;

      fy += 1.0;
    }
    fx += 1.0;
  }
    
  // Calculate the average red intensity, considering the weights
  let avg_intensity: vec3<f32> = total_intensity / total_weight;
  let nr = max(tex_color.r, avg_intensity.r);
  let ng = max(tex_color.g, avg_intensity.g);
  let nb = max(tex_color.b, avg_intensity.b);

  textureStore(phm, tex_coords, vec4(nr, ng, nb, 1.0));
}

fn waste_product_buildup(p_intensity: f32) -> f32 {
  return step(0.75, p_intensity)*0.001 + step(p_intensity, 0.75)*-0.001;
}

@compute 
@workgroup_size(32, 32, 1) 
fn update_pheremone_heatmap(@builtin(global_invocation_id) id: vec3<u32>) {
  let tex_coords = id.xy;

  pheremone_diffusion(tex_coords);

  var clr: vec4<f32> = textureLoad(phm, tex_coords);
  clr.g += waste_product_buildup(clr.r);
  clr.r *= pp.decay_factor;
  clr.b = clr.b - smoothstep(0.01, 1.0, clr.r*0.04);

  clr = clamp(vec4(0.0), vec4(1.0), clr);

  textureStore(phm, tex_coords, clr);
}
