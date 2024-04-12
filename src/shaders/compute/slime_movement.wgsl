const MAX_SCREEN: f32 = 1.0;
const MIN_SCREEN: f32 = 0.0;
const SCREEN_WIDTH: f32 = 1376.0;
const SCREEN_HEIGHT: f32 = 768.0;
const I_SCREEN_WIDTH: i32 = 1376;
const I_SCREEN_HEIGHT: i32 = 768;
const SCREEN_BUFFER: f32 = 0.001;
const MIN_POSITIVE_F32: f32 = 0x1.0p-126f;

@group(0) @binding(1) var<storage, read_write> sp: SlimeParams;
@group(0) @binding(2) var<storage, read_write> pp: PheremoneParams;
@group(0) @binding(8) var<storage, read_write> debug_arr: array<vec4<f32>, 1024>;
@group(0) @binding(9) var<storage, read_write> debug: vec4<f32>;

@group(1) @binding(0) var<uniform> tu: TimeUniform;
@group(1) @binding(1) var<uniform> cu: ConstsUniform;

@group(2) @binding(0) var phm: texture_storage_2d<rgba32float, read_write>;
@group(2) @binding(1) var agents: texture_storage_2d<rgba32float, read_write>;

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
struct TimeUniform {
  time: f32,
}
struct ConstsUniform {
  phm_height: f32,
  phm_width: f32,
}

fn respect_screen_edges(agent_pos: vec2<f32>) -> vec2<f32> {
  var dp = agent_pos;

  let minx = step(dp.x, MIN_SCREEN);
  let maxx = step(MAX_SCREEN, dp.x);

  let miny = step(dp.y, MIN_SCREEN);
  let maxy = step(MAX_SCREEN, dp.y);
  
  let dpx = minx*(MAX_SCREEN - 0.001)
    + maxx*(MIN_SCREEN + 0.001)
    + dp.x*step(minx + maxx, MIN_POSITIVE_F32);

  let dpy = miny*(MAX_SCREEN - 0.001)
    + maxy*(MIN_SCREEN + 0.001)
    + dp.y*step(miny + maxy, MIN_POSITIVE_F32);

  return vec2<f32>(dpx, dpy);
}

fn clamp_and_scale_velocity(agent_vel: vec2<f32>) -> vec2<f32> {
    let magnitude: f32 = length(agent_vel);
    let clamped_magnitude: f32 = clamp(magnitude, sp.min_velocity, sp.max_velocity);
    let normalized_velocity: vec2<f32> = normalize(agent_vel);
    let new_velocity: vec2<f32> = normalized_velocity*clamped_magnitude;
    
    return new_velocity;
}

// SLIME SENSORS
fn sensor_position(agent: vec4<f32>, heading: f32, offset: f32) -> vec2<f32> {
  // Calculate the positions of the sensors relative to the agent's heading
  let angle: f32 = heading + offset;

  return vec2<f32>(
      agent.x + sp.sensor_dist * cos(angle),
      agent.y + sp.sensor_dist * sin(angle)
  );
}

struct Sensors {
  s1_pos: vec2<f32>,
  s2_pos: vec2<f32>,
  s3_pos: vec2<f32>,
}

fn calculate_sensor_positions(agent: vec4<f32>) -> Sensors {
  // Calculate the heading angle from the agent's velocity
  let norm: vec2<f32> = normalize(agent.zw);
  let heading: f32 = atan2(norm.y, norm.x);

  // Calculate the positions of s1, s2, and s3
  let s1_pos = sensor_position(agent, heading, -sp.sensor_offset);
  let s2_pos = sensor_position(agent, heading, 0.0);
  let s3_pos = sensor_position(agent, heading, sp.sensor_offset);

  return Sensors(
    s1_pos,
    s2_pos,
    s3_pos,
  );
}

fn clamp_coord(tex_coord: vec2<i32>, i: i32, j: i32) -> vec2<i32> {
  return vec2<i32>(
    max(0, min(tex_coord.x + i, I_SCREEN_WIDTH)),
    max(0, min(tex_coord.y + j, I_SCREEN_HEIGHT)),
  );
}

fn map_to_screen_coords(agent_pos: vec2<f32>) -> vec2<i32> {
    let screen_pos: vec2<f32> = agent_pos * vec2<f32>(SCREEN_WIDTH, SCREEN_HEIGHT);
    
    return vec2(i32(screen_pos.x), i32(screen_pos.y));
}

fn pheremone_deposition(agent_pos: vec2<f32>, moved_forward: f32, food_eaten: f32) {
  let agent_sc = map_to_screen_coords(agent_pos);
  var texel = textureLoad(phm, agent_sc);
  texel.r += pp.deposition_amount + food_eaten*0.01;
  textureStore(phm, agent_sc, texel);
}

fn pcg2d(p: vec2<u32>) -> vec2<f32> {
    var v = p * 1664525u + 1013904223u;
    v.x += v.y * 1664525u; v.y += v.x * 1664525u;
    v ^= v >> vec2<u32>(16u);
    v.x += v.y * 1664525u; v.y += v.x * 1664525u;
    return vec2(f32(v.x), f32(v.y));
}

