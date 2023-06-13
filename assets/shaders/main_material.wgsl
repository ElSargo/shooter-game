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


fn length2(v: vec3<f32>) -> f32 { return dot(v, v); }
fn segShadow(ro: vec3<f32>, rd: vec3<f32>, pa: vec3<f32>, sh: f32) -> f32 {
    var sh = sh;
    let k1 = 1.0 - rd.x * rd.x; // k = dot(rd.yz,rd.yz);
    let k4 = (ro.x - pa.x) * k1;
    let k6 = (ro.x + pa.x) * k1;
    let k5 = ro.yz * k1;
    let k7 = pa.yz * k1;
    let k2 = -dot(ro.yz, rd.yz);
    let k3 = pa.yz * rd.yz;
    let j: u32 = u32(1);
    for (var i: u32 = u32(0); i < u32(4); i++) {
        var sc = vec2(f32(i & u32(1)), f32(i << j)) * 2.0 - 1.0;
        var ss: vec2<f32> = vec2(f32(sc.x), f32(sc.y));
        let thx = k2 + dot(ss, k3);
        if thx < 0.0 {continue;} // behind
        let thy = clamp(-rd.x * thx, k4, k6);
        sh = min(sh, length2(vec3(thy, k5 - k7 * ss) + rd * thx) / (thx * thx));
    }
    return sh;
}
fn boxSoftShadow(roo: vec3<f32>, rdd: vec3<f32>, rad: vec3<f32>, sk: f32) -> f32 {

    let m = 1.0 / rdd;
    let n = m * roo;
    let k = abs(m) * rad;

    let t1 = -n - k;
    let t2 = -n + k;

    let tN = max(max(t1.x, t1.y), t1.z);
    let tF = min(min(t2.x, t2.y), t2.z);
	
    // fake soft shadow
    if tF < 0.0 {return 1.0;}
    let sh = clamp(0.3 * sk * (tN - tF) / tN, 0.0, 1.0);
    return sh * sh * (3.0 - 2.0 * sh);
}  

// fn boxSoftShadow(ro: vec3<f32>, rd: vec3<f32>, rad: vec3<f32>, sk: f32) -> f32 { // shadow softness (try 8.0) 

//     let m = 1.0 / rd;
//     let n = m * ro;
//     let k = abs(m) * rad;
//     let t1 = -n - k;
//     let t2 = -n + k;

//     let  tN = max(max(t1.x, t1.y), t1.z);
//     let  tF = min(min(t2.x, t2.y), t2.z);

//     if tN > tF || tF < 0.0 {
//         var sh = 1.0;
//         sh = segShadow(ro.xyz, rd.xyz, rad.xyz, sh);
//         sh = segShadow(ro.yzx, rd.yzx, rad.yzx, sh);
//         sh = segShadow(ro.zxy, rd.zxy, rad.zxy, sh);
//         sh = clamp(sk * sqrt(sh), 0.0, 1.0);
//         return sh * sh * (3.0 - 2.0 * sh);
//     }
//     return 0.0;
// }

fn boxIntersection(ro: vec3<f32>, rd: vec3<f32>, rad: vec3<f32>) -> vec2<f32> {
    let m = 1.0 / rd;
    let n = m * ro;
    let k = abs(m) * rad;
    let t1 = -n - k;
    let t2 = -n + k;

    let tN = max(max(t1.x, t1.y), t1.z);
    let tF = min(min(t2.x, t2.y), t2.z);

    if tN > tF || tF < 0.0 { return vec2(-1.0);} // no intersection


    return vec2(tN, tF);
}
@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let sun = normalize(vec3(sunx,suny,sunz));
    let pos = world_position.xyz;
    let nor = normalize(world_normal.xyz);
    let nds = dot(nor, sun);
    let rd = -normalize(view.world_position.xyz - world_position.xyz);
    let rfl = reflect(rd, nor);
    let fre = pow(dot(rd, rfl) * 0.5 + 0.5, 5.0);
    let softness = 12.0;
    let scl = vec3(1.0, 0.8, 0.6);
    let skc = vec3(0.6, 0.7, 1.0);
    let sca = (pos.xz / 30.0 + 0.5) * 30.0;
    let c: vec2<i32> = vec2(i32(sca.x), i32(sca.y));
    let sa = floor(pos.xz);
    var sha = 1.0;
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
        occ = smoothstep(0.0, 0.1, pos.y);
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
    occ = occ * 0.2 + 0.7;
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
