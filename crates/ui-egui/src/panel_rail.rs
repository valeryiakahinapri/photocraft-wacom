//! The right dock's panel rail: the narrow strip of panel-group buttons beside the canvas.
//!
//! Icons while the dock column shows; the rail stays reachable when the dock itself is hidden
//! (`window.toggle.dock`) so hidden panels can be brought back, and it hides with the whole chrome
//! when Tab does or an embedding app turns `panels.rail` off. Extracted from `panels::right_dock`
//! so rail behaviours (visibility toggling, collapsed-dock labels, floating peeks) can live
//! beside the dock machinery without growing the panels module.

use crate::PhotocraftApp;
use crate::dock::Group;
use crate::state::Panels;
use crate::theme::Tokens;
use egui::{Stroke, pos2};
use serde_json::json;

/// The floating peek window's id (see [`peek_id`]).
const PEEK: &str = "panel-peek";

pub(crate) fn peek_id() -> egui::Id {
    egui::Id::new(PEEK)
}

/// Show or hide a panel group from the rail's visibility popover and automation. One path per
/// dock mode: the Window menu's `window.toggle.<key>` ids cannot be used here — Character has
/// no slot in that arm and unchecking Brushes in a legacy workspace would migrate it to
/// customized docking (panel_docking intercepts the id even while legacy is on) — so the rows
/// carry an explicit show/hide through commands that work the same both ways.
pub(crate) fn set_group_shown(app: &mut PhotocraftApp, group: Group, show: bool) {
    if app.ui.docking.is_some() {
        // Customized: activate restores the group from its hidden return location; closing the
        // first member closes its tab group, so the checkbox follows `sync_visibility`.
        let Some(primary) = crate::panel_docking::group_ids(group).first().copied() else { return };
        let present = app.ui.docking.as_ref().is_some_and(|layout| layout.contains(&primary.to_string()));
        if show {
            if let Err(error) = app.run("window.panel.activate", json!({"panel": primary})) {
                app.ui.status = error;
            }
        } else if present {
            if let Err(error) = app.run("window.panel.group", json!({"panel": primary, "operation": "close"})) {
                app.ui.status = error;
            }
        }
        return;
    }
    // Legacy: `window.panel.<primary>` carries the group flag and picks the theme-right tab.
    // Brushes has no such id, so its flag flips directly (with the #129 reveal when showing).
    if group == Group::Brushes {
        app.ui.panels.brushes = show;
        if show {
            crate::dock::reveal(app, group);
        }
        return;
    }
    let primary = group.key();
    if let Err(error) = app.run(&format!("window.panel.{primary}"), json!({"show": show})) {
        app.ui.status = error;
    }
}

/// The group icons in the rail's fixed (Photoshop Essentials) display order.
const ICON_ORDER: [Group; 7] = [Group::Properties, Group::Navigator, Group::Color, Group::Layers, Group::History, Group::Brushes, Group::Character];

/// The rail icon of `group` (also used by the visibility popover's rows).
fn icon(group: Group) -> &'static str {
    match group {
        Group::Properties => "sliders-horizontal",
        Group::Navigator => "navigation",
        Group::Color => "palette",
        Group::Layers => "layers",
        Group::History => "clock",
        Group::Brushes => "brush",
        Group::Character => "type",
    }
}

/// Draw the rail. `customized` is whether the shared (dragged-out) panel layout owns visibility.
pub(crate) fn rail(app: &mut PhotocraftApp, ui: &mut egui::Ui, p: &Panels, customized: bool) {
    let t = Tokens::get(ui.ctx());
    let (rw, rb) = if t.pro { (36.0, 28.0) } else { (44.0, 32.0) };
    crate::direction::end_panel("rail")
        .resizable(false)
        .exact_size(rw)
        .frame(egui::Frame::NONE.fill(t.chrome).corner_radius(crate::panels::chrome_panel_radius(&t)).inner_margin(egui::Margin::symmetric(4, 8)))
        .show(ui, |ui| {
            // The separator is on the canvas side.
            let r = ui.max_rect();
            let x = if crate::direction::is_rtl() { r.right() + 6.0 } else { r.left() - 6.0 };
            ui.painter().line_segment([pos2(x, r.top() - 8.0), pos2(x, r.bottom() + 8.0)], Stroke::new(1.0, t.separator));
            ui.spacing_mut().item_spacing.y = 4.0;
            // The visibility toggle on top of the rail: one place to hide or show the dock
            // column and each panel group in it, whatever dock mode is active.
            ui.push_id("panel-toggle", |ui| {
                let resp = crate::icons::button(ui, "panel-right", rb, p.dock, tl!("Show/Hide Panels"));
                let align = if crate::direction::is_rtl() { egui::RectAlign::RIGHT_START } else { egui::RectAlign::LEFT_START };
                egui::Popup::menu(&resp).align(align).show(|ui| visibility_menu(app, ui, p, customized));
            });
            if p.dock {
                ui.separator();
                for g in ICON_ORDER {
                    // Studio floats Properties outside the dock.
                    let docked = t.pro || g != Group::Properties;
                    let on = if customized { crate::panel_docking::rail_on(app, g) } else { g.shown(p) && !(docked && app.ui.dock.is_collapsed(g)) };
                    if crate::icons::rail_button(ui, icon(g), rb, on, g.label()).clicked() {
                        if customized {
                            crate::panel_docking::rail_click(app, g);
                        } else {
                            crate::dock::rail_click(app, g, docked);
                        }
                    }
                }
            }
        });
}

/// The rail toggle's popover: the dock column's own visibility, then one row per panel group
/// (`set_group_shown` carries the show/hide, whichever dock mode is active).
fn visibility_menu(app: &mut PhotocraftApp, ui: &mut egui::Ui, p: &Panels, customized: bool) {
    crate::widgets::style_spectrum_popup_menu(ui);
    ui.set_min_width(200.0);
    if ui.button(if p.dock { tl!("Hide Dock") } else { tl!("Show Dock") }).clicked() {
        let _ = app.run("window.toggle.dock", json!({}));
        ui.close();
    }
    ui.separator();
    for g in app.ui.dock.order() {
        let mut on = g.shown(p);
        ui.horizontal(|ui| {
            let tint = ui.visuals().text_color();
            let (rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
            crate::icons::paint(ui, rect, icon(g), 14.0, tint);
            if ui.checkbox(&mut on, tl!(g.label())).changed() {
                set_group_shown(app, g, on);
            }
        });
    }
}
