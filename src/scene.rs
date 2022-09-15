use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    map_config::{MapConfig, MapEvent, MapType},
    planet::{Location, PlanetData, PlanetEvent},
};

pub enum SceneEvent {
    Save(PathBuf),
    Load(PathBuf),
    Export,
}

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        println!("Here");
        app.add_event::<SceneEvent>()
            .add_system(handle_scene_events);
    }
}

#[derive(Serialize, Deserialize)]
struct ScenePlanet {
    data: PlanetData,
    location: Location,
}
#[derive(Serialize, Deserialize)]
struct SceneConfig {
    ty: MapType,
}

#[derive(Serialize, Deserialize)]
struct Scene {
    config: SceneConfig,
    planets: Vec<ScenePlanet>,
}

mod io {
    use std::{
        fs::File,
        io::{Read, Write},
        path::PathBuf,
    };

    use serde::Deserialize;

    pub fn write_to_file(contents: &[u8], location: &PathBuf) -> Result<(), std::io::Error> {
        let mut file = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(location)?;
        file.write_all(contents)?;
        Ok(())
    }

    pub fn read_from_file<T: for<'a> Deserialize<'a>>(
        location: &PathBuf,
    ) -> Result<T, std::io::Error> {
        let mut file = File::open(location)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        match serde_json::from_slice(&buf) {
            Ok(x) => Ok(x),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        }
    }
}

fn handle_scene_events(
    mut commands: Commands,
    planets: Query<(&PlanetData, &Location, Entity)>,
    current_config: Res<MapConfig>,
    mut events: EventReader<SceneEvent>,

    mut map_events: EventWriter<MapEvent>,
    mut planet_events: EventWriter<PlanetEvent>,
) {
    for event in events.iter() {
        match event {
            SceneEvent::Save(location) => {
                let planets = planets
                    .iter()
                    .map(|(x, y, _)| ScenePlanet {
                        data: x.clone(),
                        location: *y,
                    })
                    .collect();
                let scene_config = SceneConfig {
                    ty: current_config.ty,
                };

                let scene = Scene {
                    planets,
                    config: scene_config,
                };
                let data = serde_json::to_vec_pretty(&scene).unwrap();
                if let Err(e) = io::write_to_file(&data, location) {
                    eprintln!("Error {:?}", e);
                }
            }
            SceneEvent::Load(location) => {
                planets
                    .iter()
                    .map(|(_, _, e)| e)
                    .for_each(|e| commands.entity(e).despawn_recursive());

                let Scene { planets, config } = match io::read_from_file(location) {
                    Ok(x) => x,
                    Err(e) => {
                        eprintln!("Error {:?}", e);
                        return;
                    }
                };

                map_events.send(MapEvent::SetType(config.ty));
                planet_events.send_batch(planets.into_iter().map(
                    |ScenePlanet { data, location }| PlanetEvent::CreateNamed {
                        loc: location,
                        data,
                    },
                ));
            }
            SceneEvent::Export => {
                #[derive(Serialize)]
                struct Planet<'a> {
                    name: &'a str,
                    x: f32,
                    y: f32,
                    owner: Option<usize>,
                    ship_count: usize,
                }

                let planets: Vec<_> = planets
                    .iter()
                    .map(|(data, loc, _)| Planet {
                        name: &data.name,
                        x: loc.x as f32,
                        y: loc.y as f32,
                        owner: Some(data.player.0),
                        ship_count: data.ship_count,
                    })
                    .collect();

                println!("{}", serde_json::json!({ "planets": planets }));
            }
        }
    }
}
