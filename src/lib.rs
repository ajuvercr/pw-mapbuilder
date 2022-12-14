use bevy::{
    prelude::{Local, Plugin, Res, ResMut},
    time::Time,
};
use egui::Color32;
use petname::Petnames;

pub mod background;
pub mod input;
pub mod map_config;
pub mod planet;
pub mod scene;
pub mod ui;
pub mod utils;

pub struct HoveringUI(pub bool);

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
        app.insert_resource(utils::rng::new());
        app.insert_resource(Petnames::small());
        app.insert_resource(FPS(0));
        app.insert_resource(HoveringUI(false));
        app.add_system(fps);
    }
}

pub const ZEUS: Color32 = Color32::from_rgb(255, 128, 0);
