const NUM_AGENTS: u32 = 256u;
const INT_NUM_AGENTS: i32 = 256;
const NUM_PREDATORS: u32 = 4u;

const MAX_SCREEN: f32 = 1.0;
const MIN_SCREEN: f32 = 0.0;
const SCREEN_WIDTH: f32 = 1376.0;
const SCREEN_HEIGHT: f32 = 768.0;
const I_SCREEN_WIDTH: i32 = 1376;
const I_SCREEN_HEIGHT: i32 = 768;
const SCREEN_BUFFER: f32 = 0.001;
const MIN_POSITIVE_F32: f32 = 0x1.0p-126f;

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
struct TimeUniform {
  time: f32,
}
struct ConstsUniform {
  phm_height: f32,
  phm_width: f32,
}

@group(0) @binding(0) var<storage, read_write> agents: array<Slime>;
@group(0) @binding(1) var<storage, read_write> sp: SlimeParams;
@group(0) @binding(2) var<storage, read_write> pp: PheremoneParams;
@group(0) @binding(8) var<storage, read_write> debug_arr: array<vec4<f32>, NUM_AGENTS>;
@group(0) @binding(9) var<storage, read_write> debug: vec4<f32>;

@group(1) @binding(0) var<uniform> tu: TimeUniform;
@group(1) @binding(1) var<uniform> cu: ConstsUniform;

@group(2) @binding(0) var phm: texture_storage_2d<rgba32float, read_write>;

fn avoid_collisions(agent: Slime, agent_id: u32) -> vec2<f32> {
  var dv: vec2<f32> = vec2(0.0);
  let int_agent_id = i32(agent_id);

  for (var i: i32 = 0; i < INT_NUM_AGENTS; i++) {
    let dist: f32 = distance(agent.pos, agents[i].pos);
    let rnd: f32 = 2.0 * sin((tu.time * 0.1) + dot(agent.vel, agents[i].vel)) - 1.0;
    let in_range: f32 = step(dist, sp.avoid_factor);
    let not_self: f32 = step(0.0, f32(abs(int_agent_id - i)));
     
    dv += rnd*sp.turn_factor*in_range*not_self;
  }

  return dv;
}

fn respect_screen_edges(agent: Slime) -> vec2<f32> {
  var dp = agent.pos;

  let minx = step(dp.x, MIN_SCREEN);
  let maxx = step(MAX_SCREEN, dp.x);

  let miny = step(dp.y, MIN_SCREEN);
  let maxy = step(MAX_SCREEN, dp.y);

  let dpx = minx*(MAX_SCREEN - SCREEN_BUFFER)
    + maxx*(MIN_SCREEN + SCREEN_BUFFER);

  let dpy = miny*(MAX_SCREEN - SCREEN_BUFFER)
    + maxy*(MIN_SCREEN + SCREEN_BUFFER);

  let no_change = step((minx + maxx + miny + maxy), MIN_POSITIVE_F32);

  dp = vec2(dpx, dpy) + dp*no_change;
  
  return dp;
}

fn clamp_and_scale_velocity(agent: Slime) -> vec2<f32> {
    let magnitude: f32 = length(agent.vel);
    let clamped_magnitude: f32 = clamp(magnitude, sp.min_velocity, sp.max_velocity);
    let normalized_velocity: vec2<f32> = normalize(agent.vel);
    let new_velocity: vec2<f32> = normalized_velocity*clamped_magnitude;
    
    return new_velocity;
}

// SLIME SENSORS
fn sensor_position(agent: Slime, heading: f32, offset: f32) -> vec2<f32> {
  // Calculate the positions of the sensors relative to the slime's heading
  let angle: f32 = heading + offset;

  return vec2<f32>(
      agent.pos.x + sp.sensor_dist * cos(angle),
      agent.pos.y + sp.sensor_dist * sin(angle)
  );
}

fn calculate_sensor_positions(agent: Slime, id: u32) {
  // Calculate the heading angle from the agent's velocity
  let norm: vec2<f32> = normalize(agent.vel);
  let heading: f32 = atan2(norm.y, norm.x);

  // Calculate the positions of s1, s2, and s3
  agents[id].s1_pos = sensor_position(agent, heading, -sp.sensor_offset);
  agents[id].s2_pos = sensor_position(agent, heading, 0.0);
  agents[id].s3_pos = sensor_position(agent, heading, sp.sensor_offset);
}

