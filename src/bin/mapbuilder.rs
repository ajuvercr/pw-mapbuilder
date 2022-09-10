use bevy::{prelude::*, sprite::MaterialMesh2dBundle, window::PresentMode};
use bevy_egui::EguiPlugin;
use bevy_framepace::{FramepaceSettings, Limiter};
use mapbuilder::{
    self, input,
    map_config::{MapConfig, MapConfigPlugin},
    planet::{HoverPlanet, Location, PlanetPlugin},
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
    .add_plugin(bevy_framepace::FramepacePlugin)
    .add_plugin(mapbuilder::LibPlugin)
    .add_plugin(UIPlugin)
    .add_plugin(input::InputPlugin)
    .add_plugin(PlanetPlugin)
    .add_plugin(MapConfigPlugin)
    .add_plugin(mapbuilder::background::BackgroundPlugin)
    .add_startup_system(setup)
    .add_startup_system(setup_framepace_settings)
    .add_system(transform_hover_planet);

    app.run();
}

fn setup_framepace_settings(mut settings: ResMut<FramepaceSettings>) {
    settings.limiter = Limiter::from_framerate(120.);
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, config: Res<MapConfig>) {
    let transform = Transform::from_xyz(0.0, 0.0, 1000.0).with_scale(Vec3::new(
        1. / config.zoom,
        1. / config.zoom,
        1.,
    ));
    commands.spawn_bundle(Camera2dBundle {
        transform,
        ..default()
    });

    let bundle = Text2dBundle {
        text: Text::from_section(
            "Hello Bevy",
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 100.0,
                color: Color::WHITE,
            },
        )
        .with_alignment(TextAlignment::CENTER),
        transform: Transform::from_scale(Vec3 {
            x: 1. / config.zoom,
            y: 1. / config.zoom,
            z: 0.,
        }),
        ..default()
    };
    commands.spawn_bundle(bundle);
}

fn transform_hover_planet(
    config: Res<MapConfig>,
    mut query: Query<(&Location, &mut Transform), (With<HoverPlanet>, Changed<Location>)>,
) {
    if let Ok((loc, mut transform)) = query.get_single_mut() {
        *transform = config.location_to_transform(loc, 0.1);
    }
}
