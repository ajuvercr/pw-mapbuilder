use std::hash::Hash;

use crate::{
    map_config::{MapConfig, MapEvent, MapType},
    planet::{PlanetEvent, Player, COLORS},
    HoverPlanet, HoveringUI, Location, PlanetData, ZEUS,
};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use egui::{
    pos2, widgets, Color32, ColorImage, FontImage, ImageButton, ImageData, Rect, Response,
    Rounding, Sense, Shape, Stroke, TextureHandle, TextureId, Ui, Vec2, Widget, WidgetWithState,
};

use crate::FPS;

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(ui_editor)
            .add_system(ui_system)
            .init_resource::<Icons>()
            .add_startup_system(load_images)
            // .add_system(set_images)
            .insert_resource(HoveringUI(false));
    }
}

#[derive(Default)]
struct Icons {
    handles: Vec<Handle<Image>>,
    squares: TextureId,
    triangles: TextureId,
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
}

// fn set_images(
//     mut icons: ResMut<Icons>,
//     mut events: EventReader<AssetEvent<Image>>,
//     mut ctx: ResMut<EguiContext>,
//     assets: Res<Assets<Image>>,
// ) {
//     for event in events.iter() {
//         match event {
//             AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
//                 if handle.id == icons.triangles.1.id {
//                     let image = assets.get(handle).unwrap();
//                     let size = image.size();
//                     let color = ColorImage::from_rgba_unmultiplied([size.x as usize, size.y as usize], &image.data);
//
//                     let handle = ctx.add_image(handle)("traingles", color.into());
//
//                 }
//
//                 if handle.id == icons.squares.1.id {
//                     icons.squares.0 = assets.get(handle).cloned();
//                 }
//             }
//             _ => {}
//         }
//     }
// }

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
                        let resp = ui
                            .add_sized(ui.available_size(), egui::TextEdit::singleline(&mut name));
                        if resp.changed() {
                            planet_events.send(PlanetEvent::SetName { id: e, name });
                        }
                    });

                    ui.label(format!("x: {} y: {}", l.x, l.y));
                    let pn = *player;
                    ui.add(Collapsable::<_, _, &mut EventWriter<PlanetEvent>>::closed(
                        |ui: &mut egui::Ui,
                         open: &mut bool,
                         _: &mut &mut EventWriter<PlanetEvent>| {
                            let resp = color_option(ui, COLORS[pn.0], Vec2::splat(32.), false);

                            if resp.clicked() {
                                *open = !*open;
                            };

                            resp
                        },
                        |ui: &mut egui::Ui,
                         open: &mut bool,
                         pe: &mut &mut EventWriter<PlanetEvent>| {
                            ui.horizontal_wrapped(move |ui| {
                                let size = Vec2::splat(32.0);
                                for (i, color) in COLORS.into_iter().enumerate() {
                                    if color_option(ui, color, size, player.0 == i).clicked() {
                                        pe.send(PlanetEvent::SetPlayer {
                                            id: e,
                                            player: Player(i),
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

struct IconButton {
    id: TextureId,
    selected: bool,
}

impl Widget for IconButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let sense = Sense::hover().union(Sense::click());
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
    let resp = egui::TopBottomPanel::bottom("bottom_panel")
        .show(egui_context.ctx_mut(), |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(format!("fps {}", fps.0));
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
                    println!("seting triangles");
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

                ui.separator();
            })
        })
        .response;

    hovering_ui.0 = hovering_ui.0 || resp.hovered();
}
