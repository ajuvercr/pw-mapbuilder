use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::WindowResized,
};

use rnglib::RNG;

use crate::{
    map_config::{MapConfig, MapType},
    CurrentPlayer, HoverPlanet, HoveringUI, Location, PlanetData,
};

pub fn handle_window_resize(
    mut keyboard_input_events: EventReader<WindowResized>,
    mut config: ResMut<MapConfig>,
) {
    for event in keyboard_input_events.iter() {
        config.width = event.width;
        config.height = event.height;
    }
}

/// This system prints out all mouse events as they come in
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
    mut cameras: Query<&mut Transform, With<Camera2d>>,
    mut location: Query<&mut Location, With<HoverPlanet>>,
) {
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
    mut locations: Query<(
        &mut Mesh2dHandle,
        &mut Transform,
        &Location,
        Option<&HoverPlanet>,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
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
    mut commands: Commands,
    click: Res<Input<MouseButton>>,
    location: Query<&Location, With<HoverPlanet>>,
    planets: Query<(Entity, &Location), Without<HoverPlanet>>,
    config: Res<MapConfig>,
    current_player: Res<CurrentPlayer>,
    hovering_ui: Res<HoveringUI>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    generator: Res<RNG>,
) {
    if hovering_ui.0 {
        return;
    }

    let loc = location.single();
    if click.just_pressed(MouseButton::Left) {
        let transform = config.location_to_transform(loc, 0.);
        let planet_data = PlanetData {
            player: current_player.id,
            name: generator.generate_name(),
        };

        let location = *loc;
        commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: meshes.add(config.mesh()).into(),
                material: materials.add(ColorMaterial::from(current_player.color)),
                transform,
                ..default()
            })
            .insert(location)
            .insert(planet_data);
    }

    if click.just_pressed(MouseButton::Right) {
        for entity in planets
            .iter()
            .filter(|(_, l)| l.x == loc.x && l.y == loc.y)
            .map(|(e, _)| e)
        {
            info!("despawning {:?}", entity);
            commands.entity(entity).despawn_recursive();
        }
    }
}
