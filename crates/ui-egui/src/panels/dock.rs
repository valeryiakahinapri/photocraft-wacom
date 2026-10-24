use super::history::history;
use super::info::info_panel;
use super::layers::{channels, layers};
use super::navigator::navigator;
use super::properties::{adjustments_grid, properties_body};
use super::*;

// ----------------------------------------------------------------------------- dock

/// The panels and the canvas in the space left by the menus, bars and toolbar: craft-ui's
/// docking layout, with the icon rail at the window's end edge in the Studio themes.
///
/// The fork's rounded-dock interaction: the dock's content area can fold away
/// (`window.toggle.dockExpanded`) while the icon rail stays, animating the fold; the rail's
/// buttons then open the group in a flyout (`show_rail_flyout`).
pub fn dock_area(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let expanded = app.ui.panels.dock_expanded;
    // The fold/unfold animation; `transitioning` while the animated width is between 0 and the
    // target, so the real dock is only shown once it has fully unfolded.
    let progress = ui.ctx().animate_bool_with_time(egui::Id::new("right-dock-expanded"), expanded, 0.22);
    let transitioning = if expanded { progress < 0.999 } else { progress > 0.001 };
    if expanded && !transitioning {
        if app.ui.panels.rail && !t.pro {
            rail(app, ui, &t);
        }
        egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| crate::panel_docking::show(app, ui));
        crate::dock::persist(app, ui.ctx());
        return;
    }
    // Collapsed (or folding): the rail carries the panels in flyouts; the canvas takes the rest.
    if app.ui.panels.rail {
        rail(app, ui, &t);
    }
    if transitioning {
        let saved = ui.ctx().data(|data| data.get_temp::<f32>(dock_last_width_id())).unwrap_or(if t.pro { 290.0 } else { 300.0 });
        let width = saved * progress;
        crate::direction::end_panel("dock-fold")
            .resizable(false)
            .exact_size(width)
            .frame(egui::Frame::NONE.fill(t.canvas).corner_radius(super::chrome_panel_radius(&t)).inner_margin(egui::Margin::same(4)))
            .show(ui, |_| {});
    }
    let backdrop = if app.ui.view.hides_chrome() { egui::Color32::BLACK } else { crate::prefs_ui::pasteboard_color(app).unwrap_or(t.canvas) };
    egui::CentralPanel::default().frame(egui::Frame::NONE.fill(backdrop)).show(ui, |ui| crate::canvas::document_area(app, ui));
    crate::dock::persist(app, ui.ctx());
}

/// The docking layout's protected canvas panel: the document area, with the icon rail on its end
/// side, beside the dock, in the Pro themes.
pub(crate) fn canvas_panel(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let backdrop = if app.ui.view.hides_chrome() { egui::Color32::BLACK } else { crate::prefs_ui::pasteboard_color(app).unwrap_or(t.canvas) };
    if app.ui.panels.rail && t.pro {
        rail(app, ui, &t);
    }
    egui::CentralPanel::default().frame(egui::Frame::NONE.fill(backdrop)).show(ui, |ui| crate::canvas::document_area(app, ui));
}

