use std::marker::PhantomData;

use bevy::{
    core::Pod,
    core_pipeline::core_2d::Transparent2d,
    ecs::system::{lifetimeless::SRes, SystemParamItem},
    math::vec4,
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        mesh::Indices,
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState, Buffer,
            BufferBindingType, BufferDescriptor, BufferSize, BufferUsages, ColorTargetState,
            ColorWrites, Face, FragmentState, FrontFace, MultisampleState, PipelineCache,
            PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipelineDescriptor, ShaderStages,
            SpecializedRenderPipeline, SpecializedRenderPipelines, TextureFormat,
            VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::BevyDefault,
        view::{VisibleEntities, ViewUniforms, ExtractedView},
        Extract, RenderApp, RenderStage,
    },
    sprite::{
        DrawMesh2d, Mesh2dHandle, Mesh2dPipeline, Mesh2dPipelineKey,
        SetMesh2dViewBindGroup, Mesh2dViewBindGroup,
    },
    utils::FloatOrd,
};

fn setup_background(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    // Let's define the mesh for the object we want to draw: a nice star.
    // We will specify here what kind of topology is used to define the mesh,
    // that is, how triangles are built from the vertices. We will use a
    // triangle list, meaning that each vertex of the triangle has to be
    // specified.
    let mut background = Mesh::new(PrimitiveTopology::TriangleList);
    let mi = -0.8;
    let ma = 0.8;
    let v_pos = vec![[mi, mi, 0.0], [mi, ma, 0.0], [ma, mi, 0.0], [ma, ma, 0.0]];

    background.insert_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
    // And a RGB color attribute as well
    let mut v_color: Vec<u32> = vec![Color::BLACK.as_linear_rgba_u32()];
    v_color.extend_from_slice(&[Color::YELLOW.as_linear_rgba_u32(); 3]);

    let indices = vec![2, 1, 0, 3, 1, 2];
    background.set_indices(Some(Indices::U32(indices)));

    // We can now spawn the entities for the star and the camera
    commands.spawn_bundle((
        // We use a marker component to identify the custom colored meshes
        BackgroundMesh2d::default(),
        // The `Handle<Mesh>` needs to be wrapped in a `Mesh2dHandle` to use 2d rendering instead of 3d
        Mesh2dHandle(meshes.add(background)),
        // These other components are needed for 2d meshes to be rendered
        Transform::default(),
        GlobalTransform::default(),
        Visibility::default(),
        ComputedVisibility::default(),
    ));
}

#[derive(Component, Default)]
pub struct BackgroundMesh2d;

/// Custom pipeline for 2d meshes with vertex colors
pub struct BackgroundMesh2dPipeline {
    /// this pipeline wraps the standard [`Mesh2dPipeline`]
    mesh2d_pipeline: Mesh2dPipeline,
    color_uniform_layout: BindGroupLayout,

    shader_handle: Handle<Shader>,
}

impl FromWorld for BackgroundMesh2dPipeline {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let shader_handle: Handle<Shader> = asset_server.load("shaders/background_shader.wgsl");

        let render_device = world.resource::<RenderDevice>();
        let color_uniform_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("background uniform bind group"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new((std::mem::size_of::<f32>() * 4) as u64),
                    },
                    count: None,
                }],
            });

        Self {
            mesh2d_pipeline: Mesh2dPipeline::from_world(world),
            color_uniform_layout,
            shader_handle,
        }
    }
}

// We implement `SpecializedPipeline` to customize the default rendering from `Mesh2dPipeline`
impl SpecializedRenderPipeline for BackgroundMesh2dPipeline {
    type Key = Mesh2dPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        // Customize how to store the meshes' vertex attributes in the vertex buffer
        // Our meshes only have position and color
        let formats = vec![
            // Position
            VertexFormat::Float32x3,
        ];

