//! A shader that uses the GLSL shading language.

use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    sprite::MaterialMesh2dBundle, window::WindowResized,
};
use mapbuilder::{self, background::BackgroundConfig};

#[derive(Component, Debug, Default)]
struct HoverPlanet;

#[derive(Component, Clone, Copy, Debug, Default)]
struct Location {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, Debug)]
struct MapConfig {
    zoom: f32,

    width: f32,
    height: f32,

    x: f32,
    y: f32,

    mouse_x: Option<f32>,
    mouse_y: Option<f32>,
}

impl MapConfig {
    fn new(width: f32, height: f32) -> Self {
        Self {
            zoom: 100.,
            width,
            height,
            x: 0.,
            y: 0.,

            mouse_x: None,
            mouse_y: None,
        }
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        let x = self.x / self.zoom;
        let y = self.y / self.zoom;

        self.zoom = zoom.max(10.);

        self.x = x * self.zoom;
        self.y = y * self.zoom;
    }

    pub fn recalculate(&self) -> Option<Location> {
        self.contains(
            (self.mouse_x? - self.x) / self.zoom,
            (self.mouse_y? - self.y) / self.zoom,
        )
    }

    pub fn update_mouse(&mut self, x: f32, y: f32) -> Option<Location> {
        self.mouse_x = Some(x - self.width * 0.5);
        self.mouse_y = Some(y - self.height * 0.5);

        self.recalculate()
    }

    fn contains(&self, x: f32, y: f32) -> Option<Location> {
        let dx = x.signum() * 0.5;
        let dy = y.signum() * 0.5;
        let out = Location {
            x: (x + dx) as i32,
            y: (y + dy) as i32,
        };
        Some(out)
    }

    pub fn location_to_transform(&self, location: &Location, z: f32) -> Transform {
        Transform::default().with_translation(Vec3::new(location.x as f32, location.y as f32, z))
    }
}

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugin(mapbuilder::background::BackgroundPlugin)
        .add_startup_system(setup)
        .add_system(transform_hover_planet)
        .add_system(mouse_events)
        .add_system(world_move)
        .add_system(handle_window_resize)
        .add_system(sync_config)
        .add_system(spawn_planet)
        .add_system(change_bg_color);

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
    commands.insert_resource(config);

    let mut color = Color::PURPLE;
    color.set_a(0.4);
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Quad::new(Vec2::new(1., 1.))))
                .into(),
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

fn handle_window_resize(
    mut keyboard_input_events: EventReader<WindowResized>,
    mut config: ResMut<MapConfig>,
) {
    for event in keyboard_input_events.iter() { 
        config.width = event.width;
        config.height = event.height;
    }
}

fn transform_hover_planet(
    config: Res<MapConfig>,
    mut query: Query<(&Location, &mut Transform), (With<HoverPlanet>, Changed<Location>)>,
) {
    if let Ok((loc, mut transform)) = query.get_single_mut() {
        *transform = config.location_to_transform(loc, 0.1);
    }
}

/// This system prints out all mouse events as they come in
fn mouse_events(
    mut query: Query<&mut Location, With<HoverPlanet>>,
    mut config: ResMut<MapConfig>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut cameras: Query<&mut Transform, With<Camera2d>>,
) {
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

fn world_move(
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

fn sync_config(config: Res<MapConfig>, mut bg_config: ResMut<BackgroundConfig>) {
    bg_config.height = config.height;
    bg_config.width = config.width;
    bg_config.x = config.x;
    bg_config.y = config.y;
    bg_config.zoom = config.zoom;
}

fn change_bg_color(mut bg: ResMut<BackgroundConfig>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::A) {
        bg.set_color(Color::BLUE);
    }

    if input.just_pressed(KeyCode::D) {
        bg.set_color(Color::PURPLE);
    }
}

fn spawn_planet(
    mut commands: Commands,
    click: Res<Input<MouseButton>>,
    location: Query<&Location, With<HoverPlanet>>,
    planets: Query<(Entity, &Location), Without<HoverPlanet>>,
    config: Res<MapConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let loc = location.single();
    if click.just_pressed(MouseButton::Left) {
        let transform = config.location_to_transform(loc, 0.);

        commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: meshes
                    .add(Mesh::from(shape::Quad::new(Vec2::new(1., 1.))))
                    .into(),
                material: materials.add(ColorMaterial::from(Color::WHITE)),
                transform,
                ..default()
            })
            .insert(*loc);
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
