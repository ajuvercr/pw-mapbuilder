use bevy::{
    ecs::system::Resource, prelude::*, sprite::MaterialMesh2dBundle, window::RequestRedraw,
    winit::WinitSettings,
};
use bevy_egui::EguiPlugin;
use egui::Widget;
use mapbuilder::{
    self, input, map_config::MapConfig, CurrentPlayer, HoverPlanet, HoveringUI, Location,
    PlanetData,
};

#[derive(Debug)]
enum ExampleMode {
    Game,
    Application,
    ApplicationWithRedraw,
}

/// Update winit based on the current `ExampleMode`
fn update_winit(
    mode: Res<ExampleMode>,
    mut event: EventWriter<RequestRedraw>,
    mut winit_config: ResMut<WinitSettings>,
) {
    use ExampleMode::*;
    *winit_config = match *mode {
        Game => {
            // In the default `WinitConfig::game()` mode:
            //   * When focused: the event loop runs as fast as possible
            //   * When not focused: the event loop runs as fast as possible
            WinitSettings::game()
        }
        Application => {
            // While in `WinitConfig::desktop_app()` mode:
            //   * When focused: the app will update any time a winit event (e.g. the window is
            //     moved/resized, the mouse moves, a button is pressed, etc.), a [`RequestRedraw`]
            //     event is received, or after 5 seconds if the app has not updated.
            //   * When not focused: the app will update when the window is directly interacted with
            //     (e.g. the mouse hovers over a visible part of the out of focus window), a
            //     [`RequestRedraw`] event is received, or one minute has passed without the app
            //     updating.
            WinitSettings::desktop_app()
        }
        ApplicationWithRedraw => {
            // Sending a `RequestRedraw` event is useful when you want the app to update the next
            // frame regardless of any user input. For example, your application might use
            // `WinitConfig::desktop_app()` to reduce power use, but UI animations need to play even
            // when there are no inputs, so you send redraw requests while the animation is playing.
            event.send(RequestRedraw);
            WinitSettings::desktop_app()
        }
    };
}

