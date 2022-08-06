//! A shader that uses the GLSL shading language.

use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, CompareFunction, DepthBiasState, DepthStencilState,
            RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError, StencilState,
            TextureFormat,
        },
        texture::BevyDefault,
    },
    sprite::{
        Material2d, Material2dKey, Material2dPipeline, Material2dPlugin, MaterialMesh2dBundle,
    },
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(Material2dPlugin::<CustomMaterial>::default())
        .add_startup_system(setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut col_materials: ResMut<Assets<ColorMaterial>>,
) {
    // cube
    commands.spawn().insert_bundle(MaterialMesh2dBundle {
        mesh: meshes
            .add(Mesh::from(shape::Quad::new(Vec2::new(2., 2.))))
            .into(),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        material: materials.add(CustomMaterial { color: Color::BLUE }),
        ..default()
    });
    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
        transform: Transform::default().with_scale(Vec3::splat(128.)),
        material: col_materials.add(ColorMaterial::from(Color::PURPLE)),
        ..default()
    });
    commands.spawn_bundle(Camera2dBundle::default());
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, Clone, TypeUuid)]
#[uuid = "4ee9c363-1124-4113-890e-199d81b00281"]
pub struct CustomMaterial {
    #[uniform(0)]
    color: Color,
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
/// When using the GLSL shading language for your shader, the specialize method must be overriden.
impl Material2d for CustomMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/custom_material.vert".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/custom_material.frag".into()
    }

    // Bevy assumes by default that vertex shaders use the "vertex" entry point
    // and fragment shaders use the "fragment" entry point (for WGSL shaders).
    // GLSL uses "main" as the entry point, so we must override the defaults here
    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.entry_point = "main".into();
        descriptor.fragment.as_mut().unwrap().entry_point = "main".into();

        Ok(())
    }
}