/// Narrow icon rail, unless an embedding app hides it: shows, expands or collapses panel groups.
/// While the dock is folded away each button opens its group in a flyout instead.
fn rail(app: &mut PhotocraftApp, ui: &mut egui::Ui, t: &Tokens) {
    let customized = app.ui.docking.is_some();
    let p = app.ui.panels.clone();
    let (rw, rb) = if t.pro { (44.0, 28.0) } else { (52.0, 32.0) };
    let mut flyout = None;
    let mut rail_anchors = Vec::new();
    crate::direction::end_panel("rail")
        .resizable(false)
        .exact_size(rw)
        .frame(
            egui::Frame::NONE
                .fill(t.canvas)
                .corner_radius(super::chrome_panel_radius(&t))
                .outer_margin(egui::Margin::same(4))
                .inner_margin(egui::Margin::symmetric(4, 8)),
        )
        .show(ui, |ui| {
            // The separator is on the canvas side.
            let r = ui.max_rect();
            let x = if crate::direction::is_rtl() { r.right() + 6.0 } else { r.left() - 6.0 };
            ui.painter().line_segment([pos2(x, r.top() - 8.0), pos2(x, r.bottom() + 8.0)], Stroke::new(1.0, t.separator));
            ui.spacing_mut().item_spacing.y = 4.0;
            let expand_icon = if p.dock_expanded { "chevrons-right" } else { "chevrons-left" };
            let expand_label = if p.dock_expanded { tl!("Collapse Panel Dock") } else { tl!("Expand Panel Dock") };
            if icons::button(ui, expand_icon, rb, false, expand_label).clicked()
                && let Err(error) = crate::menus::invoke(app, ui.ctx(), "window.toggle.dockExpanded", serde_json::json!({}))
            {
                app.ui.status = error;
                app.ui.status_error = true;
            }
            ui.add_space(3.0);
            let (separator, _) = ui.allocate_exact_size(egui::vec2(rb, 1.0), egui::Sense::hover());
            ui.painter().line_segment([separator.left_center(), separator.right_center()], Stroke::new(1.0, t.separator));
            ui.add_space(3.0);
            use crate::dock::Group;
            let entries: [(&str, &str, Group); 6] = [
                ("sliders-horizontal", tl!("Properties"), Group::Properties),
                ("navigation", tl!("Navigator"), Group::Navigator),
                ("palette", tl!("Color & Swatches"), Group::Color),
                ("layers", tl!("Layers"), Group::Layers),
                ("clock", tl!("History"), Group::History),
                ("brush", tl!("Brushes"), Group::Brushes),
            ];
            for (icon, name, g) in entries {
                // Studio floats Properties outside the dock.
                let docked = t.pro || g != Group::Properties;
                let open_flyout = ui.ctx().data(|data| data.get_temp::<Group>(rail_flyout_id()));
                let on = if !p.dock_expanded {
                    open_flyout == Some(g)
                } else if customized {
                    crate::panel_docking::rail_on(app, g)
                } else {
                    g.shown(&p) && !(docked && app.ui.dock.is_collapsed(g))
                };
                let response = icons::rail_button(ui, icon, rb, on, name);
                rail_anchors.push((g, response.rect));
                if response.clicked() {
                    if !p.dock_expanded {
                        if open_flyout == Some(g) {
                            ui.ctx().data_mut(|data| data.remove::<Group>(rail_flyout_id()));
                        } else {
                            ui.ctx().data_mut(|data| data.insert_temp(rail_flyout_id(), g));
                            flyout = Some((g, response.rect));
                        }
                    } else {
                        ui.ctx().data_mut(|data| data.remove::<Group>(rail_flyout_id()));
                        if customized {
                            crate::panel_docking::rail_click(app, g);
                        } else {
                            crate::dock::rail_click(app, g, docked);
                        }
                    }
                }
            }
        });
    if let Some((group, anchor)) = flyout {
        show_rail_flyout(app, ui.ctx(), group, anchor, &t);
    } else if let Some(group) = ui.ctx().data(|data| data.get_temp::<crate::dock::Group>(rail_flyout_id()))
        && let Some((_, anchor)) = rail_anchors.iter().find(|(candidate, _)| *candidate == group)
    {
        show_rail_flyout(app, ui.ctx(), group, *anchor, &t);
    }
}

/// The rail's open flyout (`right-dock-flyout`).
fn rail_flyout_id() -> egui::Id {
    egui::Id::new("right-dock-flyout")
}

/// The dock's width at its last fully expanded moment, so unfolding returns to it.
pub(crate) fn dock_last_width_id() -> egui::Id {
    egui::Id::new("right-dock-last-expanded-width")
}

/// A floating copy of one dock group: the rail's flyout while the dock is folded away. Clicking
/// outside, the ×, or Escape closes it; choosing a tab writes the choice back to the dock state.
fn show_rail_flyout(app: &mut PhotocraftApp, ctx: &egui::Context, group: crate::dock::Group, anchor: Rect, t: &Tokens) {
    let screen = ctx.content_rect();
    let width = 300.0f32.min((screen.width() - 16.0).max(180.0));
    let max_height = (screen.height() - 24.0).clamp(180.0, 620.0);
    let x = if crate::direction::is_rtl() { anchor.right() + 8.0 } else { anchor.left() - width - 8.0 };
    let y = (anchor.top() - 18.0).clamp(screen.top() + 8.0, (screen.bottom() - max_height - 8.0).max(screen.top() + 8.0));
    let mut close = false;
    let area = egui::Area::new(egui::Id::new("right-dock-flyout-area")).order(egui::Order::Foreground).fixed_pos(pos2(x, y)).constrain(true).show(ctx, |ui| {
        ui.set_width(width);
        ui.set_max_height(max_height);
        egui::Frame::NONE
            .fill(t.chrome)
            .stroke(Stroke::new(1.0, t.card_border))
            .corner_radius(super::chrome_panel_radius(t))
            .shadow(egui::Shadow { offset: [0, 6], blur: 20, spread: 0, color: t.shadow })
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let title = group.tabs(t.pro).first().map_or_else(|| tl!("Panels").to_owned(), |name| tl!(name).to_owned());
                    ui.label(egui::RichText::new(title).strong().color(t.text));
                    ui.with_layout(crate::direction::trailing(), |ui| {
                        if icons::button(ui, "x", 24.0, false, tl!("Close")).clicked() {
                            close = true;
                        }
                    });
                });
                ui.add_space(4.0);
                ui.separator();
                let mut tabs = app.ui.dock.visible_tabs(group, t.pro);
                if tabs.is_empty() {
                    tabs = group.tabs(t.pro).iter().copied().enumerate().collect();
                }
                let labels: Vec<&str> = tabs.iter().map(|(_, name)| tl!(name)).collect();
                let current = *group.tab_mut(&mut app.ui.dock_tabs);
                let mut selected = tabs.iter().position(|(index, _)| *index == current).unwrap_or(0);
                // The strip row, laid out for `pill_tabs` like the Properties card does.
                let row = ui.available_rect_before_wrap();
                let strip_h = 20.0;
                let bar = Rect::from_min_max(row.min, pos2(row.right(), row.min.y + strip_h));
                let context = crate::tab_strip::pill_tabs(ui, egui::Id::new("right-dock-flyout-tabs").with(group.key()), bar, &labels, &mut selected);
                ui.add_space(6.0);
                crate::widgets::hairline(ui);
                ui.add_space(6.0);
                if let Some(&(tab, _)) = tabs.get(selected) {
                    if group.scrolls_itself(tab) {
                        dock_body(app, ui, group, tab);
                    } else {
                        egui::ScrollArea::vertical().id_salt(("rail-flyout", group.key())).show(ui, |ui| dock_body(app, ui, group, tab));
                    }
                }
                if let Some(&(tab, _)) = tabs.get(selected) {
                    *group.tab_mut(&mut app.ui.dock_tabs) = tab;
                    app.ui.dock.show_tab(group, tab, t.pro);
                }
                if let Some(crate::tab_strip::TabContextAction::Close(i)) = context
                    && let Some(&(tab, _)) = tabs.get(i)
                {
                    crate::dock::close_tab(app, group, tab, false);
                }
            });
    });
    let _ = area;
    let outside_click = ctx.input(|input| {
        input.pointer.any_pressed() && input.pointer.interact_pos().is_some_and(|pointer| !area.response.rect.contains(pointer) && !anchor.contains(pointer))
    });
    if close || outside_click || ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
        ctx.data_mut(|data| data.remove::<crate::dock::Group>(rail_flyout_id()));
    }
}

