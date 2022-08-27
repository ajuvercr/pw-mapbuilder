use bevy::{
    prelude::{Color, Component, Local, Res, ResMut, Plugin},
    time::Time,
};

pub mod background;
pub mod input;
pub mod map_config;
pub mod ui;

pub struct HoveringUI(pub bool);

#[derive(Component, Debug, Default)]
pub struct HoverPlanet;

#[derive(Debug, Default)]
pub struct CurrentPlayer {
    pub id: u32,
    pub color: Color,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Location {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Debug, Default)]
pub struct PlanetData {
    pub player: u32,
    pub name: String,
}

pub struct FPS(pub u32);

pub fn fps(time: Res<Time>, mut cur: Local<f32>, mut frames: ResMut<FPS>) {
    *cur += time.delta_seconds();

    if *cur > 0.1 {
        frames.0 = (1. / time.delta_seconds()) as u32;
        *cur = 0.;
    }
}

pub struct LibPlugin;

impl Plugin for LibPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        use rnglib::{Language,RNG};
        app.insert_resource(RNG::new(&Language::Curse).unwrap());
        app.insert_resource(FPS(0));
        app.add_system(fps);
    }
}
