#import bevy_pbr::mesh_types
#import bevy_pbr::mesh_view_bindings
#import "shaders/common.wgsl"

@group(1) @binding(0)
var<uniform> mesh: Mesh;

// NOTE: Bindings must come before functions that use them!
#import bevy_pbr::mesh_functions

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,

    @location(3) i_pos_scale: vec4<f32>,
    @location(4) i_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) pos: vec3<f32>,
    @location(2) nor: vec3<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let position = vertex.position * vertex.i_pos_scale.w + vertex.i_pos_scale.xyz;
    var out: VertexOutput;
    out.clip_position = mesh_position_local_to_clip(mesh.model, vec4<f32>(position, 1.0));
    out.color = vertex.i_color;
    out.pos = mesh_position_local_to_world(mesh.model , vec4<f32>(position, 1.0)).xyz;
    out.nor = mesh_normal_local_to_world(vertex.normal);
    return out;
}

// @fragment
// fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
//     return in.color;
// }
@fragment
fn fragment(
in : VertexOutput 
) -> @location(0) vec4<f32> {
    let sun = normalize(vec3(sunx,suny,sunz));
    let pos = in.pos;
    let nor = in.nor;
    let nds = dot(nor, sun);
    let rd = -normalize(view.world_position.xyz - pos);
    let rfl = reflect(rd, nor);
    let fre = pow(dot(rd, rfl) * 0.5 + 0.5, 5.0);
    let softness = 12.0;
    let scl = vec3(1.0, 0.8, 0.6);
    let skc = vec3(0.6, 0.7, 1.0);
    let spc = pow((dot(rfl, sun) * 0.5 + 0.5) * fre, 9.0);
    let bcl = in.color.rgb;
    let sk = sky(vec3(rfl.x,abs(rfl.y),rfl.z));
    let bou = mix(
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        smoothstep(1.0, -1.0, pos.x)
    );
    let col = bcl * (
    // sun
     scl * 2.0 * (max(0.0, nds) + 
    //sky
    spc * 10.0) + skc *  (0.7 + fre) * (dot(nor, vec3(0.0, 1.0, 0.0)) * 0.25 + 0.75) + 
    //bounc
    bou * 2.0 * max((-nds * 0.5 + 0.5), dot(nor, sun * vec3(1.0, -1.0, 1.0))) * max(0.0, 1. - pos.y) * (1. + fre) + 
    //spec
    sk *  (1.0 + fre) * 0.5
    );
    
    return  vec4(col, 1.0)    ;
}
