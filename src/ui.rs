use crate::{
    map_config::{MapConfig, MapEvent, MapType},
    planet::{HoverPlanet, Location, PlanetData, PlanetEvent, Player, Selected, COLORS},
    scene::SceneEvent,
    HoveringUI, ZEUS,
};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use egui::{
    pos2, Color32, Rect, Response, RichText, Rounding, Sense, Shape, Stroke, TextureId, Ui, Vec2,
    Widget, WidgetWithState,
};
// use rfd::FileDialog;
use std::{
    hash::Hash,
    ops::DerefMut,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
};

use crate::FPS;

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(ui_editor.before(ui_system))
            .add_system(ui_system)
            .init_resource::<Icons>()
            .add_startup_system(load_images)
            .add_startup_system(set_font_sizes)
            .insert_resource(HoveringUI(false));
    }
}

#[derive(Default)]
struct Icons {
    handles: Vec<Handle<Image>>,
    squares: TextureId,
    triangles: TextureId,
    hexagons: TextureId,
    octagons: TextureId,
}

fn set_font_sizes(mut egui_context: ResMut<EguiContext>) {
    use egui::FontFamily::Proportional;
    use egui::FontId;
    use egui::TextStyle::*;

    let ctx = egui_context.ctx_mut();
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (Heading, FontId::new(24.0, Proportional)),
        (Body, FontId::new(16.0, Proportional)),
        (Monospace, FontId::new(16.0, Proportional)),
        (Button, FontId::new(16.0, Proportional)),
        (Small, FontId::new(12.0, Proportional)),
    ]
    .into();
    ctx.set_style(style);
}

fn load_images(
    asset_server: Res<AssetServer>,
    mut icons: ResMut<Icons>,
    mut ctx: ResMut<EguiContext>,
) {
    let sq = asset_server.load("icons/square.png");
    icons.squares = ctx.add_image(sq.clone_weak());
    icons.handles.push(sq);
    let tri = asset_server.load("icons/triangle.png");
    icons.triangles = ctx.add_image(tri.clone_weak());
    icons.handles.push(tri);

    let tri = asset_server.load("icons/hexagon.png");
    icons.hexagons = ctx.add_image(tri.clone_weak());
    icons.handles.push(tri);

    let tri = asset_server.load("icons/octagon.png");
    icons.octagons = ctx.add_image(tri.clone_weak());
    icons.handles.push(tri);
}

#[derive(Clone, Copy)]
pub struct CollapsableState {
    open: bool,
}

pub struct Collapsable<H, U, S> {
    header: H,
    content: U,
    id: egui::Id,
    state: S,
}

impl<H, U, S> WidgetWithState for Collapsable<H, U, S> {
    type State = CollapsableState;
}

impl<H, U, S> Collapsable<H, U, S> {
    pub fn opened(header: H, content: U, state: S, name: impl Hash) -> Self {
        Self {
            header,
            content,
            state,
            id: egui::Id::new(name),
        }
    }

    pub fn closed(header: H, content: U, state: S, name: impl Hash) -> Self {
        Self {
            header,
            content,
            state,
            id: egui::Id::new(name),
        }
    }
}

impl<H, U, S> Widget for Collapsable<H, U, S>
where
    U: FnOnce(&mut egui::Ui, &mut bool, &mut S) -> Response,
    H: FnOnce(&mut egui::Ui, &mut bool, &mut S) -> Response,
{
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let state = ui.ctx().data().get_persisted(self.id);
        let mut state_mut = state.map(|x: CollapsableState| x.open).unwrap_or_default();
        let mut response = (self.header)(ui, &mut state_mut, &mut self.state);

        if state_mut {
            response |= (self.content)(ui, &mut state_mut, &mut self.state);
        }

        if state_mut != state.map(|x: CollapsableState| x.open).unwrap_or_default() {
            let data = CollapsableState { open: state_mut };
            ui.ctx().data().insert_persisted(self.id, data);
        }

        response
    }
}

struct PlanetWidget<'a, 'w, 's> {
    i: usize,
    data: &'a PlanetData,
    loc: &'a Location,
    entity: Entity,
    events: &'a mut EventWriter<'w, 's, PlanetEvent>,
}

