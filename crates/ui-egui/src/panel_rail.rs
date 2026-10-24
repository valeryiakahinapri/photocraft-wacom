//! The right dock's panel rail: the narrow strip of panel-group buttons beside the canvas.
//!
//! The same icon list shows in both states (the owner's modern-canvas-studio grammar): while
//! the dock column shows, an icon toggles its group (collapse-to-strip, #129-safe); while the
//! column is hidden — folded (`window.toggle.dockExpanded`, the rail's top toggle) or hidden
//! outright (`window.toggle.dock`, ⇧Tab) — the icon opens the group as a floating peek, the
//! one lit icon, and clicking it again closes it. The fold toggle on top folds and unfolds the
//! column's content without touching its visibility; the fold is its own persisted preference,
//! so Tab round-trips never undo it. The rail hides with the whole chrome when Tab does or an
//! embedding app turns `panels.rail` off. Extracted from `panels::right_dock` so rail
//! behaviours (visibility toggling, peeks) live beside the dock machinery without growing the
//! panels module.

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

/// The group icons in the rail's fixed (Photoshop Essentials) display order.
const ICON_ORDER: [Group; 7] = [Group::Properties, Group::Navigator, Group::Color, Group::Layers, Group::History, Group::Brushes, Group::Character];

/// The rail icon of `group`.
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