        let vertex_layout =
            VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, formats);

        RenderPipelineDescriptor {
            vertex: VertexState {
                // Use our custom shader
                shader: self.shader_handle.clone(),
                entry_point: "vertex".into(),
                shader_defs: Vec::new(),
                // Use our custom vertex buffer
                buffers: vec![vertex_layout],
            },
            fragment: Some(FragmentState {
                // Use our custom shader
                shader: self.shader_handle.clone(),
                shader_defs: Vec::new(),
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            // Use the two standard uniforms for 2d meshes
            layout: Some(vec![
                // Bind group 0 is the view uniform
                self.mesh2d_pipeline.view_layout.clone(),
                // Bind group 1 is the mesh uniform
                self.color_uniform_layout.clone(),
            ]),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: key.primitive_topology(),
                strip_index_format: None,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("background_mesh2d_pipeline".into()),
        }
    }
}

// This specifies how to render a colored 2d mesh
type DrawBackgroundMesh2d = (
    // Set the pipeline
    SetItemPipeline,
    // Set the view uniform as bind group 0
    SetMesh2dViewBindGroup<0>,
    // Set the mesh uniform as bind group 1
    // SetMesh2dBindGroup<1>,
    SetColorUniformBindGroup<1, BackgroundConfig>,
    // Draw the mesh
    DrawMesh2d,
);

pub struct BackgroundConfig {
    pub color: Color,
}

impl Default for BackgroundConfig {
    fn default() -> Self {
        Self { color: Color::BLUE }
    }
}

#[derive(Default)]
struct ExtractedColor {
    color: Vec4,
}

trait GetPod {
    type Inner: Pod;

    fn get(&self) -> Self::Inner;
}

impl GetPod for ExtractedColor {
    type Inner = Vec4;

    fn get(&self) -> Self::Inner {
        self.color
    }
}

impl ExtractResource for ExtractedColor {
    type Source = BackgroundConfig;

    fn extract_resource(unif: &Self::Source) -> Self {
        let color = unif.color;
        ExtractedColor {
            color: vec4(color.r(), color.g(), color.b(), color.a()),
        }
    }
}

// write the extracted time into the corresponding uniform buffer
fn prepare_uniform<T: Send + Sync + 'static, R: ExtractResource<Source = T> + GetPod>(
    time: Res<R>,
    time_meta: ResMut<UniformMeta<T>>,
    render_queue: Res<RenderQueue>,
) {
    render_queue.write_buffer(&time_meta.buffer, 0, bevy::core::cast_slice(&[time.get()]));
}

struct UniformMeta<T> {
    pd: PhantomData<T>,
    buffer: Buffer,
    bind_group: Option<BindGroup>,
}

pub fn queue_mesh2d_view_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    mesh2d_pipeline: Res<BackgroundMesh2dPipeline>,
    view_uniforms: Res<ViewUniforms>,
    views: Query<Entity, With<ExtractedView>>,
) {
    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        for entity in &views {
            let view_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: view_binding.clone(),
                }],
                label: Some("mesh2d_view_bind_group"),
                layout: &mesh2d_pipeline.mesh2d_pipeline.view_layout,
            });

            commands.entity(entity).insert(Mesh2dViewBindGroup {
                value: view_bind_group,
            });
        }
    }
}

// create a bind group for the time uniform buffer
fn queue_time_bind_group<T: Send + Sync + 'static>(
    render_device: Res<RenderDevice>,
    mut uniform_meta: ResMut<UniformMeta<T>>,
    pipeline: Res<BackgroundMesh2dPipeline>,
) {
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.color_uniform_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: uniform_meta.buffer.as_entire_binding(),
        }],
    });
    uniform_meta.bind_group = Some(bind_group);
}

#[derive(Default)]
struct SetColorUniformBindGroup<const I: usize, T>(PhantomData<T>);