/// The dock's narrowest width (points). It is the right dock, on the left in a right-to-left UI.
pub(crate) const DOCK_MIN_WIDTH: f32 = 250.0;

/// What the dock leaves the toolbar and the canvas when it is widest (points).
const DOCK_LEAVES: f32 = 400.0;

/// The dock's width range (points) in a window `window_width` wide: from [`DOCK_MIN_WIDTH`] to
/// everything but [`DOCK_LEAVES`], and never capped below the old 520, so two-column layouts can
/// be as wide as the window allows (#7896).
pub(crate) fn dock_width_range(window_width: f32) -> std::ops::RangeInclusive<f32> {
    let max = if window_width.is_finite() { (window_width - DOCK_LEAVES).max(520.0) } else { 520.0 };
    DOCK_MIN_WIDTH..=max
}

pub(crate) fn dock_width_id() -> egui::Id {
    egui::Id::unique("dock-width-request")
}

/// Set the right dock's width on the next frame (`ui.set {dockWidth}`); clamped to its range.
pub fn request_dock_width(ctx: &egui::Context, w: f32) {
    let range = dock_width_range(ctx.content_rect().width());
    let w = if w.is_finite() { w.clamp(*range.start(), *range.end()) } else { *range.start() };
    ctx.data_mut(|d| d.insert_temp(dock_width_id(), w));
}

/// One dock group's tab content; `panel_docking` bounds it and scrolls it when it's taller.
pub(crate) fn dock_body(app: &mut PhotocraftApp, ui: &mut egui::Ui, group: crate::dock::Group, tab: usize) {
    use crate::dock::Group;
    let pro = Tokens::get(ui.ctx()).pro;
    match (group, tab) {
        (Group::Color, 2) => crate::preset_panels::gradients_panel(app, ui),
        (Group::Color, 3) => crate::preset_panels::patterns_panel(app, ui),
        // The Color tab is the same in every theme, in the mode picked from its panel menu
        // (#6000); only the tab order differs between themes.
        (Group::Color, tab) if group.tabs(pro).get(tab) == Some(&"Color") => crate::color_panel_ui::panel(app, ui),
        (Group::Color, _) => crate::swatches_ui::panel(app, ui),
        (Group::Properties, 0) => properties_body(app, ui),
        (Group::Properties, _) => adjustments_grid(app, ui),
        (Group::Character, tab) => crate::type_tool::character_panel(app, ui, tab == 1),
        (Group::Navigator, 0) => navigator(app, ui),
        (Group::Navigator, 1) => crate::histogram_panel::show(app, ui),
        (Group::Navigator, _) => info_panel(app, ui),
        (Group::History, 0) => history(app, ui),
        (Group::History, 1) => crate::actions::panel(app, ui),
        (Group::History, _) => crate::comps_ui::panel(app, ui),
        (Group::Layers, 0) => layers(app, ui),
        (Group::Layers, 1) => channels(app, ui),
        (Group::Layers, _) => crate::vector_ui::paths_panel(app, ui),
        (Group::Brushes, _) => crate::brushes_tab::show(app, ui),
    }
}