/// Draw the rail. `customized` is whether the shared (dragged-out) panel layout owns visibility;
/// `column_shown` is whether the dock column is visible right now (visible AND unfolded, or the
/// #6760 tabbed-in chrome): the icons toggle groups while it shows and open peeks while not.
pub(crate) fn rail(app: &mut PhotocraftApp, ui: &mut egui::Ui, p: &Panels, customized: bool, column_shown: bool) {
    let t = Tokens::get(ui.ctx());
    let rtl = crate::direction::is_rtl();
    let (rw, rb) = if t.pro || t.islands { (44.0, 28.0) } else { (52.0, 32.0) };
    // The island shell (REV6): the rail is the seam half of the right-side island. Its
    // screen-edge margin animates with the column's own slide value (the same
    // `animate_bool_responsive` id egui's `show_collapsible` drives): 0 while the column
    // claims — the seam abuts its segment — swinging to 8 once folded, where the rail
    // becomes a complete rounded island at the screen edge.
    let edge = if t.islands {
        let anim = egui::Id::new(if customized { "shared-panel-dock" } else { "dock" }).with("animation");
        let fraction = ui.ctx().animate_bool_responsive(anim, column_shown);
        (8.0 * (1.0 - fraction)).round()
    } else {
        0.0
    };
    let frame = if t.islands {
        let edge = edge as i8;
        if rtl {
            egui::Frame::NONE.fill(t.page).inner_margin(egui::Margin { left: 8, right: edge, top: 4, bottom: 4 })
        } else {
            egui::Frame::NONE.fill(t.page).inner_margin(egui::Margin { left: 4, right: edge, top: 4, bottom: 4 })
        }
    } else {
        egui::Frame::NONE
            .fill(t.canvas)
            .corner_radius(crate::panels::chrome_panel_radius(&t))
            .outer_margin(egui::Margin::same(4))
            .inner_margin(egui::Margin::symmetric(4, 8))
    };
    let resp = crate::direction::end_panel("rail").resizable(false).exact_size(if t.islands { rw + 4.0 + edge } else { rw }).frame(frame).show(ui, |ui| {
        if t.islands {
            // The rail segment of the right-side island: rounded toward the canvas,
            // square at the seam while the column claims, all four corners once folded
            // (the edge margin's swing carries the seam-side rounding with it).
            let r = ui.max_rect();
            let rr = t.radius.round() as u8;
            let seam_round = (rr as f32 * (edge / 8.0)).round() as u8;
            let radius = if rtl {
                egui::CornerRadius { nw: seam_round, ne: rr, sw: seam_round, se: rr }
            } else {
                egui::CornerRadius { nw: rr, ne: seam_round, sw: rr, se: seam_round }
            };
            crate::panels::paint_island_rect(ui.painter(), &t, r, radius);
            // The mockup rail's `border-r`: a hairline at the seam, only while the column
            // claims its half of the island.
            if edge < 7.0 {
                let x = if rtl { r.left() } else { r.right() };
                ui.painter().line_segment([pos2(x, r.top()), pos2(x, r.bottom())], Stroke::new(1.0, t.separator));
            }
        } else {
            // The separator is on the canvas side; the outer margin puts the island's edge 4px
            // further out than the content rect, so the line clears it by 2px at ±10.
            let r = ui.max_rect();
            let x = if rtl { r.right() + 10.0 } else { r.left() - 10.0 };
            ui.painter().line_segment([pos2(x, r.top() - 8.0), pos2(x, r.bottom() + 8.0)], Stroke::new(1.0, t.separator));
        }
        ui.spacing_mut().item_spacing.y = 4.0;
        // The fold toggle on top of the rail: one press folds or unfolds the dock column's
        // content (`window.toggle.dockExpanded` — the fold is its own persisted preference;
        // ⇧Tab's `window.toggle.dock` hides the column outright). UI-level ids dispatch
        // through menus::invoke — the menu bar and keybinding path; app.run goes straight
        // to the engine, which only catalogs these ids.
        let (fold_icon, fold_label) = if p.dock_expanded {
            (if crate::direction::is_rtl() { "chevrons-left" } else { "chevrons-right" }, tl!("Collapse Panel Dock"))
        } else {
            (if crate::direction::is_rtl() { "chevrons-right" } else { "chevrons-left" }, tl!("Expand Panel Dock"))
        };
        if crate::icons::button(ui, fold_icon, rb, false, fold_label).clicked() {
            if let Err(error) = crate::menus::invoke(app, ui.ctx(), "window.toggle.dockExpanded", json!({})) {
                app.ui.status = error;
            }
        }
        ui.add_space(3.0);
        let (separator, _) = ui.allocate_exact_size(egui::vec2(rb, 1.0), egui::Sense::hover());
        ui.painter().line_segment([separator.left_center(), separator.right_center()], Stroke::new(1.0, t.separator));
        ui.add_space(3.0);
        // One constant icon list in both states (the owner's mockup grammar): the icons
        // toggle their group while the column shows, and open a floating peek while it is
        // hidden — clicking the peeked icon again closes it (the no-params `window.panel.peek`
        // parity). The peeked icon is the only lit one in the hidden state.
        for g in ICON_ORDER {
            // Studio floats Properties outside the dock.
            let docked = t.pro || g != Group::Properties;
            let peeked = app.ui.panel_peek.as_ref().is_some_and(|peek| peek.group == g);
            let on = if column_shown {
                if customized { crate::panel_docking::rail_on(app, g) } else { g.shown(p) && !(docked && app.ui.dock.is_collapsed(g)) }
            } else {
                peeked
            };
            if crate::icons::rail_button(ui, icon(g), rb, on, g.label()).clicked() {
                if column_shown {
                    if customized {
                        crate::panel_docking::rail_click(app, g);
                    } else {
                        crate::dock::rail_click(app, g, docked);
                    }
                } else if peeked {
                    app.ui.panel_peek = None;
                } else if edge < 7.0 {
                    // Mid-slide: a peek would anchor to the column's still-animating edge and
                    // stay there; the settled frame places it correctly (audit P2).
                } else {
                    let tab = *g.tab_mut(&mut app.ui.dock_tabs);
                    let anchor = ui.ctx().pointer_interact_pos().or_else(|| ui.ctx().pointer_latest_pos());
                    app.ui.panel_peek = Some(crate::state::PanelPeek { group: g, tab, anchor, first: true });
                }
            }
        }
    });
    // The peek window places itself beside the rail and keeps clicks on the rail from
    // counting as "outside" (see [`peek_window`]).
    ui.ctx().data_mut(|d| d.insert_temp(rail_rect_id(), resp.response.rect));
}

/// Whether the rail draws at all: its own flag, minus the chrome Tab hid with the dock (so
/// Tab hides everything, #1313) — the same rule `panels::right_dock` gates the rail with.
pub(crate) fn rail_visible(p: &Panels) -> bool {
    p.rail && !(p.hidden_by_tab.is_some() && !p.dock)
}

/// The id the rail stores its last screen rect under, for the peek window's placement and
/// focus-loss rule (see [`rail`], [`peek_window`]).
fn rail_rect_id() -> egui::Id {
    egui::Id::new("panel-rail-rect")
}

/// The peek window's id is per group: a user-resized window keeps its size for that group's
/// peeks, without imposing it on another group's (`Window` area state lives by id).
fn peek_window_id(g: Group) -> egui::Id {
    peek_id().with(g.key())
}

