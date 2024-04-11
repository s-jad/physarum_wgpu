const SCREEN_WIDTH: f32 = 1376.0;
const SCREEN_HEIGHT: f32 = 768.0;
const I_SCREEN_WIDTH: i32 = 1376;
const I_SCREEN_HEIGHT: i32 = 768;
const MIN_POSITIVE_F32: f32 = 0x1.0p-126f;

struct TimeUniform {
  time: f32,
}

// FOOD GROUP
@group(0) @binding(0) var phm: texture_storage_2d<rgba32float, read_write>;
@group(0) @binding(1) var<storage> food: array<vec2<u32>>;
@group(0) @binding(2) var<uniform> tu: TimeUniform;

fn pcg2d(p: vec2<u32>) -> vec2<f32> {
    var v = p * 1664525u + 1013904223u;
    v.x += v.y * 1664525u; v.y += v.x * 1664525u;
    v ^= v >> vec2<u32>(16u);
    v.x += v.y * 1664525u; v.y += v.x * 1664525u;
    v ^= v >> vec2<u32>(16u);

    // Normalize the output to the range [0, 1]
    return vec2<f32>(f32(v.x), f32(v.y)) / 0xFFFFFFFF;
}

@compute 
@workgroup_size(1, 1, 1) 
fn update_food(@builtin(global_invocation_id) id: vec3<u32>) {
  let tex_coords = id.xy;
  let ts = sin(tu.time);
  let food_coords = pcg2d(vec2(u32(ts*23.0), u32(ts*17.0)));
  let fc = vec2(u32(food_coords.x * SCREEN_WIDTH), u32(food_coords.y * SCREEN_HEIGHT));

  var tex = textureLoad(phm, fc);
  tex.b = 1.0;

  textureStore(phm, fc, tex);
}
