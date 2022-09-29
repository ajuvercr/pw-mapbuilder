use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    window::WindowResized,
};

use crate::{
    map_config::{MapConfig, MapEvent, MapType},
    planet::{HoverPlanet, Location, PlanetEvent, Player, Selected},
    HoveringUI,
};

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(mouse_events)
            .add_system(world_move)
            .add_system(handle_window_resize)
            .add_system(spawn_planet)
            .add_system(change_bg_color);
    }
}

pub fn handle_window_resize(
    mut keyboard_input_events: EventReader<WindowResized>,
    mut config: ResMut<MapConfig>,
) {
    for event in keyboard_input_events.iter() {
        config.width = event.width;
        config.height = event.height;
    }
}

pub fn mouse_events(
    mut query: Query<&mut Location, With<HoverPlanet>>,
    mut config: ResMut<MapConfig>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut cameras: Query<&mut Transform, With<Camera2d>>,
    hovering_ui: Res<HoveringUI>,
) {
    if hovering_ui.0 {
        return;
    }

    let mut loc = query.single_mut();

    for event in cursor_moved_events.iter() {
        if let Some(l) = config.update_mouse(event.position.x, event.position.y) {
            *loc = l;
        }
    }

    for event in mouse_wheel_events.iter() {
        let amount = if event.unit == MouseScrollUnit::Line {
            10.
        } else {
            1.
        };

        let z = config.zoom;

        config.set_zoom(z + event.y * amount);

        for mut cam_trans in cameras.iter_mut() {
            *cam_trans = cam_trans.with_scale(Vec3::new(1. / config.zoom, 1. / config.zoom, 1.));
        }

        if let Some(l) = config.recalculate() {
            *loc = l;
        }
    }
}

pub fn world_move(
    mut config: ResMut<MapConfig>,
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    hovering_ui: Res<HoveringUI>,
    mut cameras: Query<&mut Transform, With<Camera2d>>,
    mut location: Query<&mut Location, With<HoverPlanet>>,
) {
    if hovering_ui.0 {
        return;
    }
    let scale = 400.0;
    let mut changed = false;
    let delta = time.delta_seconds() * scale;

    let mut translate = Vec3::ZERO;

    if input.pressed(KeyCode::W) || input.pressed(KeyCode::Up) {
        translate.y += delta;
        changed = true;
    }

    if input.pressed(KeyCode::S) || input.pressed(KeyCode::Down) {
        translate.y -= delta;
        changed = true;
    }

    if input.pressed(KeyCode::D)|| input.pressed(KeyCode::Right) {
        translate.x += delta;
        changed = true;
    }
    if input.pressed(KeyCode::A) || input.pressed(KeyCode::Left) {
        translate.x -= delta;
        changed = true;
    }

    // TODO this should be a world_move event
    if changed {
        config.x -= translate.x;
        config.y -= translate.y;
        let mut loc = location.single_mut();

        if let Some(l) = config.recalculate() {
            *loc = l;
        }

        for mut cam_trans in cameras.iter_mut() {
            *cam_trans = *cam_trans * Transform::from_translation(translate);
        }
    }
}

pub fn change_bg_color(
    input: Res<Input<KeyCode>>,
    hovering_ui: Res<HoveringUI>,
    mut writer: EventWriter<MapEvent>,
) {
    if hovering_ui.0 {
        return;
    }

    if input.just_pressed(KeyCode::A) {
        writer.send(MapEvent::SetColor(Color::BLUE));
    }

    if input.just_pressed(KeyCode::D) {
        writer.send(MapEvent::SetColor(Color::PURPLE));
    }

    if input.just_pressed(KeyCode::Z) {
        writer.send(MapEvent::SetType(MapType::Squares));
    }

    if input.just_pressed(KeyCode::X) {
        writer.send(MapEvent::SetType(MapType::Triangles));
    }
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_planet(
    click: Res<Input<MouseButton>>,
    location: Query<&Location, With<HoverPlanet>>,
    planets: Query<(Entity, &Location, &Selected), Without<HoverPlanet>>,
    hovering_ui: Res<HoveringUI>,
    mut planet_events: EventWriter<PlanetEvent>,
    current_player: Res<Player>,
) {
    if hovering_ui.0 {
        return;
    }

    let loc = location.single();
    if click.just_pressed(MouseButton::Left) {
        if let Some((e, _, s)) = planets.iter().find(|(_, l, _)| *l == loc) {
            planet_events.send(PlanetEvent::SetSelected {
                id: e,
                selected: !s.0,
            });
        } else {
            planet_events.send(PlanetEvent::Create {
                loc: *location.single(),
                player: *current_player,
            });
        }
    }

    if click.just_pressed(MouseButton::Right) {
        planet_events.send_batch(
            planets
                .iter()
                .filter(|(_, l, _)| *l == loc)
                .map(|(e, _, _)| PlanetEvent::Delete { id: e }),
        );
    }
}
