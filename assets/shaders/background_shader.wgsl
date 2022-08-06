// Import the standard 2d mesh uniforms and set their bind groups
#import bevy_sprite::mesh2d_types
#import bevy_sprite::mesh2d_view_bindings
@group(1) @binding(0)
var<uniform> mesh: Mesh2d;
// NOTE: Bindings must come before functions that use them!
#import bevy_sprite::mesh2d_functions
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
    out.position = vec2<f32>(vertex.position.xy) * 10.0;
    return out;
}

struct Color {
     color: vec4<f32>,
};
@group(2) @binding(0)
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
