
struct Config {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    zoom: f32,

    cx: f32,
    cy: f32,
    cz: f32,
};

@group(0) @binding(0)
var<uniform> config: Config;

// NOTE: Bindings must come before functions that use them!
// The structure of the vertex buffer is as specified in `specialize()`
struct Vertex {
    @location(0) position: vec3<f32>,
};
struct VertexOutput {
    // The vertex shader must set the on-screen position of the vertex
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec2<f32>,
};

/// Entry point for the vertex shader
@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(vertex.position, 1.0);

    let ratio: f32 = config.width / config.height;
    let uv = vec2<f32>(vertex.position.x * config.width * 0.5 - config.x, vertex.position.y * config.height * 0.5 - config.y);
    out.position = vec2<f32>(uv / config.zoom);
    return out;
}

// The input of the fragment shader must correspond to the output of the vertex shader for all `location`s
struct FragmentInput {
    @location(0) position: vec2<f32>,
};

fn  plot(st: f32, pct: f32) -> f32{
  return  smoothstep( pct - 0.04 , pct, st) -
          smoothstep( pct, pct + 0.04, st);
}

/// Entry point for the fragment shader
@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var  W: f32 = 0.1;

    var frac: vec2<f32> = abs(fract(in.position + 0.5 + vec2(W)) - vec2(W) ); 
    var is_border: f32 = min(frac.x, frac.y);

    var b = plot(is_border, 0.0);
    var bt = 1.0 - b;
    return b * vec4(config.cx,config.cy,config.cz, 1.0) + bt * vec4(0.0, 0.0, 0.0, 1.0);
}

