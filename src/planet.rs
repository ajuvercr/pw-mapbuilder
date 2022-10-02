use bevy::prelude::DespawnRecursiveExt;
use bevy::{
    prelude::*,
    sprite::{ColorMaterial, MaterialMesh2dBundle},
};
use egui::Color32;
use petname::Petnames;
use serde::{Deserialize, Serialize};

use crate::map_config::MapConfig;
use crate::{utils, eprintit};

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
            .add_startup_system(setup)
            .add_system(align_planet_name)
            .add_system(handle_planet_events)
            .add_system(change_planet_color)
            .add_system(show_text_on_selected);
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<MapConfig>,
) {
    let mut color = Color::PURPLE;
    color.set_a(0.4);
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: config.mesh().into(),
            material: materials.add(ColorMaterial::from(color)),
            ..default()
        })
        .insert_bundle((HoverPlanet, Location { x: 0, y: 0 }));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
pub struct PlanetMesh;
#[derive(Component, Clone, Debug)]
pub struct PlanetName;

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct PlanetData {
    pub player: Player,
    pub name: String,
    pub ship_count: usize,
}

#[derive(Component, Clone, Debug)]
pub struct PlanetEntity {
    pub name: Entity,
    pub mesh: Entity,
}

#[derive(Component, Debug, Default)]
pub struct HoverPlanet;

#[derive(
    Component, Clone, Copy, Serialize, Deserialize, Debug, Default, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Location {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Debug)]
pub struct Selected(pub bool);

pub enum PlanetEvent {
    Create { loc: Location, player: Player },
    CreateNamed { loc: Location, data: PlanetData },
    Delete { id: Entity },
    SetPlayer { id: Entity, player: Player },
    SetName { id: Entity, name: String },
    SetShipCount { id: Entity, amount: usize },
    SetSelected { id: Entity, selected: bool },
}

fn show_text_on_selected(
    planets: Query<(&PlanetEntity, &Selected), Changed<Selected>>,
    mut visibles: Query<&mut Visibility>,
) {
    for (p, s) in planets.iter() {
        let mut vis = visibles.get_mut(p.name).unwrap();
        vis.is_visible = s.0;
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_planet_events(
    mut event_reader: EventReader<PlanetEvent>,
    mut planets: Query<(&mut PlanetData, &mut Selected)>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    names: Res<Petnames<'static>>,
    mut rng: ResMut<utils::rng::RNG>,
    config: Res<MapConfig>,
) {
    for event in event_reader.iter() {
        match event {
            PlanetEvent::Create { loc, player } => {
                let data = PlanetData {
                    player: *player,
                    ship_count: 10,
                    name: names.generate(rng.as_mut(), 2, " "),
                };

                spawn_named_planet(&config, &mut commands, data, *loc, &mut materials);
            }
            PlanetEvent::CreateNamed { data, loc } => {
                spawn_named_planet(&config, &mut commands, data.clone(), *loc, &mut materials);
            }
            PlanetEvent::Delete { id } => {
                commands.entity(*id).despawn_recursive();
            }
            PlanetEvent::SetPlayer { id, player } => {
                if let Ok((mut data, _)) = planets.get_mut(*id) {
                    data.player = *player;
                }
            }
            PlanetEvent::SetName { id, name } => {
                if let Ok((mut data, _)) = planets.get_mut(*id) {
                    data.name = name.clone();
                }
            }
            PlanetEvent::SetShipCount { id, amount } => {
                if let Ok((mut data, _)) = planets.get_mut(*id) {
                    data.ship_count = *amount;
                }
            }
            PlanetEvent::SetSelected { id, selected } => {
                if let Ok((_, mut s)) = planets.get_mut(*id) {
                    s.0 = *selected;
                }
            }
        }
    }
}

fn spawn_named_planet(
    config: &Res<MapConfig>,
    commands: &mut Commands,
    data: PlanetData,
    loc: Location,

    materials: &mut Assets<ColorMaterial>,
) {
    let color = data.player.color();
    let transform = config.text_transform(&loc);
    let name = commands
        .spawn_bundle(Text2dBundle {
            text: Text::from_section(
                data.name.clone(),
                TextStyle {
                    font: config.font.clone_weak(),
                    font_size: 30.0,
                    color: Color::WHITE,
                },
            )
            .with_alignment(TextAlignment::CENTER),
            transform,
            visibility: Visibility { is_visible: false },
            ..default()
        })
        .insert(PlanetName)
        .id();

    let transform = config.shape_transform(&loc, 0.5);
    let mesh = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: config.mesh().into(),
            material: materials.add(ColorMaterial::from(color)),
            transform,
            ..default()
        })
        .insert(PlanetMesh)
        .id();

    commands
        .spawn()
        .insert(Visibility::default())
        .insert(ComputedVisibility::default())
        .insert(GlobalTransform::default())
        .insert(Transform::default())
        .insert(loc)
        .insert(data)
        .insert(Selected(false))
        .add_child(name)
        .add_child(mesh)
        .insert(PlanetEntity { name, mesh });
}

fn change_planet_color(
    planets: Query<(&PlanetData, &PlanetEntity), Changed<PlanetData>>,
    meshes_query: Query<&Handle<ColorMaterial>, With<PlanetMesh>>,
    mut meshes: ResMut<Assets<ColorMaterial>>,
) {
    for (d, e) in planets.into_iter() {
        if let Ok(h) = meshes_query.get(e.mesh) {
            meshes.set_untracked(h, ColorMaterial::from(d.player.color()));
        }
    }
}

fn align_planet_name(
    mut query: Query<(&PlanetData, &PlanetEntity), Changed<PlanetData>>,
    mut texts: Query<&mut Text, With<PlanetName>>,
) {
    for (d, e) in query.iter_mut() {
        if let Ok(mut t) = texts.get_mut(e.name) {
            t.sections[0].value = d.name.clone();
        } else {
            eprintit!("No such entity found!");
        }
    }
}
