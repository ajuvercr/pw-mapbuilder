// Import the standard 2d mesh uniforms and set their bind groups
#import bevy_sprite::mesh2d_types

// I do not understand why I cannot just import this View thing
struct View {
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
    projection: mat4x4<f32>,
    inverse_projection: mat4x4<f32>,
    world_position: vec3<f32>,
    width: f32,
    height: f32,
};

@group(0) @binding(0)
var<uniform> view: View;

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

    let ratio: f32 = view.width / view.height;
    /* let ratio: f32 = 1699.0 / 1387.0; */
    let uv = vec2<f32>(vertex.position.x * ratio    , vertex.position.y  );
    out.position = vec2<f32>(uv * 10.);
    return out;
}


struct Color {
     color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> color: Color;


// The input of the fragment shader must correspond to the output of the vertex shader for all `location`s
struct FragmentInput {
    @location(0) position: vec2<f32>,
};

fn  plot(st: f32, pct: f32) -> f32{
  return  smoothstep( pct - 0.02 , pct, st) -
          smoothstep( pct, pct + 0.02, st);
}

/// Entry point for the fragment shader
@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var  W: f32 = 0.1;
    var frac: vec2<f32> = abs(fract(in.position + vec2(W * 0.5) - vec2(-W * 0.5)) - vec2(W)); 
    var is_border: f32 = min(frac.x, frac.y);
    var b = plot(is_border, 0.0);
    var bt = 1.0 - b;
    return b * color.color + bt * vec4(0.0, 0.0, 0.0, 1.0);
}

