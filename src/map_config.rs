use bevy::{
    math::{Vec2, Vec3},
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
    sprite::Mesh2dHandle,
};

use crate::planet::{HoverPlanet, Location, PlanetEntity, PlanetMesh};

use serde::{Deserialize, Serialize};

pub enum MapEvent {
    SetColor(Color),
    SetType(MapType),
}

pub struct MapConfigPlugin;
impl Plugin for MapConfigPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, setup_config)
            .add_event::<MapEvent>()
            .add_system(handle_map_events);
    }
}

fn setup_config(
    mut commands: Commands,
    windows: Res<Windows>,
    asset: Res<AssetServer>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
) {
    let (w, h) = {
        let window = windows.get_primary().unwrap();
        (window.width(), window.height())
    };
    let config = MapConfig::new(w, h, &asset, &mut mesh_assets);
    commands.insert_resource(config);
}

fn handle_map_events(
    mut reader: EventReader<MapEvent>,
    mut config: ResMut<MapConfig>,

    mut hover_planet: Query<(&mut Transform, &mut Mesh2dHandle, &Location), With<HoverPlanet>>,

    mut locations: Query<
        (&PlanetEntity, &mut Transform, &Location),
        (Without<HoverPlanet>, Without<PlanetMesh>),
    >,
    mut meshes: Query<
        (&mut Transform, &mut Mesh2dHandle),
        (Without<HoverPlanet>, With<PlanetMesh>),
    >,
) {
    let mut update_meshes = false;
    for event in reader.iter() {
        match event {
            MapEvent::SetType(ty) => {
                config.ty = *ty;
                update_meshes = true;
            }
            MapEvent::SetColor(color) => {
                config.bg_color = *color;
            }
        }
    }

    if update_meshes {
        let mesh_handle: Mesh2dHandle = config.mesh().into();

        for (e, mut t, loc) in locations.iter_mut() {
            // Update hover planet mesh
            *t = config.location_to_transform(loc, 0.);

            let (mut t, mut l) = meshes.get_mut(e.mesh).unwrap();

            *t = config.location_to_delta(loc);
            *l = mesh_handle.clone();
        }

        for (mut t, mut l, loc) in hover_planet.iter_mut() {
            *l = mesh_handle.clone();

            *t = config
                .location_to_transform(loc, 0.1)
                .mul_transform(config.location_to_delta(loc));
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum MapType {
    Squares,
    Triangles,
}

const TRIAG_HEIGHT: f32 = 0.866_025_4; // sqrt(1 - 0.25) height of equal triangle

#[derive(Clone, Debug)]
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

    pub font: Handle<Font>,

    meshes: Vec<(MapType, Handle<Mesh>)>,
}

impl MapConfig {
    pub fn new(
        width: f32,
        height: f32,
        asset_server: &AssetServer,
        mesh_assets: &mut Assets<Mesh>,
    ) -> Self {
        let meshes = [MapType::Squares, MapType::Triangles]
            .into_iter()
            .map(|x| (x, MapConfig::mesh_asset(x, mesh_assets)))
            .collect();
        let font = asset_server.load("fonts/FiraSans-Bold.ttf");
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
            meshes,
            font,
        }
    }

    pub fn mesh(&self) -> Handle<Mesh> {
        self.meshes
            .iter()
            .find(|x| x.0 == self.ty)
            .map(|x| x.1.clone_weak())
            .unwrap()
    }

    fn mesh_asset(ty: MapType, mesh_assets: &mut Assets<Mesh>) -> Handle<Mesh> {
        let mesh = match ty {
            MapType::Squares => Mesh::from(shape::Quad::new(Vec2::new(1., 1.))),
            MapType::Triangles => {
                let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
                let th2 = TRIAG_HEIGHT * 0.5;
                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    // vec![
                    //     [-0.5, -TRIAG_HEIGHT * 0.5, 0.0],
                    //     [0.5, -TRIAG_HEIGHT * 0.5, 0.0],
                    //     [0., TRIAG_HEIGHT * 0.5, 0.0],
                    // ],
                    vec![[-0.5, -th2, 0.0], [0.5, -th2, 0.0], [0., th2, 0.0]],
                    // vec![[-0.5, 0., 0.0], [0.5, 0., 0.0], [0., TRIAG_HEIGHT, 0.0]],
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
        };

        mesh_assets.add(mesh)
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
                };
                Some(out)
            }
            MapType::Triangles => {
                let mut x = x * 2.0 + 1.;
                let y = y + TRIAG_HEIGHT * 0.5;
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
                })
            }
        }
    }

    pub fn location_to_delta(&self, location: &Location) -> Transform {
        match self.ty {
            MapType::Squares => Transform::default(),
            MapType::Triangles => {
                if location.x % 2 == 1 || location.x % 2 == -1 {
                    Transform::from_matrix(Mat4::from_rotation_z(std::f32::consts::PI))
                } else {
                    Transform::default()
                }
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
