//! A shader and a material that uses it.

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};

pub struct MainMaterialPlugin;
impl Plugin for MainMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<MainMaterial>::default());
    }
}

/// set up a simple 3D scene

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for MainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/main_material.wgsl".into()
    }
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone, Default, Reflect)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct MainMaterial {
    #[uniform(0)]
    pub color: Color,
    #[texture(1)]
    #[sampler(2)]
    pub boxes: Option<Handle<Image>>,
}