impl<'a, 'w, 's> Widget for PlanetWidget<'a, 'w, 's> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let PlanetData {
            ref player,
            ref name,
            ref ship_count,
        } = self.data;

        let mut name = name.clone();
        ui.horizontal(|ui| {
            ui.label("planet:");
            let resp = ui.add_sized(ui.available_size(), egui::TextEdit::singleline(&mut name));
            if resp.changed() {
                self.events.send(PlanetEvent::SetName {
                    id: self.entity,
                    name,
                });
            }
        });

        let mut ship_count = ship_count.to_string();
        ui.horizontal(|ui| {
            ui.label("ship count:");
            let resp = ui.add_sized(
                ui.available_size(),
                egui::TextEdit::singleline(&mut ship_count),
            );
            if resp.changed() {
                if let Ok(amount) = ship_count.parse() {
                    self.events.send(PlanetEvent::SetShipCount {
                        id: self.entity,
                        amount,
                    });
                } else {
                    let (response, painter) = ui.allocate_painter(
                        Vec2::splat(64.),
                        Sense {
                            drag: false,
                            click: false,
                            focusable: false,
                        },
                    );

                    let rect = response.rect;

                    let rect = rect.shrink(5.);
                    painter.rect_filled(rect, Rounding::none(), Color32::RED);
                }
            }
        });

        ui.label(format!("x: {} y: {}", self.loc.x, self.loc.y));

        let pn = *player;
        ui.add(Collapsable::<_, _, &mut EventWriter<PlanetEvent>>::closed(
            |ui: &mut egui::Ui, open: &mut bool, _: &mut &mut EventWriter<PlanetEvent>| {
                let resp = color_option(ui, COLORS[pn.0], Vec2::splat(32.), false);

                if resp.clicked() {
                    *open = !*open;
                };

                resp
            },
            |ui: &mut egui::Ui, open: &mut bool, pe: &mut &mut EventWriter<PlanetEvent>| {
                ui.horizontal_wrapped(move |ui| {
                    let size = Vec2::splat(32.0);
                    for (i, color) in COLORS.into_iter().enumerate() {
                        if color_option(ui, color, size, player.0 == i).clicked() {
                            pe.send(PlanetEvent::SetPlayer {
                                id: self.entity,
                                player: Player(i),
                            });
                            *open = false;
                        }
                    }
                })
                .response
            },
            &mut self.events,
            self.i,
        ))
    }
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

struct SceneChannel(SyncSender<SceneEvent>, Receiver<SceneEvent>);
impl Default for SceneChannel {
    fn default() -> Self {
        let (s, r) = sync_channel(5);
        Self(s, r)
    }
}

struct PWUrl(String);
impl Default for PWUrl {
    fn default() -> Self {
        Self(String::from("https://planetwars.dev/api/maps"))
    }
}

#[allow(clippy::too_many_arguments)]
fn ui_editor(
    mut egui_context: ResMut<EguiContext>,
    query: Query<(&Location, &PlanetData, Entity, &Selected), Without<HoverPlanet>>,
    mut hovering_ui: ResMut<HoveringUI>,
    mut planet_events: EventWriter<PlanetEvent>,
    mut scene_events: EventWriter<SceneEvent>,

    mut size_buf: Local<String>,
    mut url_buf: Local<PWUrl>,
    mut map_name: Local<String>,
    mut scale: Local<f32>,
    mut enabled: Local<bool>,
    mut help_closed: Local<bool>,
) {
    hovering_ui.0 = false;
    let resp = egui::SidePanel::right("right_panel")
        .min_width(250.)
        .resizable(true)
        .show(egui_context.ctx_mut(), |ui| {
            ui.add_space(8.);
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    scene_events.send(SceneEvent::Save);
                }

                if ui.button("Load").clicked() {
                    scene_events.send(SceneEvent::Load);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Longest expedition in turns: ");
                if ui.text_edit_singleline(size_buf.deref_mut()).changed() {
                    if let Ok(ns) = size_buf.parse() {
                        *scale = ns;
                        *enabled = true;
                    } else {
                        *enabled = false;
                    }
                };
            });

                ui.label("Planetwars upload url: ");
                ui.text_edit_singleline(&mut url_buf.deref_mut().0);
                ui.label("Map name: ");
                ui.text_edit_singleline(map_name.deref_mut());

            ui.add_enabled_ui(*enabled && map_name.len() > 0, |ui| {
                ui.horizontal(|ui| {

                if ui.button("Export").clicked() {
                    scene_events.send(SceneEvent::Export{girth: *scale, name: map_name.to_string()});
                }

                if ui.button("Upload").clicked() {
                    scene_events.send(SceneEvent::Upload{girth: *scale, url: url_buf.0.clone(), name: map_name.to_string()});
                }
                });
            });

            ui.add_space(8.);
            ui.separator();
            ui.add_space(8.);

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label(RichText::new("Selected").color(ZEUS));
                for (i, (l, player, e, s)) in query.iter().enumerate() {
                    if s.0 {
                        ui.add(PlanetWidget {
                            i,
                            data: player,
                            loc: l,
                            entity: e,
                            events: &mut planet_events,
                        });
                        ui.separator();
                    }
                }

                ui.label(RichText::new("Others").color(ZEUS));

                for (i, (l, player, e, s)) in query.iter().enumerate() {
                    if !s.0 {
                        ui.add(PlanetWidget {
                            i,
                            data: player,
                            loc: l,
                            entity: e,
                            events: &mut planet_events,
                        });
                        ui.separator();
                    }
                }

            if !*help_closed {
                ui.separator();
                    ui.heading("Creation");
                    ui.label("Move around with WASD or arrow keys");
                    ui.label("Left click to place a planet.");
                    ui.label("Right click to place a planet.");
                    ui.label("Right click again to show the name name of the planet.");
                    ui.label("Right click to delete a planet.");
                    ui.label("Click colored squares to change planet owner.");
                    ui.label("Click different shapes to change the map layout.");
                ui.separator();
                    ui.heading("Editing");
                    ui.label("Right, change the name of the planet and ship count.");
                    ui.label("You can also change to owner of a planet on the right side.");
                ui.separator();
                    ui.heading("Exporting");
                    ui.label("'Save' lets you save the editor's state to resume later.");
                    ui.label("'Load' lets you resume the eidtor's state.");
                    ui.label("'Export' lets you export your creation to a valid planetwars map.");
                    ui.label("Longest expedition is a field that changes the scale of the map");
                    ui.label("the number entered is the total number of turns between the furthest planets on the map.");
                ui.separator();
                    if ui.button("Close help").clicked() {
                        *help_closed = true;
                    }
            }
            });

        })
        .response;
    hovering_ui.0 = hovering_ui.0 || resp.hovered();
}

