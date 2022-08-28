use std::hash::Hash;

use crate::{map_config::MapConfig, HoverPlanet, HoveringUI, Location, PlanetData, planet::{COLORS, PlanetEvent, Player}};
use bevy::prelude::{Query, Res, ResMut, Without, EventWriter, Entity, Plugin};
use bevy_egui::{egui, EguiContext};
use egui::{Color32, Response, Rounding, Sense, Stroke, Ui, Vec2, Widget, WidgetWithState};

use crate::FPS;


pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(ui_editor).add_system(ui_system);
    }
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
    query: Query<(&Location, &PlanetData, Entity), Without<HoverPlanet>>,
    mut hovering_ui: ResMut<HoveringUI>,
    mut planet_events: EventWriter<PlanetEvent>,
) {
    hovering_ui.0 = false;
    let resp = egui::SidePanel::right("right_panel")
        .resizable(true)
        .show(egui_context.ctx_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {

                for (i, (l, player, e)) in query.iter().enumerate() {
                    let PlanetData {
                        ref player,
                        ref name,
                    } = player;

                    let mut name = name.clone();

                    ui.horizontal(|ui| {
                        ui.label("planet:");
                        let resp = ui.add_sized(ui.available_size(), egui::TextEdit::singleline(&mut name));
                        if resp.changed() {
                            planet_events.send(PlanetEvent::SetName {
                                id: e,
                                name,
                            });
                        }
                    });

                    ui.label(format!("x: {} y: {}", l.x, l.y));
                    let pn = *player;
                    ui.add(Collapsable::<_, _, &mut EventWriter<PlanetEvent>>::closed(
                        |ui: &mut egui::Ui, open: &mut bool, _: &mut &mut EventWriter<PlanetEvent>| {
                            let resp =
                                color_option(ui, COLORS[pn.0], Vec2::splat(32.), false);

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
                                            id: e,
                                            player: Player(i)
                                        });
                                        *open = false;
                                    }
                                }
                            })
                            .response
                        },
                        &mut planet_events,
                        i,
                    ));
                    ui.separator();
                }
            })
        })
        .response;
    hovering_ui.0 = hovering_ui.0 || resp.hovered();
}


pub fn ui_system(
    mut egui_context: ResMut<EguiContext>,
    fps: Res<FPS>,
    config: Res<MapConfig>,
    mut player: ResMut<Player>,
    mut hovering_ui: ResMut<HoveringUI>,
) {
    let resp = egui::TopBottomPanel::bottom("bottom_panel")
        .show(egui_context.ctx_mut(), |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(format!("fps {}", fps.0));
                ui.label(format!("zoom {}", config.zoom));

                let size = Vec2::splat(32.0);
                for (i, color) in COLORS.into_iter().enumerate() {
                    if color_option(ui, color, size, i == player.0).clicked() {
                        player.0 = i ;
                    }
                }
            })
        })
        .response;

    hovering_ui.0 = hovering_ui.0 || resp.hovered();
}

