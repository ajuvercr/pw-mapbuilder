
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

    let uv = vec2<f32>(vertex.position.x * config.width * 0.5 - config.x, vertex.position.y * config.height * 0.5 - config.y);
    out.position = vec2<f32>(uv / config.zoom) + vec2(0.5);
    return out;
}

// The input of the fragment shader must correspond to the output of the vertex shader for all `location`s
struct FragmentInput {
    @location(0) position: vec2<f32>,
};

fn  plot(st: f32, pct: f32) -> f32{
    return step(abs(st), 0.02);
}

/// Entry point for the fragment shader
@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var  W: f32 = 0.1;

    var hor = fract(in.position.y - 0.005);
    var vert = fract(in.position.x - 0.005);
    var l = max(hor, vert);

    var b = plot(1.0 - l, 0.0);

    var bt = 1.0 -  b;
    return b * vec4(config.cx,config.cy,config.cz, 1.0) + bt * vec4(0.0, 0.0, 0.0, 1.0);
}

