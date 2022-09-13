use bevy::prelude::DespawnRecursiveExt;
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
            .add_startup_system(setup)
            .add_system(align_planet_name)
            .add_system(handle_planet_events)
            .add_system(change_planet_color)
            .add_system(show_text_on_selected);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<MapConfig>,
) {
    let mut color = Color::PURPLE;
    color.set_a(0.4);
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: meshes.add(config.mesh()).into(),
            material: materials.add(ColorMaterial::from(color)),
            ..default()
        })
        .insert_bundle((HoverPlanet, Location { x: 0, y: 0 }));
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
pub struct PlanetMesh;
#[derive(Component, Clone, Debug)]
pub struct PlanetName;

#[derive(Component, Clone, Debug)]
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

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Debug)]
pub struct Selected(pub bool);

pub enum PlanetEvent {
    Create { loc: Location, player: Player },
    Delete { id: Entity },
    SetPlayer { id: Entity, player: Player },
    SetName { id: Entity, name: String },
    SetShipCount { id: Entity, amount: usize },
    SetSelected { id: Entity, selected: bool },
}

fn change_planet_color(
    planets: Query<(&Handle<ColorMaterial>, &PlanetData), Changed<PlanetData>>,
    mut meshes: ResMut<Assets<ColorMaterial>>,
) {
    for (h, d) in planets.into_iter() {
        meshes.set_untracked(h, ColorMaterial::from(d.player.color()));
    }
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    generator: Res<RNG>,
    config: Res<MapConfig>,
    asset_server: Res<AssetServer>,
) {
    for event in event_reader.iter() {
        match event {
            PlanetEvent::Create { loc, player } => {
                spawn_planet(
                    &config,
                    loc,
                    player,
                    &generator,
                    &mut commands,
                    &asset_server,
                    &mut meshes,
                    &mut materials,
                );
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

#[allow(clippy::too_many_arguments)]
fn spawn_planet(
    config: &Res<MapConfig>,
    loc: &Location,
    player: &Player,
    generator: &Res<RNG>,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let data = PlanetData {
        player: *player,
        ship_count: 10,
        name: generator.generate_name(),
    };
    let color = player.color();
    let name = commands
        .spawn_bundle(Text2dBundle {
            text: Text::from_section(
                data.name.clone(),
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 50.0,
                    color: Color::WHITE,
                },
            )
            .with_alignment(TextAlignment::CENTER),
            transform: Transform::from_scale(Vec3 {
                x: 1. / config.zoom,
                y: 1. / config.zoom,
                z: 1.,
            })
            .with_translation(Vec3 {
                x: 0.,
                y: -0.7,
                z: 0.5,
            }),
            visibility: Visibility { is_visible: false },
            ..default()
        })
        .insert(PlanetName)
        .id();

    let transform = config.location_to_delta(loc);
    let mesh = commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: meshes.add(config.mesh()).into(),
            material: materials.add(ColorMaterial::from(color)),
            transform,
            ..default()
        })
        .insert(PlanetMesh)
        .id();

    let transform = config.location_to_transform(loc, 0.5);
    commands
        .spawn()
        .insert(Visibility::default())
        .insert(ComputedVisibility::default())
        .insert(GlobalTransform::default())
        .insert(transform)
        .insert(*loc)
        .insert(data)
        .insert(Selected(false))
        .add_child(name)
        .add_child(mesh)
        .insert(PlanetEntity { name, mesh });
}

fn align_planet_name(
    mut query: Query<(&PlanetData, &PlanetEntity), Changed<PlanetData>>,
    mut texts: Query<&mut Text, With<PlanetName>>,
) {
    for (d, e) in query.iter_mut() {
        if let Ok(mut t) = texts.get_mut(e.name) {
            t.sections[0].value = d.name.clone();
        } else {
            eprintln!("No such entity found!");
        }
    }
}
