use bevy::{
    prelude::*,
    sprite::{ColorMaterial, MaterialMesh2dBundle},
};
use egui::Color32;
use rnglib::RNG;

use crate::map_config::MapConfig;

pub const COLORS: [Color32; 7] = [
    Color32::GRAY,
    Color32::RED,
    Color32::BLUE,
    Color32::YELLOW,
    Color32::GOLD,
    Color32::KHAKI,
    Color32::DEBUG_COLOR,
];

pub struct PlanetPlugin;
impl Plugin for PlanetPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Player(0))
            .add_event::<PlanetEvent>()
            .add_system(handle_planet_events)
            .add_system(change_planet_color);
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Player(pub usize);

impl Player {
    pub fn color(&self) -> Color {
        let color = self.color32();
        Color::rgba_u8(color.r(), color.g(), color.b(), color.a())
    }

    pub fn color32(&self) -> Color32 {
        COLORS[self.0]
    }
}


#[derive(Component, Clone, Debug)]
pub struct PlanetData {
    pub player: Player,
    pub name: String,
}

#[derive(Component, Debug, Default)]
pub struct HoverPlanet;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Debug)]
pub struct Selected;

pub enum PlanetEvent {
    Create { loc: Location, player: Player },
    Delete { id: Entity },
    SetPlayer { id: Entity, player: Player },
    SetName { id: Entity, name: String },
}

use bevy::prelude::DespawnRecursiveExt;

fn change_planet_color(
    planets: Query<(&Handle<ColorMaterial>, &PlanetData), Changed<PlanetData>>,
    mut meshes: ResMut<Assets<ColorMaterial>>,
) {
    for (h, d) in planets.into_iter() {
        meshes.set_untracked(h, ColorMaterial::from(d.player.color()));
    }
}

fn handle_planet_events(
    mut event_reader: EventReader<PlanetEvent>,
    mut planets: Query<&mut PlanetData>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    generator: Res<RNG>,
    config: Res<MapConfig>,
) {
    for event in event_reader.iter() {
        match event {
            PlanetEvent::Create { loc, player } => {
                let transform = config.location_to_transform(loc, 0.);
                let data = PlanetData {
                    player: *player,
                    name: generator.generate_name(),
                };
                let color = player.color();

                commands
                    .spawn_bundle(MaterialMesh2dBundle {
                        mesh: meshes.add(config.mesh()).into(),
                        material: materials.add(ColorMaterial::from(color)),
                        transform,
                        ..default()
                    })
                    .insert(*loc)
                    .insert(data);
            }
            PlanetEvent::Delete { id } => {
                commands.entity(*id).despawn_recursive();
            }
            PlanetEvent::SetPlayer { id, player } => {
                if let Ok(mut data) = planets.get_mut(*id) {
                    data.player = *player;
                }
            }
            PlanetEvent::SetName { id, name } => {
                if let Ok(mut data) = planets.get_mut(*id) {
                    data.name = name.clone();
                }
            }
        }
    }
}
