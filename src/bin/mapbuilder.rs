use bevy::{prelude::*, window::PresentMode, sprite::Mesh2dHandle};
use bevy_egui::EguiPlugin;

#[cfg(not(target_family = "wasm"))]
use bevy_framepace::{FramepaceSettings, Limiter};
use mapbuilder::{
    self, input,
    map_config::{MapConfig, MapConfigPlugin},
    planet::{HoverPlanet, Location, PlanetPlugin},
    scene,
    ui::UIPlugin,
};

fn main() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        present_mode: PresentMode::AutoNoVsync,
        ..default()
    })
    .add_plugins(DefaultPlugins)
    .add_plugin(EguiPlugin)
    .add_plugin(mapbuilder::LibPlugin)
    .add_plugin(MapConfigPlugin)
    .add_plugin(scene::ScenePlugin)
    .add_plugin(UIPlugin)
    .add_plugin(input::InputPlugin)
    .add_plugin(PlanetPlugin)
    .add_plugin(mapbuilder::background::BackgroundPlugin)
    .add_startup_system(setup)
    .add_system(transform_hover_planet);

    #[cfg(not(target_family = "wasm"))]
    app.add_plugin(bevy_framepace::FramepacePlugin)
        .add_startup_system(setup_framepace_settings);

    app.run();
}

#[cfg(not(target_family = "wasm"))]
fn setup_framepace_settings(mut settings: ResMut<FramepaceSettings>) {
    settings.limiter = Limiter::from_framerate(120.);
}

fn setup(mut commands: Commands, config: Res<MapConfig>) {
    let transform = Transform::from_xyz(0.0, 0.0, 1000.0).with_scale(Vec3::new(
        1. / config.zoom,
        1. / config.zoom,
        1.,
    ));
    commands.spawn_bundle(Camera2dBundle {
        transform,
        ..default()
    });
}

fn transform_hover_planet(
    config: Res<MapConfig>,
    mut query: Query<(&Location, &mut Transform, &mut Mesh2dHandle), (With<HoverPlanet>, Changed<Location>)>,
) {
    if let Ok((loc, mut transform, mut mesh)) = query.get_single_mut() {
        *transform = config.shape_transform(loc, 0.1);
        *mesh = config.mesh(loc).into();
    }
}
