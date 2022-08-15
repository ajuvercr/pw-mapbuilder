use bevy::{
    math::{Vec2, Vec3},
    prelude::{shape, Color, Mat4, Mesh, Transform},
    render::mesh::{Indices, PrimitiveTopology},
};

use crate::Location;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

    pub bg_color: Color,
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

            bg_color: Color::GRAY,
        }
    }

    pub fn mesh(&self) -> Mesh {
        match self.ty {
            MapType::Squares => Mesh::from(shape::Quad::new(Vec2::new(1., 1.))),
            MapType::Triangles => {
                let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    // vec![
                    //     [-0.5, -TRIAG_HEIGHT * 0.5, 0.0],
                    //     [0.5, -TRIAG_HEIGHT * 0.5, 0.0],
                    //     [0., TRIAG_HEIGHT * 0.5, 0.0],
                    // ],
                    vec![[-0.5, 0., 0.0], [0.5, 0., 0.0], [0., TRIAG_HEIGHT, 0.0]],
                );
                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_NORMAL,
                    vec![[0., 0., 1.], [0., 0., 1.], [0., 0., 1.]],
                );
                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_UV_0,
                    vec![[0.0, 0.0], [1.0, 0.0], [0.5, TRIAG_HEIGHT]],
                );
                mesh.set_indices(Some(Indices::U32(vec![0, 1, 2])));
                mesh
            }
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
                let mut x = x * 2.0 + 1.;
                let y = y;
                let p = 1.154_700_5; // tan(pi / 6) * 2    //  30 degrees
                let row = (y / TRIAG_HEIGHT).floor();
                let mut frac = y - row * TRIAG_HEIGHT;
                if frac < 0. {
                    frac += 1.0;
                }

                if row as i32 % 2 == 0 {
                    x += 1.0;
                }

                let mut col_frac = x.fract();
                if col_frac < 0. {
                    col_frac += 1.0;
                }
                let mut col = x.floor() as i32;
                let mut triangle_bot_length = 1. - p * frac;
                if col % 2 == 0 {
                    triangle_bot_length = 1.0 - triangle_bot_length;
                }

                if col_frac < triangle_bot_length {
                    col -= 1;
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
                let mut mat = Mat4::IDENTITY;

                if location.x % 2 == 1 || location.x % 2 == -1 {
                    mat = Mat4::from_translation(Vec3::new(0., TRIAG_HEIGHT * 0.5, 0.))
                        * Mat4::from_rotation_z(std::f32::consts::PI)
                        * Mat4::from_translation(Vec3::new(0., TRIAG_HEIGHT * -0.5, 0.))
                        * mat;
                }

                let dx = if location.y % 2 == 0 { -1. } else { 0. };
                mat = Mat4::from_translation(Vec3::new(
                    (location.x as f32 + dx) * 0.5,
                    location.y as f32 * TRIAG_HEIGHT,
                    z,
                )) * mat;

                Transform::from_matrix(mat)
            }
        }
    }
}
