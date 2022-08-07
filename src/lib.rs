use bevy::prelude::Component;

pub mod background;
pub mod map_config;
pub mod input;

#[derive(Component, Debug, Default)]
pub struct HoverPlanet;

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Location {
    pub x: i32,
    pub y: i32,
}