fn main() {
    let mut app = App::new();

    app.insert_resource(WinitSettings::desktop_app())
        .insert_resource(ExampleMode::ApplicationWithRedraw)
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_system(ui::ui_system)
        .add_system(ui::ui_editor)
        .add_system(ui::change_planet_color)
        .add_plugin(mapbuilder::background::BackgroundPlugin)
        .add_startup_system(setup)
        .add_system(transform_hover_planet)
        .add_system(input::mouse_events)
        .add_system(input::world_move)
        .add_system(input::handle_window_resize)
        .add_system(input::spawn_planet)
        .add_system(input::change_bg_color)
        .add_system(fps)
        .add_system(update_winit);

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
    use std::{hash::Hash, ops::DerefMut};

    use bevy::{
        prelude::{Assets, Changed, Handle, Query, Res, ResMut, Without},
        sprite::ColorMaterial,
    };
    use bevy_egui::{egui, EguiContext};
    use egui::{Color32, Response, Rounding, Sense, Stroke, Ui, Vec2, Widget, WidgetWithState};
    use mapbuilder::{
        map_config::MapConfig, CurrentPlayer, HoverPlanet, HoveringUI, Location, PlanetData,
    };

    use crate::FPS;

    #[derive(Clone, Copy)]
    pub struct CollapsableState {
        open: bool,
    }

    pub enum CollapseAction {
        Toggle,
        Open,
        Close,
        None,
    }

    pub struct Collapsable<H, U> {
        header: H,
        inner: U,
        id: egui::Id,
    }

    impl<H, U> WidgetWithState for Collapsable<H, U> {
        type State = CollapsableState;
    }

    impl<H, U> Collapsable<H, U> {
        pub fn opened(header: H, inner: U, name: impl Hash) -> Self {
            Self {
                header,
                inner,
                id: egui::Id::new(name),
            }
        }

        pub fn closed(header: H, inner: U, name: impl Hash) -> Self {
            Self {
                header,
                inner,
                id: egui::Id::new(name),
            }
        }
    }

    impl<H, U> Widget for Collapsable<H, U>
    where
        U: Widget,
        H: FnMut(&mut egui::Ui) -> (Response, CollapseAction),
    {
        fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
            let (response, state) = (self.header)(ui);

            let (data, save): (CollapsableState, bool) = match state {
                CollapseAction::Open => (CollapsableState { open: true }, true),
                CollapseAction::Close => (CollapsableState { open: false }, true),
                CollapseAction::Toggle => {
                    let data: Option<CollapsableState> = ui.ctx().data().get_persisted(self.id);
                    (
                        CollapsableState {
                            open: !data.map(|x| x.open).unwrap_or_default(),
                        },
                        true,
                    )
                }
                CollapseAction::None => {
                    let data: Option<CollapsableState> = ui.ctx().data().get_persisted(self.id);
                    (
                        CollapsableState {
                            open: data.map(|x| x.open).unwrap_or_default(),
                        },
                        false,
                    )
                }
            };

            if save {
                ui.ctx().data().insert_persisted(self.id, data);
            }

            if data.open {
                return self.inner.ui(ui) | response;
            }

            response
        }
    }

    fn color_to_color(color: Color32) -> bevy::prelude::Color {
        bevy::prelude::Color::rgba_u8(color.r(), color.g(), color.b(), color.a())
    }

    fn color_option(ui: &mut Ui, color: Color32, size: Vec2, active: bool) -> Response {
        let (response, painter) = ui.allocate_painter(size, Sense::hover().union(Sense::click()));

        let rect = response.rect;

        let stroke = if response.hovered() || active {
            Stroke::new(2., Color32::WHITE)
        } else {
            Stroke::new(1., Color32::GRAY)
        };

        let rect = rect.shrink(5.);
        painter.rect_filled(rect, Rounding::none(), color);
        painter.rect_stroke(rect, Rounding::none(), stroke);

        response
    }

    pub fn ui_editor(
        mut egui_context: ResMut<EguiContext>,
        mut query: Query<(&Location, &mut PlanetData), Without<HoverPlanet>>,
        mut hovering_ui: ResMut<HoveringUI>,
    ) {
        hovering_ui.0 = false;
        let resp = egui::SidePanel::right("right_panel")
            .resizable(true)
            .show(egui_context.ctx_mut(), |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, (l, mut player)) in query.iter_mut().enumerate() {
                        let PlanetData {
                            ref mut player,
                            ref mut name,
                        } = player.deref_mut();

                        ui.horizontal(|ui| {
                            ui.label("planet:");
                            ui.add_sized(ui.available_size(), egui::TextEdit::singleline(name));
                        });

                        ui.label(format!("x: {} y: {}", l.x, l.y));
                        let pn = *player;
                        ui.add(Collapsable::closed(
                            |ui: &mut egui::Ui| {
                                let resp =
                                    color_option(ui, COLORS[pn as usize], Vec2::splat(32.), false);
                                let action = if resp.clicked() {
                                    CollapseAction::Toggle
                                } else {
                                    CollapseAction::None
                                };
                                (resp, action)
                            },
                            move |ui: &mut egui::Ui| {
                                ui.horizontal_wrapped(move |ui| {
                                    let size = Vec2::splat(32.0);
                                    for (i, color) in COLORS.into_iter().enumerate() {
                                        if color_option(ui, color, size, false).clicked() {
                                            *player = i as u32;
                                        }
                                    }
                                })
                                .response
                            },
                            i,
                        ));
                        ui.separator();
                    }
                })
            })
            .response;
        hovering_ui.0 = hovering_ui.0 || resp.hovered();
    }

    const COLORS: [Color32; 7] = [
        Color32::GRAY,
        Color32::RED,
        Color32::BLUE,
        Color32::YELLOW,
        Color32::GOLD,
        Color32::KHAKI,
        Color32::DEBUG_COLOR,
    ];

    pub fn ui_system(
        mut egui_context: ResMut<EguiContext>,
        fps: Res<FPS>,
        config: Res<MapConfig>,
        mut player: ResMut<CurrentPlayer>,
        mut hovering_ui: ResMut<HoveringUI>,
    ) {
        let resp = egui::TopBottomPanel::bottom("bottom_panel")
            .show(egui_context.ctx_mut(), |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(format!("fps {}", fps.0));
                    ui.label(format!("zoom {}", config.zoom));

                    let size = Vec2::splat(32.0);
                    for (i, color) in COLORS.into_iter().enumerate() {
                        if color_option(ui, color, size, i as u32 == player.id).clicked() {
                            player.id = i as u32;
                            player.color = color_to_color(color);
                        }
                    }
                })
            })
            .response;

        hovering_ui.0 = hovering_ui.0 || resp.hovered();
    }

    pub fn change_planet_color(
        planets: Query<(&Handle<ColorMaterial>, &PlanetData), Changed<PlanetData>>,
        mut meshes: ResMut<Assets<ColorMaterial>>,
    ) {
        for (h, d) in planets.into_iter() {
            meshes.set_untracked(
                h,
                ColorMaterial::from(color_to_color(COLORS[d.player as usize])),
            );
        }
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
    commands.insert_resource(HoveringUI(false));
    commands.insert_resource(config);
    commands.insert_resource(FPS(0));
    commands.insert_resource(CurrentPlayer {
        id: 0,
        color: Color::GRAY,
    });

    let mut color = Color::PURPLE;
    color.set_a(0.4);
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: meshes.add(config.mesh()).into(),
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
