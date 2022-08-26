use bevy::{prelude::*, sprite::MaterialMesh2dBundle, window::RequestRedraw, winit::WinitSettings};
use bevy_egui::EguiPlugin;
use mapbuilder::{
    self, input, map_config::MapConfig, ui, CurrentPlayer, HoverPlanet, HoveringUI, Location, FPS,
};

#[allow(dead_code)]
#[derive(Debug)]
enum ExampleMode {
    Game,
    Application,
    ApplicationWithRedraw,
}

/// Update winit based on the current `ExampleMode`
fn update_winit(
    mode: Res<ExampleMode>,
    mut event: EventWriter<RequestRedraw>,
    mut winit_config: ResMut<WinitSettings>,
) {
    use ExampleMode::*;
    *winit_config = match *mode {
        Game => {
            // In the default `WinitConfig::game()` mode:
            //   * When focused: the event loop runs as fast as possible
            //   * When not focused: the event loop runs as fast as possible
            WinitSettings::game()
        }
        Application => {
            // While in `WinitConfig::desktop_app()` mode:
            //   * When focused: the app will update any time a winit event (e.g. the window is
            //     moved/resized, the mouse moves, a button is pressed, etc.), a [`RequestRedraw`]
            //     event is received, or after 5 seconds if the app has not updated.
            //   * When not focused: the app will update when the window is directly interacted with
            //     (e.g. the mouse hovers over a visible part of the out of focus window), a
            //     [`RequestRedraw`] event is received, or one minute has passed without the app
            //     updating.
            WinitSettings::desktop_app()
        }
        ApplicationWithRedraw => {
            // Sending a `RequestRedraw` event is useful when you want the app to update the next
            // frame regardless of any user input. For example, your application might use
            // `WinitConfig::desktop_app()` to reduce power use, but UI animations need to play even
            // when there are no inputs, so you send redraw requests while the animation is playing.
            event.send(RequestRedraw);
            WinitSettings::desktop_app()
        }
    };
}

fn main() {
    let mut app = App::new();

    app.insert_resource(WinitSettings::desktop_app())
        .insert_resource(ExampleMode::ApplicationWithRedraw)
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_system(ui::ui_system)
        .add_system(ui::ui_editor)
        .add_system(ui::change_planet_color)
        .add_plugin(mapbuilder::background::BackgroundPlugin)
        .add_startup_system(setup)
        .add_system(transform_hover_planet)
        .add_system(input::mouse_events)
        .add_system(input::world_move)
        .add_system(input::handle_window_resize)
        .add_system(input::spawn_planet)
        .add_system(input::change_bg_color)
        .add_system(mapbuilder::fps)
        .add_system(update_winit);

    app.run();
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
    commands.insert_resource(FPS(0));
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
