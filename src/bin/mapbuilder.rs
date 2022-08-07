use std::f32::consts::TAU;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_egui::EguiPlugin;
use mapbuilder::{
    self, background::BackgroundConfig, input, map_config::MapConfig, HoverPlanet, Location,
};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(mapbuilder::background::BackgroundPlugin)
        .add_startup_system(setup)
        .add_system(transform_hover_planet)
        .add_system(input::mouse_events)
        .add_system(input::world_move)
        .add_system(input::handle_window_resize)
        .add_system(input::spawn_planet)
        .add_system(input::change_bg_color)
        .add_system(ui::ui_system)
        .add_system(fps)
        .add_system(sync_config);

    app.run();
}

pub struct FPS(u32);

fn fps(time: Res<Time>, mut cur: Local<f32>, mut frames: ResMut<FPS>) {
    *cur += time.delta_seconds();

    if *cur > 0.1 {
        frames.0 = (1. / time.delta_seconds()) as u32;
        *cur = 0.;
    }
}

mod ui {
    use std::f32::consts::TAU;

    use bevy::{prelude::{Res, ResMut} };
    use bevy_egui::{egui, EguiContext};
    use egui::{vec2, Color32, Sense, Stroke, Ui, Vec2, Rounding, Pos2, Rect};
    use mapbuilder::map_config::MapConfig;

    use crate::FPS;

    fn color_to_color(color: Color32) -> bevy::prelude::Color {
        bevy::prelude::Color::rgba_u8(color.r(), color.g(), color.b(), color.a())
    }

    fn color_option(ui: &mut Ui, color: Color32, size: Vec2) -> bool {
        let (response, painter) = ui.allocate_painter(size, Sense::hover());

        let rect = response.rect;

        let stroke = if response.hovered() {
            Stroke::new(2., Color32::WHITE)
        } else {
            Stroke::new(1., Color32::GRAY)
        };

        let rect = rect.shrink(5.); 
        painter.rect_filled(rect, Rounding::none(), color);
        painter.rect_stroke(rect, Rounding::none(), stroke);

        response.clicked()
    }

    pub fn ui_system(mut egui_context: ResMut<EguiContext>, fps: Res<FPS>, config: Res<MapConfig>) {
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .show(egui_context.ctx_mut(), |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(format!("fps {}", fps.0));
                    ui.label(format!("zoom {}", config.zoom));

                    let size = Vec2::splat(32.0);
                    for color in [Color32::RED, Color32::BLUE, Color32::YELLOW] {
                        color_option(ui, color, size);
                    }
                });
            });
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    windows: Res<Windows>,
) {
    let (w, h) = {
        let window = windows.get_primary().unwrap();
        (window.width(), window.height())
    };

    let config = MapConfig::new(w, h);
    commands.insert_resource(config);
    commands.insert_resource(FPS(0));

    let mut color = Color::PURPLE;
    color.set_a(0.4);
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Quad::new(Vec2::new(1., 1.))))
                .into(),
            material: materials.add(ColorMaterial::from(color)),
            ..default()
        })
        .insert_bundle((HoverPlanet, Location { x: 0, y: 0 }));

    let transform = Transform::from_xyz(0.0, 0.0, 1000.0).with_scale(Vec3::new(
        1. / config.zoom,
        1. / config.zoom,
        1.,
    ));
    commands.spawn_bundle(Camera2dBundle {
        transform,
        ..default()
    });
}

fn transform_hover_planet(
    config: Res<MapConfig>,
    mut query: Query<(&Location, &mut Transform), (With<HoverPlanet>, Changed<Location>)>,
) {
    if let Ok((loc, mut transform)) = query.get_single_mut() {
        *transform = config.location_to_transform(loc, 0.1);
    }
}

fn sync_config(config: Res<MapConfig>, mut bg_config: ResMut<BackgroundConfig>) {
    bg_config.height = config.height;
    bg_config.width = config.width;
    bg_config.x = config.x;
    bg_config.y = config.y;
    bg_config.zoom = config.zoom;
}
