
const sunx: f32 = 1.0; 
const suny: f32 =  2.3;
const sunz: f32 =   2.0;

fn sky(rd: vec3<f32>) -> vec3<f32>{
    let sun = normalize(vec3(sunx,suny,sunz));
    let rds = dot(rd,sun);
    let suc = smoothstep(0.999,1.0,dot(rd,sun));
    return (exp(rds*rds*rds-vec3(4.,2.,1.)*( rd.y  + 1.0)*0.7))*3.5+suc;

}

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
