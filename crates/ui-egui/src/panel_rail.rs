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
    let resp = crate::direction::end_panel("rail")
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
                egui::Popup::menu(&resp).align(align).show(|ui| visibility_menu(app, ui, p));
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
            } else {
                // Collapsed dock: the active groups list vertically, their names rotated a
                // quarter turn (bottom-up, mirrored top-down in a right-to-left UI), each
                // click opening the group as a floating peek (`peek_window`).
                let scroll = egui::ScrollArea::vertical().id_salt("panel-rail-labels").auto_shrink([false, false]);
                scroll.show(ui, |ui| {
                    for g in app.ui.dock.order() {
                        label_chip(app, ui, &t, g, rw - 8.0);
                    }
                });
            }
        });
    // The peek window places itself beside the rail and keeps clicks on the rail from
    // counting as "outside" (see [`peek_window`]).
    ui.ctx().data_mut(|d| d.insert_temp(rail_rect_id(), resp.response.rect));
}

/// The rail toggle's popover: the dock column's own visibility, then one row per panel group
/// (`set_group_shown` carries the show/hide, whichever dock mode is active; both modes agree
/// on the rows' checked state through `sync_visibility`).
fn visibility_menu(app: &mut PhotocraftApp, ui: &mut egui::Ui, p: &Panels) {
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

/// One vertical (90°) rail label for a panel group, Photoshop's collapsed-dock style: the name
/// reads bottom-up in a left-to-right UI, mirrored top-down in a right-to-left one. Clicking
/// opens the group as a floating peek ([`peek_window`]); the label stays accent-tinted while its
/// group is the one peeked. `width` is the rail's inner width.
fn label_chip(app: &mut PhotocraftApp, ui: &mut egui::Ui, t: &Tokens, g: Group, width: f32) {
    // The same groups the icon rail would light up (panel_docking's projection in customized
    // mode, plain visibility minus all-Closed groups in the legacy one).
    let active = if app.ui.docking.is_some() {
        crate::panel_docking::rail_on(app, g)
    } else {
        g.shown(&app.ui.panels) && !app.ui.dock.visible_tabs(g, t.pro).is_empty()
    };
    if !active {
        return;
    }
    let peeked = app.ui.panel_peek.as_ref().is_some_and(|peek| peek.group == g);
    let font = egui::FontId::proportional(11.5);
    let galley = ui.ctx().fonts_mut(|f| f.layout_no_wrap(tl!(g.label()).to_owned(), font, egui::Color32::PLACEHOLDER));
    let (along, across) = (galley.rect.width(), galley.rect.height());
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, along + 8.0), egui::Sense::click());
    if peeked {
        ui.painter().rect_filled(rect, t.radius_sm, t.accent_soft);
        ui.painter().rect_stroke(rect, t.radius_sm, egui::Stroke::new(1.0, t.accent_border), egui::StrokeKind::Inside);
    } else if resp.hovered() {
        ui.painter().rect_filled(rect, t.radius_sm, t.hover);
    }
    // The text color rides on the shape (the galley holds placeholders), so the hover state is
    // decided here, after allocation, without relaying the text out.
    let tint = if peeked {
        t.accent_text
    } else if resp.hovered() {
        t.text
    } else {
        t.text_faint
    };
    let pad = 4.0;
    let (pos, angle) = if crate::direction::is_rtl() {
        (egui::pos2(rect.left() + (width + across) / 2.0, rect.top() + pad), std::f32::consts::FRAC_PI_2)
    } else {
        (egui::pos2(rect.left() + (width - across) / 2.0, rect.bottom() - pad), -std::f32::consts::FRAC_PI_2)
    };
    ui.painter().add(egui::epaint::TextShape::new(pos, galley, tint).with_angle(angle));
    if resp.clicked() {
        let tab = *g.tab_mut(&mut app.ui.dock_tabs);
        let anchor = ui.ctx().pointer_interact_pos().or_else(|| ui.ctx().pointer_latest_pos());
        app.ui.panel_peek = Some(crate::state::PanelPeek { group: g, tab, anchor, first: true });
    }
    resp.on_hover_text(tl!(g.label())).on_hover_cursor(egui::CursorIcon::PointingHand);
}

/// The id the rail stores its last screen rect under, for the peek window's placement and
/// focus-loss rule (see [`rail`], [`peek_window`]).
fn rail_rect_id() -> egui::Id {
    egui::Id::new("panel-rail-rect")
}

