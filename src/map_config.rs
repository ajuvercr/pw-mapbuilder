use bevy::{
    math::{Quat, Vec3},
    prelude::Transform,
};

use crate::Location;

#[derive(Clone, Copy, Debug)]
pub enum MapType {
    Squares,
    Triangles,
}

const TRIAG_HEIGHT: f32 = 0.866_025_4; // sqrt(1 - 0.25) height of equal triangle

#[derive(Clone, Copy, Debug)]
pub struct MapConfig {
    pub ty: MapType,

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
            ty: MapType::Triangles,
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
        match self.ty {
            MapType::Squares => {
                let dx = x.signum() * 0.5;
                let dy = y.signum() * 0.5;
                let out = Location {
                    x: (x + dx) as i32,
                    y: (y + dy) as i32,
                    player: None,
                };
                Some(out)
            }
            MapType::Triangles => {
                let p = 1.154_700_5; // tan(pi / 6) * 2    //  30 degrees
                let row = (y / TRIAG_HEIGHT).floor();
                let frac = y - row * TRIAG_HEIGHT;

                let triangle_bot_length = p * frac;

                let col_frac = x.fract();
                let mut col = x.floor();

                if col_frac < triangle_bot_length {
                    col -= 1.;
                }

                Some(Location {
                    x: col as i32,
                    y: row as i32,
                    player: None,
                })
            }
        }
    }

    pub fn location_to_transform(&self, location: &Location, z: f32) -> Transform {
        match self.ty {
            MapType::Squares => Transform::default().with_translation(Vec3::new(
                location.x as f32,
                location.y as f32,
                z,
            )),
            MapType::Triangles => {
                let rot = if location.x % 2 == 0 {
                    Quat::from_rotation_y(std::f32::consts::PI)
                } else {
                    Quat::default()
                };
                Transform::default()
                    .with_translation(Vec3::new(
                        location.x as f32,
                        location.y as f32 * TRIAG_HEIGHT,
                        z,
                    ))
                    .with_rotation(rot)
            }
        }
    }
}