/// Draw the floating peek window of the group opened from the collapsed-dock rail
/// ([`rail`]). A peek follows focus, like inspector panels that collapse: Escape, the window's
/// close, a press on another rail icon, or the next pointer press outside the window and the
/// rail closes it and returns the group to its dimmed icon. Never shown while the dock column
/// itself is visible (a peek is a hidden-column affordance; the arms in `window.toggle.dock`
/// and `window.toggle.dockExpanded` clear it).
pub(crate) fn peek_window(app: &mut PhotocraftApp, ctx: &egui::Context) {
    let Some(peek) = app.ui.panel_peek.clone() else { return };
    let g = peek.group;
    // A peek is a hidden-column affordance only: the column coming back — however that happened
    // (the fold opening, the dock returning while unfolded, the #6760 tabbed-in chrome, ui.set,
    // a workspace) — or the rail itself disappearing ends it, whatever path forgot to clear the
    // state on the way. The dock returning while folded keeps it: the column stays hidden, so
    // the peek is still the only view of that panel.
    let tabbed_in = app.ui.view.hides_chrome() && app.ui.panels.full_screen_chrome;
    if (app.ui.panels.dock && app.ui.panels.dock_expanded) || tabbed_in || !rail_visible(&app.ui.panels) {
        app.ui.panel_peek = None;
        return;
    }
    if let Some(p) = ctx.input(|i| i.pointer.any_pressed().then(|| i.pointer.press_origin()).flatten()) {
        // On the opening frame the window area does not exist yet: keep it (dock.rs's peek
        // counts a rect it just drew; here the click that opened came from the rail anyway).
        let inside_window = ctx.memory(|m| m.area_rect(peek_window_id(g))).is_none_or(|r| r.contains(p));
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
        .fill(if t.islands { t.popover } else { t.card })
        .stroke(egui::Stroke::new(1.0, t.card_border))
        .corner_radius(egui::CornerRadius::same(t.radius_lg as u8))
        .shadow(egui::Shadow { offset: [0, 10], blur: 30, spread: 0, color: t.shadow })
        .inner_margin(egui::Margin::same(8));
    let mut window = egui::Window::new(tl!(g.label()).to_owned())
        .id(peek_window_id(g))
        .title_bar(false)
        .resizable(true)
        .min_size(egui::vec2(280.0, g.min_height()))
        .default_size(egui::vec2(width, g.default_height()))
        .frame(frame);
    if peek.first {
        // Beside the rail, at the click that opened it (Windows/automation without an anchor
        // get the rail's top). `current_pos` applies once: after that the window owns its spot.
        let area = ctx.content_rect();
        let fallback = if crate::direction::is_rtl() {
            egui::Rect::from_min_size(egui::pos2(area.left(), area.top()), egui::vec2(36.0, area.height()))
        } else {
            egui::Rect::from_min_size(egui::pos2((area.right() - 36.0).max(area.left()), area.top()), egui::vec2(36.0, area.height()))
        };
        let rail = ctx.data(|d| d.get_temp::<egui::Rect>(rail_rect_id())).unwrap_or(fallback);
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
        if t.islands {
            // The mockup flyout header's `border-b` hairline.
            crate::widgets::hairline(ui);
        }
        // The group's tabs as pills, like the dock's strip: a switch selects the panel the
        // dock would show, so a re-shown dock and the peek agree on the tab. The tab is read
        // live: automation may have changed the selection while peeked.
        let tab = *g.tab_mut(&mut app.ui.dock_tabs);
        let visible = app.ui.dock.visible_tabs(g, pro);
        if visible.len() > 1 {
            crate::widgets::hairline(ui);
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                for (tab_i, name) in visible {
                    let label = tl!(name);
                    if crate::widgets::pill_tab(ui, &label, tab == tab_i).clicked() {
                        *g.tab_mut(&mut app.ui.dock_tabs) = tab_i;
                        if let Some(current) = app.ui.panel_peek.as_mut() {
                            current.tab = tab_i;
                        }
                    }
                }
            });
        }
        crate::widgets::hairline(ui);
        ui.add_space(4.0);
        let tab = *g.tab_mut(&mut app.ui.dock_tabs);
        let inner = ui.available_height().max(0.0);
        if g.scrolls_itself(tab) {
            ui.set_min_height(inner);
            crate::panels::dock_body(app, ui, g, tab);
        } else {
            crate::panels::offer_color_field_fill(ui, pro, g, tab);
            ui.spacing_mut().scroll.fade.strength = 0.0;
            egui::ScrollArea::vertical().id_salt(("peek-scroll", g.key())).max_height(inner).auto_shrink([false, false]).show(ui, |ui| {
                crate::panels::dock_body(app, ui, g, tab);
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

#[cfg(test)]
#[path = "panel_rail_tests.rs"]
mod tests;