/// Draw the floating peek window of the group opened from the collapsed-dock rail
/// ([`rail`]). A peek follows focus, like inspector panels that collapse: Escape, the window's
/// close, or the next pointer press outside the window and the rail closes it and returns the
/// group to its rotated label. Never shown while the dock column itself is visible (a peek is
/// a hidden-dock affordance; the arm in `window.toggle.dock` clears it).
pub(crate) fn peek_window(app: &mut PhotocraftApp, ctx: &egui::Context) {
    let Some(peek) = app.ui.panel_peek.clone() else { return };
    let g = peek.group;
    if let Some(p) = ctx.input(|i| i.pointer.any_pressed().then(|| i.pointer.press_origin()).flatten()) {
        // On the opening frame the window area does not exist yet: keep it (dock.rs's peek
        // counts a rect it just drew; here the click that opened came from the rail anyway).
        let inside_window = ctx.memory(|m| m.area_rect(peek_id())).is_none_or(|r| r.contains(p));
        let inside_rail = ctx.data(|d| d.get_temp::<egui::Rect>(rail_rect_id())).is_some_and(|r| r.contains(p));
        if !inside_window && !inside_rail {
            app.ui.panel_peek = None;
            return;
        }
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.ui.panel_peek = None;
        return;
    }
    let t = Tokens::get(ctx);
    let pro = t.pro;
    let width = if pro { 290.0 } else { 300.0 };
    let frame = egui::Frame::NONE
        .fill(t.card)
        .stroke(egui::Stroke::new(1.0, t.card_border))
        .corner_radius(egui::CornerRadius::same(t.radius_lg as u8))
        .shadow(egui::Shadow { offset: [0, 10], blur: 30, spread: 0, color: t.shadow })
        .inner_margin(egui::Margin::same(8));
    let mut window = egui::Window::new(tl!(g.label()).to_owned())
        .id(peek_id())
        .title_bar(false)
        .resizable(true)
        .min_size(egui::vec2(280.0, g.min_height()))
        .default_size(egui::vec2(width, g.default_height()))
        .frame(frame);
    if peek.first {
        // Beside the rail, at the click that opened it (Windows/automation without an anchor
        // get the rail's top). `current_pos` applies once: after that the window owns its spot.
        let area = ctx.content_rect();
        let rail = ctx
            .data(|d| d.get_temp::<egui::Rect>(rail_rect_id()))
            .unwrap_or_else(|| egui::Rect::from_min_size(egui::pos2(area.right(), area.top()), egui::vec2(36.0, area.height())));
        let x = if crate::direction::is_rtl() { rail.right() } else { rail.left() - width };
        let y = peek.anchor.map_or(area.top() + 90.0, |a| a.y.clamp(area.top(), (area.bottom() - g.min_height()).max(area.top())));
        window = window.current_pos(egui::pos2(x, y));
    }
    let mut close = false;
    window.show(ctx, |ui| {
        // Photoshop-style tab header with a close box (preset_panels::float_window's shape).
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(tl!(g.label())).color(t.text).size(12.0).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if crate::icons::button(ui, "x", 20.0, false, tl!("Close")).clicked() {
                    close = true;
                }
            });
        });
        // The group's tabs as pills, like the dock's strip: a switch selects the panel the
        // dock would show, so a re-shown dock and the peek agree on the tab.
        let visible = app.ui.dock.visible_tabs(g, pro);
        if visible.len() > 1 {
            crate::widgets::hairline(ui);
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                for (tab, name) in visible {
                    let label = tl!(name);
                    if crate::widgets::pill_tab(ui, &label, peek.tab == tab).clicked() {
                        *g.tab_mut(&mut app.ui.dock_tabs) = tab;
                        if let Some(current) = app.ui.panel_peek.as_mut() {
                            current.tab = tab;
                        }
                    }
                }
            });
        }
        crate::widgets::hairline(ui);
        ui.add_space(4.0);
        let inner = ui.available_height().max(0.0);
        if g.scrolls_itself(peek.tab) {
            ui.set_min_height(inner);
            crate::panels::dock_body(app, ui, g, peek.tab);
        } else {
            crate::panels::offer_color_field_fill(ui, pro, g, peek.tab);
            ui.spacing_mut().scroll.fade.strength = 0.0;
            egui::ScrollArea::vertical().id_salt(("peek-scroll", g.key())).max_height(inner).auto_shrink([false, false]).show(ui, |ui| {
                crate::panels::dock_body(app, ui, g, peek.tab);
            });
        }
    });
    if let Some(current) = app.ui.panel_peek.as_mut() {
        current.first = false;
    }
    if close {
        app.ui.panel_peek = None;
    }
}
