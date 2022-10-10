
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
    out.position = vec2<f32>(uv / config.zoom);

    out.position.x = out.position.x - 1.0;
    return out;
}

// The input of the fragment shader must correspond to the output of the vertex shader for all `location`s
struct FragmentInput {
    @location(0) position: vec2<f32>,
};

fn  plot(st: f32, pct: f32) -> f32{
    return step(abs(st), 0.01);
}


fn dashed(f: f32) -> f32 {
    var len = 2. * (1. + 0.5);

    var per = fract(f / len);
    return step(per, 1. / len);
}

fn dashed_twice(st: vec2<f32>) -> f32 {
    var t_height = 2.0 * sin(radians(60.));

    var l1 = dashed(st.x) * fract(st.y  / t_height - 0.005) ;
    var l2 = dashed(st.x + 1.5) * fract(st.y / t_height + 0.5 - 0.005);

    return max(l1, l2);
}

fn rotate(input: vec2<f32>, a: f32) -> vec2<f32> {
    var c = cos(a);
    var s = sin(a);
    return vec2(
        c * input.x + s * input.y,
        -s * input.x + c * input.y
    );
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var dp1 = rotate(vec2(in.position.x, in.position.y), radians(240.0));
    var dp2 = rotate(vec2(in.position.x, in.position.y), radians(120.0));

    var h1 = dashed_twice(in.position);
    var d1 = dashed_twice(dp1);
    var d2 = dashed_twice(dp2);

    var l = max(max(h1, d1), d2 );

    var b = plot(1.0 - l, 0.0);

    var bt = 1.0 - b;
    return b * vec4(config.cx,config.cy,config.cz, 1.0) + bt * vec4(0.0, 0.0, 0.0, 1.0);
}


