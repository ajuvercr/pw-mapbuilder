use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    sprite::Mesh2dHandle,
    window::WindowResized,
};

use crate::{
    map_config::{MapConfig, MapType},
    planet::{PlanetEvent, Player},
    HoverPlanet, HoveringUI, Location,
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

    if input.pressed(KeyCode::W) {
        translate.y += delta;
        changed = true;
    }

    if input.pressed(KeyCode::S) {
        translate.y -= delta;
        changed = true;
    }

    if input.pressed(KeyCode::D) {
        translate.x += delta;
        changed = true;
    }
    if input.pressed(KeyCode::A) {
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
    mut bg: ResMut<MapConfig>,
    input: Res<Input<KeyCode>>,
    hovering_ui: Res<HoveringUI>,
    mut locations: Query<(
        &mut Mesh2dHandle,
        &mut Transform,
        &Location,
        Option<&HoverPlanet>,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if hovering_ui.0 {
        return;
    }
    if input.just_pressed(KeyCode::A) {
        bg.bg_color = Color::BLUE;
    }

    if input.just_pressed(KeyCode::D) {
        bg.bg_color = Color::PURPLE;
    }

    let mut update_meshes = false;
    if input.just_pressed(KeyCode::Z) {
        bg.ty = MapType::Squares;
        update_meshes = true;
    }

    if input.just_pressed(KeyCode::X) {
        bg.ty = MapType::Triangles;
        update_meshes = true;
    }

    if update_meshes {
        let mesh_handle: Mesh2dHandle = meshes.add(bg.mesh()).into();

        for (mut l, mut t, loc, h) in locations.iter_mut() {
            *l = mesh_handle.clone();
            let z = if h.is_some() { 0.1 } else { 0.0 };
            *t = bg.location_to_transform(loc, z);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_planet(
    click: Res<Input<MouseButton>>,
    location: Query<&Location, With<HoverPlanet>>,
    planets: Query<(Entity, &Location), Without<HoverPlanet>>,
    hovering_ui: Res<HoveringUI>,
    mut planet_events: EventWriter<PlanetEvent>,
    current_player: Res<Player>,
) {
    if hovering_ui.0 {
        return;
    }

    let loc = location.single();
    if click.just_pressed(MouseButton::Left) {
        planet_events.send(PlanetEvent::Create {
            loc: *location.single(),
            player: *current_player,
        });
    }

    if click.just_pressed(MouseButton::Right) {
        planet_events.send_batch(
            planets
                .iter()
                .filter(|(_, l)| *l == loc)
                .map(|(e, _)| PlanetEvent::Delete { id: e }),
        );
    }
}