impl<const I: usize, T: Send + Sync + 'static> EntityRenderCommand
    for SetColorUniformBindGroup<I, T>
{
    type Param = SRes<UniformMeta<T>>;

    fn render<'w>(
        _view: Entity,
        _item: Entity,
        time_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let time_bind_group = time_meta.into_inner().bind_group.as_ref().unwrap();
        pass.set_bind_group(I, time_bind_group, &[]);

        RenderCommandResult::Success
    }
}

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        // Load our custom shader
        app.insert_resource::<BackgroundConfig>(BackgroundConfig {
            color: Color::ORANGE_RED,
        });
        app.add_startup_system(setup_background);

        let render_device = app.world.resource::<RenderDevice>();
        let buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("time uniform buffer"),
            size: (std::mem::size_of::<f32>() * 4) as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        app.add_plugin(ExtractResourcePlugin::<ExtractedColor>::default());

        // Register our custom draw function and pipeline, and add our render systems
        let render_app = app.get_sub_app_mut(RenderApp).unwrap();
        render_app
            .add_render_command::<Transparent2d, DrawBackgroundMesh2d>()
            .init_resource::<BackgroundMesh2dPipeline>()
            .insert_resource(UniformMeta::<BackgroundConfig> {
                buffer,
                pd: PhantomData,
                bind_group: None,
            })
            .init_resource::<SpecializedRenderPipelines<BackgroundMesh2dPipeline>>()
            .add_system_to_stage(
                RenderStage::Prepare,
                prepare_uniform::<BackgroundConfig, ExtractedColor>,
            )
            .add_system_to_stage(RenderStage::Extract, extract_colored_mesh2d)
            .add_system_to_stage(RenderStage::Queue, queue_colored_mesh2d)
            .add_system_to_stage(
                RenderStage::Queue,
                queue_time_bind_group::<BackgroundConfig>,
            );
    }
}

/// Extract the [`ColoredMesh2d`] marker component into the render app
pub fn extract_colored_mesh2d(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    // When extracting, you must use `Extract` to mark the `SystemParam`s
    // which should be taken from the main world.
    query: Extract<Query<(Entity, &ComputedVisibility), With<BackgroundMesh2d>>>,
) {
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, computed_visibility) in query.iter() {
        if !computed_visibility.is_visible() {
            continue;
        }
        values.push((entity, (BackgroundMesh2d,)));
    }
    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}


/// Queue the 2d meshes marked with [`ColoredMesh2d`] using our custom pipeline and draw function
#[allow(clippy::too_many_arguments)]
pub fn queue_colored_mesh2d(
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    colored_mesh2d_pipeline: Res<BackgroundMesh2dPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<BackgroundMesh2dPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    msaa: Res<Msaa>,
    render_meshes: Res<RenderAssets<Mesh>>,
    colored_mesh2d: Query<&Mesh2dHandle, With<BackgroundMesh2d>>,
    mut views: Query<(&VisibleEntities, &mut RenderPhase<Transparent2d>)>,
) {
    if colored_mesh2d.is_empty() {
        return;
    }
    // Iterate each view (a camera is a view)
    for (visible_entities, mut transparent_phase) in &mut views {
        let draw_colored_mesh2d = transparent_draw_functions
            .read()
            .get_id::<DrawBackgroundMesh2d>()
            .unwrap();

        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples);

        // Queue all entities visible to that view
        for visible_entity in &visible_entities.entities {
            if let Ok(mesh2d_handle) = colored_mesh2d.get(*visible_entity) {
                // Get our specialized pipeline
                let mut mesh2d_key = mesh_key;
                if let Some(mesh) = render_meshes.get(&mesh2d_handle.0) {
                    mesh2d_key |=
                        Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology);
                }

                let pipeline_id =
                    pipelines.specialize(&mut pipeline_cache, &colored_mesh2d_pipeline, mesh2d_key);

                transparent_phase.add(Transparent2d {
                    entity: *visible_entity,
                    draw_function: draw_colored_mesh2d,
                    pipeline: pipeline_id,
                    // The 2d render items are sorted according to their z value before rendering,
                    // so fake background z value
                    sort_key: FloatOrd(-1.),
                    // This material is not batched
                    batch_range: None,
                });
            }
        }
    }
}
