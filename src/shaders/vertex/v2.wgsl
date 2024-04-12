struct VertexOutput {
  @builtin(position) frag_coord: vec4<f32>,
  @location(0) tex_coord: vec2<f32>,
}

@vertex
fn main(@location(0) pos: vec2<f32>) -> VertexOutput {
  let vo = VertexOutput(
      vec4<f32>(pos, 0.0, 1.0),
      vec2<f32>((pos * 2.0) - 1.0)
  );
  
  return vo;
}
