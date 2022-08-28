use bevy::{prelude::*, sprite::MaterialMesh2dBundle, window::PresentMode};
use bevy_egui::EguiPlugin;
use bevy_framepace::{FramepaceSettings, Limiter};
use mapbuilder::{
    self, input, map_config::MapConfig, planet::PlanetPlugin, ui::UIPlugin, CurrentPlayer,
    HoverPlanet, HoveringUI, Location,
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
    .add_plugin(mapbuilder::background::BackgroundPlugin)
    .add_startup_system(setup)
    .add_startup_system(setup_framepace_settings)
    .add_system(transform_hover_planet);

    app.run();
}

fn setup_framepace_settings(mut settings: ResMut<FramepaceSettings>) {
    settings.limiter = Limiter::from_framerate(120.);
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windows: Res<Windows>,
) {
    let (w, h) = {
        let window = windows.get_primary().unwrap();
        (window.width(), window.height())
    };

    let config = MapConfig::new(w, h);
    commands.insert_resource(HoveringUI(false));
    commands.insert_resource(config);
    commands.insert_resource(CurrentPlayer {
        id: 0,
        color: Color::GRAY,
    });

    let mut color = Color::PURPLE;
    color.set_a(0.4);
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: meshes.add(config.mesh()).into(),
            material: materials.add(ColorMaterial::from(color)),
            ..default()
        })
        .insert_bundle((HoverPlanet, Location { x: 0, y: 0 }));

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
    mut query: Query<(&Location, &mut Transform), (With<HoverPlanet>, Changed<Location>)>,
) {
    if let Ok((loc, mut transform)) = query.get_single_mut() {
        *transform = config.location_to_transform(loc, 0.1);
    }
}
