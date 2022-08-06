//! A shader that uses the GLSL shading language.

use bevy::{
    prelude::*,
    sprite::MaterialMesh2dBundle,
};
use mapbuilder::{self, background::BackgroundConfig};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugin(mapbuilder::background::BackgroundPlugin)
        .add_startup_system(setup)
        .add_system(change_bg_color);

    app.run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: meshes
            .add(Mesh::from(shape::Quad::new(Vec2::new(100., 100.))))
            .into(),
        material: materials.add(ColorMaterial::from(Color::PURPLE)),
        transform: Transform::default().with_translation(Vec3::new(100., 100., 0.5)),
        ..default()
    });

    commands.spawn_bundle(Camera2dBundle::default());
}

fn change_bg_color(mut bg: ResMut<BackgroundConfig>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::A) {
        bg.color = Color::BLUE;
    }

    if input.just_pressed(KeyCode::D) {
        bg.color = Color::PURPLE;
    }
}