fn clamp_coord(tex_coord: vec2<i32>, i: i32, j: i32) -> vec2<i32> {
  return vec2<i32>(
    max(0, min(tex_coord.x + i, I_SCREEN_WIDTH)),
    max(0, min(tex_coord.y + j, I_SCREEN_HEIGHT)),
  );
}

fn map_to_screen_coords(agent_pos: vec2<f32>) -> vec2<i32> {
    // Convert normalized coordinates to screen coordinates
    let screen_pos: vec2<f32> = agent_pos * vec2<f32>(SCREEN_WIDTH, SCREEN_HEIGHT);
    
    return vec2(i32(screen_pos.x), i32(screen_pos.y));
}

fn pheremone_deposition(agent_pos: vec2<f32>, moved_forward: f32) {
  let agent_sc = map_to_screen_coords(agent_pos);
  var texel = textureLoad(phm, agent_sc);
  texel.r += pp.deposition_amount;
  textureStore(phm, agent_sc, texel);
}

struct QuiescenceResult {
  direction: vec2<f32>,
  moved_forward: f32,
}

fn quiescence(agent: Slime) -> QuiescenceResult {
  var s1_total: f32 = 0.0;
  var s2_total: f32 = 0.0;
  var s3_total: f32 = 0.0;

  let s_radius = i32(sp.sensor_radius);

  // Calculate the positions to sample
  let s1_tex_coord = vec2<i32>(agent.s1_pos * vec2(SCREEN_WIDTH, SCREEN_HEIGHT));
  let s2_tex_coord = vec2<i32>(agent.s2_pos * vec2(SCREEN_WIDTH, SCREEN_HEIGHT));
  let s3_tex_coord = vec2<i32>(agent.s3_pos * vec2(SCREEN_WIDTH, SCREEN_HEIGHT));

  for (var i: i32 = -s_radius; i <= s_radius; i++) {
    for (var j: i32 = -s_radius; j <= s_radius; j++) {
      let s1_coord: vec2<i32> = clamp_coord(s1_tex_coord, i, j);
      let s2_coord: vec2<i32> = clamp_coord(s2_tex_coord, i, j);
      let s3_coord: vec2<i32> = clamp_coord(s3_tex_coord, i, j);

      // Sample the texture
      let s1_sample = textureLoad(phm, s1_coord);
      let s2_sample = textureLoad(phm, s2_coord);
      let s3_sample = textureLoad(phm, s3_coord);

      // Add to totals
      s1_total += s1_sample.r;
      s2_total += s2_sample.r;
      s3_total += s3_sample.r;
    }
  }

  let max_total = max(max(s1_total, s2_total), s3_total);
  debug = vec4(max_total, s1_total, s2_total, s3_total);
  
  // Move in direction of highest pheremone concentration
  // If max_total - sensor_total is less that MIN_POSITIVE_F32 
  // That means its 0.0 and that sensor found the highest concentration
  // move_* variables used to zero out direction_shift changes
  let move_left = step(MIN_POSITIVE_F32, max_total - s1_total);
  let move_forward = step(MIN_POSITIVE_F32, max_total - s2_total);
  let move_right = step(MIN_POSITIVE_F32, max_total - s3_total);

  let direction = move_left*(normalize(agent.s1_pos - agent.pos))
    + move_forward*(normalize(agent.s2_pos - agent.pos))
    + move_right*(normalize(agent.s3_pos - agent.pos));

  return QuiescenceResult(
    direction*sp.turn_factor,
    move_forward,
  );
}

@compute 
@workgroup_size(16, 16, 1) 
fn update_slime_positions(@builtin(global_invocation_id) id: vec3<u32>) {
  var agent = agents[id.x];
  calculate_sensor_positions(agent, id.x);
  
  // Sense pheremones
  let qr = quiescence(agent);
  agent.vel += qr.direction;
  //agents[id.x].vel += avoid_collisions(agents[id.x], id.x);

  agent.vel = clamp_and_scale_velocity(agent);

  // Move
  agent.pos += agent.vel;
  agent.pos = respect_screen_edges(agent);

  // Deposit Pheremones
  pheremone_deposition(agent.pos, qr.moved_forward);

  agents[id.x] = agent;
}
