//! A shader that uses the GLSL shading language.

use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    sprite::MaterialMesh2dBundle,
};
use mapbuilder::{self, background::BackgroundConfig};

#[derive(Component, Debug, Default)]
struct HoverPlanet;

#[derive(Component, Clone, Copy, Debug, Default)]
struct Location {
    x: i32,
    y: i32,
}

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

    pub fn recalculate(&self) -> Option<Location> {
        self.contains(self.mouse_x?, self.mouse_y?)
    }

    pub fn update_mouse(&mut self, x: f32, y: f32) -> Option<Location> {
        self.mouse_x = Some(x);
        self.mouse_y = Some(y);

        self.contains(x, y)
    }

    fn contains(&self, x: f32, y: f32) -> Option<Location> {
        let x = (x - self.width * 0.5 - self.x) / self.zoom;
        let y = (y - self.height * 0.5 - self.y) / self.zoom;
        let dx = x.signum() * 0.5;
        let dy = y.signum() * 0.5;
        let out = Location {
            x: (x + dx) as i32,
            y: (y + dy) as i32,
        };
        Some(out)
    }

    pub fn location_to_transform(&self, location: &Location, z: f32) -> Transform {
        Transform::default()
            .with_scale(Vec3::splat(self.zoom))
            .with_translation(Vec3::new(
                location.x as f32 * self.zoom,
                location.y as f32 * self.zoom,
                z,
            ))
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
    commands.insert_resource(MapConfig::new(w, h));

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

    commands.spawn_bundle(Camera2dBundle::default());
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
        config.zoom += event.y * amount;

        if let Some(l) = config.recalculate() {
            *loc = l;
        }
    }
}

fn world_move(mut config: ResMut<MapConfig>, time: Res<Time>, input: Res<Input<KeyCode>>) {
    if input.pressed(KeyCode::W) {
        config.y += time.delta_seconds() * 10.;
    }

    if input.pressed(KeyCode::S) {
        config.y -= time.delta_seconds() * 10.;
    }

    if input.pressed(KeyCode::D) {
        config.x += time.delta_seconds() * 10.;
    }
    if input.pressed(KeyCode::A) {
        config.x -= time.delta_seconds() * 10.;
    }
}

fn change_bg_color(mut bg: ResMut<BackgroundConfig>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::A) {
        bg.color = Color::BLUE;
    }

    if input.just_pressed(KeyCode::D) {
        bg.color = Color::PURPLE;
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

        commands.spawn_bundle(MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Quad::new(Vec2::new(1., 1.))))
                .into(),
            material: materials.add(ColorMaterial::from(Color::WHITE)),
            transform,
            ..default()
        }).insert(*loc);
    }

    if click.just_pressed(MouseButton::Right) {
        println!("just clicked!");
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
