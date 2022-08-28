use bevy::{
    prelude::{Color, Component, Local, Plugin, Res, ResMut},
    time::Time,
};
use egui::Color32;
use planet::Player;

pub mod background;
pub mod input;
pub mod map_config;
pub mod planet;
pub mod ui;

pub struct HoveringUI(pub bool);

#[derive(Component, Debug, Default)]
pub struct HoverPlanet;

#[derive(Debug, Default)]
pub struct CurrentPlayer {
    pub id: usize,
    pub color: Color,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Debug)]
pub struct PlanetData {
    pub player: Player,
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
        use rnglib::{Language, RNG};
        app.insert_resource(RNG::new(&Language::Curse).unwrap());
        app.insert_resource(FPS(0));
        app.add_system(fps);
    }
}

pub const ZEUS: Color32 = Color32::from_rgb(255, 128, 0);

