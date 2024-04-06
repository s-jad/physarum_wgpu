const NUM_AGENTS: u32 = 256u;
const NUM_PREDATORS: u32 = 4u;

const MAX_SCREEN_X: f32 = 1.0;
const MIN_SCREEN_X: f32 = 0.0;
const MAX_SCREEN_Y: f32 = 1.0;
const MIN_SCREEN_Y: f32 = 0.0;
const SCREEN_BUFFER: f32 = 0.1;

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
struct TimeUniform {
  time: f32,
}
struct ConstsUniform {
  phm_height: f32,
  phm_width: f32,
}

@group(0) @binding(0) var<storage, read_write> agents: array<Slime>;
@group(0) @binding(1) var<storage, read_write> sp: SlimeParams;
@group(0) @binding(8) var<storage, read_write> debug_arr: array<vec4<f32>, NUM_AGENTS>;
@group(0) @binding(9) var<storage, read_write> debug: vec4<f32>;

@group(1) @binding(0) var<uniform> tu: TimeUniform;
@group(1) @binding(1) var<uniform> cu: ConstsUniform;

@group(2) @binding(0) var phm: texture_storage_2d<rgba32float, read_write>;

// TODO! tune turning factor so slime cannot exceed limits
// fn respect_screen_edges(agent: Slime) -> vec2<f32> {
//   var dv = vec2(0.0);
// 
//   if (agent.pos.x < MIN_SCREEN_X + SCREEN_BUFFER) {
//     dv.x += sp.turn_factor;
//   }
//   if (agent.pos.x > MAX_SCREEN_X - SCREEN_BUFFER) {
//     dv.x -= sp.turn_factor;
//   }
//   if (agent.pos.y < MIN_SCREEN_Y + SCREEN_BUFFER) {
//     dv.y += sp.turn_factor;
//   }
//   if (agent.pos.y > MAX_SCREEN_Y - SCREEN_BUFFER) {
//     dv.y -= sp.turn_factor;
//   }
// 
//   return dv;
// }

fn respect_screen_edges(agent: Slime) -> vec2<f32> {
  var dv = agent.vel;

  if (
    agent.pos.x < MIN_SCREEN_X + SCREEN_BUFFER
    || agent.pos.x > MAX_SCREEN_X - SCREEN_BUFFER
    || agent.pos.y < MIN_SCREEN_Y + SCREEN_BUFFER
    || agent.pos.y > MAX_SCREEN_Y - SCREEN_BUFFER
  ) {
    dv = -(agent.vel);
  }

  return dv;
}

fn respect_speed_limit(agent: Slime) -> vec2<f32> {
  return clamp(agent.vel, vec2(sp.min_velocity), vec2(sp.max_velocity));
}

// SLIME SENSORS
fn sensor_position(slime: Slime, heading: f32, offset: f32) -> vec2<f32> {
  // Calculate the positions of the sensors relative to the slime's heading
  let angle: f32 = heading + offset;

  return vec2<f32>(
      slime.pos.x + sp.sensor_dist * cos(angle),
      slime.pos.y + sp.sensor_dist * sin(angle)
  );
}

fn calculate_sensor_positions(slime: Slime, id: u32) {
  // Calculate the heading angle from the slime's velocity
  let norm: vec2<f32> = normalize(slime.vel);
  let heading: f32 = atan2(norm.y, norm.x);

  // Calculate the positions of s1, s2, and s3
  agents[id].s1_pos = sensor_position(slime, heading, -sp.sensor_offset);
  agents[id].s2_pos = sensor_position(slime, heading, 0.0);
  agents[id].s3_pos = sensor_position(slime, heading, sp.sensor_offset);
}

@compute 
@workgroup_size(16, 16, 1) 
fn update_slime_positions(@builtin(global_invocation_id) id: vec3<u32>) {
  //agents[id.x].vel = respect_screen_edges(agents[id.x]);
  //agents[id.x].vel = respect_speed_limit(agents[id.x]);
 
  //calculate_sensor_positions(agents[id.x], id.x);
  //agents[id.x].pos += agents[id.x].vel;
  
  //textureStore(phm, id.xy, vec4<f32>(1.0, 0.0, 0.0, 1.0));
}
