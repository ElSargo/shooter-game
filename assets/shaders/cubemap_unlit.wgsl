#import bevy_pbr::mesh_view_bindings
#import "shaders/common.wgsl"

#ifdef CUBEMAP_ARRAY
@group(1) @binding(0)
var base_color_texture: texture_cube_array<f32>;
#else
@group(1) @binding(0)
var base_color_texture: texture_cube<f32>;
#endif

@group(1) @binding(1)
var base_color_sampler: sampler;


@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let rd = normalize(world_position.xyz );
    // return textureSample(
    //     base_color_texture,
    //     base_color_sampler,
    //     fragment_position_view_lh
    // );
    return vec4(
        sky(rd),
        1.0
    );
}
