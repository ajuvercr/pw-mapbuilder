use bevy::{prelude::Transform, math::Vec3};

use crate::Location;


#[derive(Clone, Copy, Debug)]
pub struct MapConfig {
    pub zoom: f32,

    pub width: f32,
    pub height: f32,

    pub x: f32,
    pub y: f32,

    pub mouse_x: Option<f32>,
    pub mouse_y: Option<f32>,
}

impl MapConfig {
    pub fn new(width: f32, height: f32) -> Self {
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

        self.zoom = zoom.max(20.);

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
            player: None,
        };
        Some(out)
    }

    pub fn location_to_transform(&self, location: &Location, z: f32) -> Transform {
        Transform::default().with_translation(Vec3::new(location.x as f32, location.y as f32, z))
    }
}
