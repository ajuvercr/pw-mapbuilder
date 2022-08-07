use bevy::prelude::{Color, Component};

pub mod background;
pub mod input;
pub mod map_config;

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

    pub player: Option<u32>,
}
