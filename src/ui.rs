
use std::{hash::Hash, ops::DerefMut};

use bevy::{
    prelude::{Assets, Changed, Handle, Query, Res, ResMut, Without},
    sprite::ColorMaterial,
};
use bevy_egui::{egui, EguiContext};
use egui::{Color32, Response, Rounding, Sense, Stroke, Ui, Vec2, Widget, WidgetWithState};
use crate::{
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
