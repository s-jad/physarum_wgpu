// CONSTANTS
const PI: f32 = 3.14159265;
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
struct ViewParams {
  shift_modifier: f32,
  x_shift: f32,
  y_shift: f32,
  zoom: f32,
  time_modifier: f32,
}

// GROUPS AND BINDINGS
@group(0) @binding(8)
var<storage, read_write> debug_arr: array<vec4<f32>>;
@group(0) @binding(9)
var<storage, read_write> debug: vec4<f32>;

@group(1) @binding(0)
var<uniform> tu: TimeUniform;

@group(2) @binding(0)
var<storage, read_write> vp: ViewParams;

@group(3) @binding(0)
var phm: texture_2d<f32>;
@group(3) @binding(1)
var phm_sampler: sampler;
@group(3) @binding(2)
var agents: texture_storage_2d<rgba32float, read_write>;

// ASPECT RATIO
fn scale_aspect(fc: vec2<f32>) -> vec2<f32> {
  // Scale from screen dimensions to 0.0 --> 1.0
  var uv: vec2<f32> = fc / vec2(SCREEN_WIDTH, SCREEN_HEIGHT);
  uv.y = 1.0 - uv.y; // Flip Y axis if necessary
  return uv;
}

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
  let t: f32 = tu.time * vp.time_modifier;
  var uv: vec2<f32> = scale_aspect(in.frag_coord.xy); // Scale to 0.0 -> 1.0 + fix aspect ratio
  var uv0 = uv;
  uv.x += vp.x_shift * vp.zoom;
  uv.y += vp.y_shift * vp.zoom;
  uv /= vp.zoom;
  var color = vec3(0.0);
// -----------------------------------------------------------------------------------------------

  let phm_sample = textureSample(phm, phm_sampler, uv);
  color.r += phm_sample.r;
  color.g += phm_sample.g*0.1;
  color.b += phm_sample.b;

// -----------------------------------------------------------------------------------------------
  return vec4<f32>(color, 1.0);
}
