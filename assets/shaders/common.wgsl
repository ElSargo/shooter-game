
const sunx: f32 = 1.0; 
const suny: f32 =  2.3;
const sunz: f32 =   2.0;

fn sky(rd: vec3<f32>) -> vec3<f32>{
    let sun = normalize(vec3(sunx,suny,sunz));
    let rds = dot(rd,sun);
    let suc = smoothstep(0.999,1.0,dot(rd,sun));
    return (exp(rds*rds*rds-vec3(4.,2.,1.)*( rd.y  + 1.0)*0.7))*3.5+suc;
}