#import bevy_pbr::mesh_view_bindings
#import "shaders/common.wgsl"

struct CustomMaterial {
    color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: CustomMaterial;
@group(1) @binding(1)
var box_texture: texture_2d<f32>;
@group(1) @binding(2)
var box_sampler: sampler;


@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let sun = normalize(vec3(sunx,suny,sunz));
    let pos = world_position.xyz;
    let nor = normalize(world_normal.xyz);
    let nds = dot(nor, sun);
    let rd = -normalize(view.world_position.xyz - pos);
    let rfl = reflect(rd, nor);
    let fre = pow(dot(rd, rfl) * 0.5 + 0.5, 5.0);
    let softness = 12.0;
    let scl = vec3(1.0, 0.8, 0.6);
    let skc = vec3(0.6, 0.7, 1.0);
    var sha = 1.0;
    let sca = (pos.xz / 30.0 + 0.5) * 30.0;
    let c: vec2<i32> = vec2(i32(sca.x), i32(sca.y));
    let sa = floor(pos.xz);
    let h1 = textureLoad(box_texture, c + vec2(1, 1), 0);
    let h2 = textureLoad(box_texture, c + vec2(0, 1), 0);
    let h3 = textureLoad(box_texture, c + vec2(1, 0), 0);
    let re = pos.xz - sa;
    let sk = sky(vec3(rfl.x,abs(rfl.y),rfl.z));
    //   h2 h1
    // h5 c h3
    //   h6
    var occ = 1.0;
    if pos.y != 0.0 {
        occ = smoothstep(0.0, 0.05, pos.y);
    }
    if h1.r >= 0.6 {
        let box_height = h1.r * 0.5;
        let box_pos = vec3(floor(pos.x) + 1.5, box_height, floor(pos.z) + 1.5);
        sha = min(sha, boxSoftShadow(pos - box_pos, sun, vec3(0.5, box_height, 0.5), softness));
        if pos.y == 0.0 {
            occ *= smoothstep(1.0, 0.9, min(re.x, re.y));
        }
    }
    if h2.r >= 0.6 {
        let box_height = h2.r * 0.5;
        let box_pos = vec3(floor(pos.x) + 0.5, box_height, floor(pos.z) + 1.5);
        sha = min(sha, boxSoftShadow(pos - box_pos, sun, vec3(0.5, box_height, 0.5), softness));
        if pos.y == 0.0 {
            occ *= smoothstep(1.0, 0.9, re.y);
        }
    }

    if h3.r >= 0.6 {
        let box_height = h3.r * 0.5;
        let box_pos = vec3(floor(pos.x) + 1.5, box_height, floor(pos.z) + 0.5);
        sha = min(sha, boxSoftShadow(pos - box_pos, sun, vec3(0.5, box_height, 0.5), softness));
        if pos.y == 0.0 {
            occ *= smoothstep(1.0, 0.9, re.x);
        }
    }
    occ = occ * 0.4 + 0.6;
    let spc = pow((dot(rfl, sun) * 0.5 + 0.5) * fre, 9.0);
    let bcl = material.color.rgb;
    let bou = mix(
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        smoothstep(1.0, -1.0, pos.x)
    );
    let col = bcl * (
    // sun
    sha * scl * 2.0 * (max(0.0, nds) + 
    //sky
    spc * 10.0) + skc * occ * (0.7 + fre) * (dot(nor, vec3(0.0, 1.0, 0.0)) * 0.25 + 0.75) + 
    //bounc
    bou * 2.0 * max((-nds * 0.5 + 0.5), dot(nor, sun * vec3(1.0, -1.0, 1.0))) * max(0.0, 1. - pos.y) * (1. + fre) + 
    //spec
    sk * occ * (1.0 + fre) * 0.5
    );
    
    return  vec4(col, 1.0)    ;
}