struct IconButton {
    id: TextureId,
    selected: bool,
}

impl Widget for IconButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let sense = Sense::click_and_drag();
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(32.0), sense);
        if self.selected {
            let shape = Shape::rect_filled(rect.shrink(2.), 4., Color32::DARK_GRAY);
            ui.painter().add(shape);
        }

        let mut mesh = egui::Mesh::with_texture(self.id);
        let tint = if response.hovered() {
            ZEUS
        } else {
            Color32::WHITE
        };

        mesh.add_rect_with_uv(rect.shrink(4.), uv, tint);
        ui.painter().add(Shape::mesh(mesh));

        response
    }
}

fn ui_system(
    mut egui_context: ResMut<EguiContext>,
    fps: Res<FPS>,
    config: Res<MapConfig>,
    mut player: ResMut<Player>,
    mut hovering_ui: ResMut<HoveringUI>,
    icons: Res<Icons>,
    mut writer: EventWriter<MapEvent>,
) {
    egui::TopBottomPanel::bottom("bottom_panel")
        // .default_height(70.)
        .show(egui_context.ctx_mut(), |ui| {
            let response =
                ui.allocate_response(ui.available_size_before_wrap(), egui::Sense::hover());
            hovering_ui.0 = hovering_ui.0 || response.hovered();

            ui.allocate_ui_at_rect(response.rect, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(format!("fps {}", fps.0));
                    ui.label(format!("hovering {}", hovering_ui.0));
                    ui.label(format!("zoom {}", config.zoom));

                    let size = Vec2::splat(32.0);
                    for (i, color) in COLORS.into_iter().enumerate() {
                        if color_option(ui, color, size, i == player.0).clicked() {
                            player.0 = i;
                        }
                    }

                    ui.separator();

                    if ui
                        .add(IconButton {
                            id: icons.triangles,
                            selected: config.ty == MapType::Triangles,
                        })
                        .clicked()
                        && config.ty != MapType::Triangles
                    {
                        writer.send(MapEvent::SetType(MapType::Triangles));
                    }

                    if ui
                        .add(IconButton {
                            id: icons.squares,
                            selected: config.ty == MapType::Squares,
                        })
                        .clicked()
                        && config.ty != MapType::Squares
                    {
                        writer.send(MapEvent::SetType(MapType::Squares));
                    }

                    if ui
                        .add(IconButton {
                            id: icons.hexagons,
                            selected: config.ty == MapType::Hexagons,
                        })
                        .clicked()
                        && config.ty != MapType::Hexagons
                    {
                        writer.send(MapEvent::SetType(MapType::Hexagons));
                    }

                    if ui
                        .add(IconButton {
                            id: icons.octagons,
                            selected: config.ty == MapType::Octagons,
                        })
                        .clicked()
                        && config.ty != MapType::Octagons
                    {
                        writer.send(MapEvent::SetType(MapType::Octagons));
                    }

                    ui.separator();
                })
            })
        });
}
