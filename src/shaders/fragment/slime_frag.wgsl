// CONSTANTS
const PI: f32 = 3.14159265;
const NUM_AGENTS: u32 = 256u;
const NUM_PREDATORS: u32 = 4u;
const SCREEN_WIDTH: f32 = 1376.0;
const SCREEN_HEIGHT: f32 = 768.0;

// STRUCTS
struct VertexOutput {
    @builtin(position) frag_coord: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};
struct TimeUniform {
    time: f32,
};
struct ConstUniforms {
  phm_height: f32,
  phm_width: f32,
}
struct ViewParams {
  shift_modifier: f32,
  x_shift: f32,
  y_shift: f32,
  zoom: f32,
  time_modifier: f32,
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
  deposition_range: f32,
  diffusion_factor: f32,
  decay_factor: f32,
}
struct TextureExtent {
  height: f32,
  width: f32,
}

// GROUPS AND BINDINGS
@group(0) @binding(0)
var<storage, read_write> agents: array<Slime>;
@group(0) @binding(1)
var<storage, read_write> sp: SlimeParams;
@group(0) @binding(2)
var<storage, read_write> pp: PheremoneParams;
@group(0) @binding(9)
var<storage, read_write> debug: vec4<f32>;

@group(1) @binding(0)
var<uniform> tu: TimeUniform;
@group(1) @binding(1)
  var<uniform> cu: ConstUniforms;

@group(2) @binding(0)
var<storage, read_write> vp: ViewParams;

@group(3) @binding(0)
var phm: texture_2d<f32>;
@group(3) @binding(1)
var phm_sampler: sampler;


// ASPECT RATIO
const screen: vec2<f32> = vec2(SCREEN_WIDTH, SCREEN_HEIGHT);
fn scale_aspect(fc: vec2<f32>) -> vec2<f32> {
  // Scale from screen dimensions to 0.0 --> 1.0
  var uv: vec2<f32> = fc / screen;
  uv.y = 1.0 - uv.y; // Flip Y axis if necessary
  return uv;
}

// COLORS
fn palette(t: f32) -> vec3<f32> {
  let a: vec3<f32> = vec3<f32>(0.120, 0.618, 0.624); 
  let b: vec3<f32> = vec3<f32>(0.878, 0.214, 0.229);
  let c: vec3<f32> = vec3<f32>(0.654, 0.772, 0.426);
  let d: vec3<f32> = vec3<f32>(0.937, 0.190, 0.152);

  return a * b * cos(PI * 2.0 * (c * t + d));
}

// HASHING
fn shash21(pos: vec2<f32>) -> f32 {
  return fract(sin(dot(pos, vec2(12.34777, 67.8913375))) * 4277123.455) * 2.0 - 1.0;
}

fn shash22(pos: vec2<f32>) -> vec2<f32> {
  return vec2<f32>(
    fract(sin(dot(pos, vec2(12.34777, 67.8913375))) * 4277123.455),
    fract(cos(dot(pos, vec2(71.43119, 33.7654637))) * 9854756.117)
  ) * 2.0 - 1.0;
}

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
  let t: f32 = tu.time * vp.time_modifier;
  debug = in.frag_coord;
  var uv: vec2<f32> = scale_aspect(in.frag_coord.xy); // Scale to 0.0 -> 1.0 + fix aspect ratio
  var uv0 = uv;
  uv.x += vp.x_shift * vp.zoom;
  uv.y += vp.y_shift * vp.zoom;
  uv /= vp.zoom;
  var color = vec3(0.0);
// -----------------------------------------------------------------------------------------------

  for (var i: u32 = 0u; i < NUM_AGENTS; i++) {
    // Show Agents
    let sd = distance(uv, agents[i].pos);
    color += 1.0 - smoothstep(0.0, 0.003, sd);
    
    // Show Sensors
    //let sd1 = distance(uv, agents[i].s1_pos);
    //let sd2 = distance(uv, agents[i].s2_pos);
    //let sd3 = distance(uv, agents[i].s3_pos);
    //color += vec3<f32>(0.3, 0.0, 0.0) * (1.0 - smoothstep(0.0, sp.sensor_radius*2.0, sd1));
    //color += vec3<f32>(0.0, 0.3, 0.0) * (1.0 - smoothstep(0.0, sp.sensor_radius*2.0, sd2));
    //color += vec3<f32>(0.0, 0.0, 0.3) * (1.0 - smoothstep(0.0, sp.sensor_radius*2.0, sd3));
  }

  let tex_sample = textureSample(phm, phm_sampler, uv);
  color += tex_sample.xyz;
  
// -----------------------------------------------------------------------------------------------
  return vec4<f32>(color, 1.0);
}
