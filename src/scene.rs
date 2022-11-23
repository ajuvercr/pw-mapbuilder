use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    eprintit,
    map_config::{MapConfig, MapEvent, MapType},
    planet::{Location, PlanetData, PlanetEvent},
};

pub enum SceneEvent {
    Save,
    Load,
    LoadCont(String),
    Export {
        girth: f32,
        name: String,
    },
    Upload {
        girth: f32,
        name: String,
        url: String,
    },
}

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SceneEvent>()
            .add_plugin(io::IOPlugin)
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

fn get_planets_export(
    dist: f32,
    planets: &Query<(&PlanetData, &Location, Entity)>,
    current_config: &MapConfig,
    name: &str,
) -> Value {
    #[derive(Serialize)]
    struct Planet<'a> {
        name: &'a str,
        x: f32,
        y: f32,
        owner: Option<usize>,
        ship_count: usize,
    }

    let mut longest_dist = 0.0;
    for (_, l1, _) in planets {
        let t1 = current_config.shape_transform(l1, 0.);
        for (_, l2, _) in planets {
            let t2 = current_config.shape_transform(l2, 0.);
            let d = (t1.translation.x - t2.translation.x).powi(2)
                + (t1.translation.y - t2.translation.y).powi(2);
            if d > longest_dist {
                longest_dist = d;
            }
        }
    }
    longest_dist = longest_dist.sqrt();

    let scale = dist / longest_dist;

    let planets: Vec<_> = planets
        .iter()
        .map(|(data, loc, _)| {
            let t1 = current_config.shape_transform(loc, 0.);
            let x = t1.translation.x;
            let y = t1.translation.y;

            Planet {
                name: &data.name,
                x: x * scale,
                y: y * scale,
                owner: Some(data.player.0),
                ship_count: data.ship_count,
            }
        })
        .collect();

    serde_json::json!({ "planets": planets, "name": name })
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
            SceneEvent::Save => {
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
                let data = serde_json::to_string_pretty(&scene).unwrap();

                io::save(data);
            }
            SceneEvent::Export { girth, name } => {
                let content =
                    get_planets_export(*girth, &planets, &current_config, &name).to_string();
                io::export(content);
            }
            SceneEvent::Load => {
                if let Some(data) = io::load() {
                    load_cont(
                        &data,
                        &planets,
                        &mut commands,
                        &mut map_events,
                        &mut planet_events,
                    );
                }
            }
            SceneEvent::LoadCont(data) => load_cont(
                data,
                &planets,
                &mut commands,
                &mut map_events,
                &mut planet_events,
            ),
            SceneEvent::Upload { girth, url, name } => {
                let content =
                    get_planets_export(*girth, &planets, &current_config, &name).to_string();
                io::upload(url, content);
            }
        }
    }
}

fn load_cont(
    data: &str,
    planets: &Query<(&PlanetData, &Location, Entity)>,
    commands: &mut Commands,
    map_events: &mut EventWriter<MapEvent>,
    planet_events: &mut EventWriter<PlanetEvent>,
) {
    let Scene {
        planets: p2,
        config,
    } = match serde_json::from_str(data) {
        Ok(x) => x,
        Err(e) => {
            eprintit!("Error: {}", e);
            return;
        }
    };

    planets
        .iter()
        .map(|(_, _, e)| e)
        .for_each(|e| commands.entity(e).despawn_recursive());
    map_events.send(MapEvent::SetType(config.ty));
    planet_events.send_batch(p2.into_iter().map(|ScenePlanet { data, location }| {
        PlanetEvent::CreateNamed {
            loc: location,
            data,
        }
    }));
}

#[cfg(not(target_family = "wasm"))]
mod io {
    use bevy::{prelude::Plugin, tasks::IoTaskPool};
    use rfd::{AsyncFileDialog, FileDialog};

    use std::{
        fs::File,
        io::{Read, Write},
        path::Path,
    };

    pub struct IOPlugin;
    impl Plugin for IOPlugin {
        fn build(&self, _app: &mut bevy::prelude::App) {}
    }

    pub fn write_to_file(contents: &[u8], location: &Path) -> Result<(), std::io::Error> {
        let mut file = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(location)?;
        file.write_all(contents)?;
        Ok(())
    }

    pub fn read_from_file(location: &Path) -> Result<String, std::io::Error> {
        let mut file = File::open(location)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;

        Ok(buf)
    }

    pub fn save(content: String) {
        let thread_pool = IoTaskPool::get();
        thread_pool
            .spawn(async move {
                if let Some(path) = AsyncFileDialog::new().save_file().await {
                    write_to_file(content.as_bytes(), path.path()).unwrap();
                }
            })
            .detach();
    }

    pub fn load() -> Option<String> {
        if let Some(path) = FileDialog::new().pick_file() {
            read_from_file(&path).ok()
        } else {
            None
        }
    }

    pub fn export(content: String) {
        let thread_pool = IoTaskPool::get();
        thread_pool
            .spawn(async move {
                if let Some(path) = AsyncFileDialog::new().save_file().await {
                    write_to_file(content.as_bytes(), path.path()).unwrap();
                }
            })
            .detach();
    }

    pub fn upload(url: &str, content: String) {
        todo!()
    }
}

#[cfg(target_family = "wasm")]
mod io {
    use std::sync::Mutex;

    use bevy::prelude::{EventWriter, Plugin};
    use wasm_bindgen::prelude::wasm_bindgen;

    use super::SceneEvent;

    mod js {
        use wasm_bindgen::prelude::wasm_bindgen;

        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_namespace = console)]
            pub fn log(s: &str);

            #[wasm_bindgen(js_namespace = ["window", "scene"])]
            pub fn save(s: &str);

            #[wasm_bindgen(js_namespace = ["window", "scene"])]
            pub fn load();

            #[wasm_bindgen(js_namespace = ["window", "scene"])]
            pub fn exp(s: &str);

            #[wasm_bindgen(js_namespace = ["window", "scene"])]
            pub fn upload(url: &str, content: &str);
        }
    }

    pub struct IOPlugin;
    impl Plugin for IOPlugin {
        fn build(&self, app: &mut bevy::prelude::App) {
            app.add_system(complete_load);
        }
    }

    fn complete_load(mut ev: EventWriter<SceneEvent>) {
        if let Ok(mut x) = LOAD.lock() {
            if let Some(st) = x.take() {
                ev.send(SceneEvent::LoadCont(st));
            }
        }
    }

    pub fn save(content: String) {
        js::save(&content);
    }

    static LOAD: Mutex<Option<String>> = Mutex::new(None);

    pub fn load() -> Option<String> {
        js::load();
        None
    }

    #[wasm_bindgen]
    pub fn finish_load(st: &str) {
        if let Ok(mut x) = LOAD.lock() {
            *x = Some(st.to_string());
        }
    }

    pub fn export(content: String) {
        js::exp(&content);
    }

    pub fn upload(url: &str, content: String) {
        js::upload(url, &content);
    }
}