fn rand22(n: vec2<f32>) -> vec2<f32> {
    let hashed = pcg2d(vec2(u32(n.x), u32(n.y)));
    return vec2<f32>(hashed) / vec2<f32>(0xffffffff);
}

struct QuiescenceResult {
  direction: vec2<f32>,
  moved_forward: f32,
  food: f32,
}

fn quiescence(agent: vec4<f32>, id: vec2<f32>) -> QuiescenceResult {
  var s1_total: vec4<f32> = vec4(0.0);
  var s2_total: vec4<f32> = vec4(0.0);
  var s3_total: vec4<f32> = vec4(0.0);

  let s_radius = 2;
  
  let sensors = calculate_sensor_positions(agent);
  
  // Calculate the positions to sample
  let s1_tex_coord = vec2<i32>(sensors.s1_pos * vec2(SCREEN_WIDTH, SCREEN_HEIGHT));
  let s2_tex_coord = vec2<i32>(sensors.s2_pos * vec2(SCREEN_WIDTH, SCREEN_HEIGHT));
  let s3_tex_coord = vec2<i32>(sensors.s3_pos * vec2(SCREEN_WIDTH, SCREEN_HEIGHT));

  for (var i: i32 = -s_radius; i <= s_radius; i++) {
    for (var j: i32 = -s_radius; j <= s_radius; j++) {
      // Get neighbouring coords within sensor radius
      let s1_coord: vec2<i32> = clamp_coord(s1_tex_coord, i, j);
      let s2_coord: vec2<i32> = clamp_coord(s2_tex_coord, i, j);
      let s3_coord: vec2<i32> = clamp_coord(s3_tex_coord, i, j);

      // Sample the texture
      let s1_sample = textureLoad(phm, s1_coord);
      let s2_sample = textureLoad(phm, s2_coord);
      let s3_sample = textureLoad(phm, s3_coord);

      // Add to totals
      s1_total += s1_sample;
      s2_total += s2_sample;
      s3_total += s3_sample;
    }
  }
  
  // Find:
  // -- max pheremone value -> RED 
  // -- max waste value -> GREEN
  // -- max food value -> BLUE
  let max_pheremones = max(max(s1_total.r, s2_total.r), s3_total.r);
  let max_waste = max(max(s1_total.g, s2_total.g), s3_total.g);
  let max_food = max(max(s1_total.b, s2_total.b), s3_total.b);

  let s1_max: f32 = step(max_pheremones - s1_total.r, MIN_POSITIVE_F32);
  let s2_max: f32 = step(max_pheremones - s2_total.r, MIN_POSITIVE_F32);
  let s3_max: f32 = step(max_pheremones - s3_total.r, MIN_POSITIVE_F32);

  let w1_max: f32 = step(max_waste - s1_total.g, MIN_POSITIVE_F32);
  let w2_max: f32 = step(max_waste - s2_total.g, MIN_POSITIVE_F32);
  let w3_max: f32 = step(max_waste - s3_total.g, MIN_POSITIVE_F32);

  let f1_max: f32 = step(max_food - s1_total.b, MIN_POSITIVE_F32);
  let f2_max: f32 = step(max_food - s2_total.b, MIN_POSITIVE_F32);
  let f3_max: f32 = step(max_food - s3_total.b, MIN_POSITIVE_F32);
  let food: f32 = step(MIN_POSITIVE_F32, s1_total.b + s2_total.b + s3_total.b);
  
  let s1_dir: vec2<f32> = normalize(sensors.s1_pos - agent.xy);
  let s2_dir: vec2<f32> = normalize(sensors.s2_pos - agent.xy);
  let s3_dir: vec2<f32> = normalize(sensors.s3_pos - agent.xy);
  
  var direction: vec2<f32> = (
    s1_max*s1_dir
    + s2_max*s2_dir
    + s3_max*s3_dir
  )*sp.turn_factor;
  
  direction += (
    w1_max*s1_dir
    + w2_max*s2_dir
    + w3_max*s3_dir
  )*sp.avoid_factor;

  direction += food*(
    f1_max*s1_dir
    + f2_max*s2_dir
    + f3_max*s3_dir
  )*sp.turn_factor*10.0;
  
  direction += rand22(direction)*sp.brownian_offset;

  return QuiescenceResult(
    direction,
    s2_max,
    food,
  );
}

@compute 
@workgroup_size(32, 32, 1) 
fn update_slime_positions(@builtin(global_invocation_id) id: vec3<u32>) {
  let agent = textureLoad(agents, id.xy);
  var agent_pos = agent.xy;
  var agent_vel = agent.zw;

  // Sense pheremones
  let qr = quiescence(agent, vec2(f32(id.x), f32(id.y)));
  agent_vel += qr.direction;

  // Move
  agent_vel = clamp_and_scale_velocity(agent_vel);
  agent_pos += agent_vel;
  agent_pos = respect_screen_edges(agent_pos);

  // Deposit Pheremones
  pheremone_deposition(agent_pos, qr.moved_forward, qr.food);

  textureStore(agents, id.xy, vec4<f32>(agent_pos, agent_vel));
}
