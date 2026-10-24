//! Panel rail tests: the top toggle, the constant icon list in both dock states, and the
//! floating peek the collapsed-dock icons open.

use egui::{Key, Modifiers, PointerButton, Pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use serde_json::{Value, json};

use super::*;
use crate::dock::Group;
use crate::theme::ThemeKind;

fn app() -> PhotocraftApp {
    let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
    app.run("file.new", json!({"width": 200, "height": 150})).unwrap();
    app
}

/// The dock_tests harness plus the floating peek window the full layout draws after it.
fn harness(app: PhotocraftApp, size: egui::Vec2, theme: ThemeKind) -> Harness<'static, PhotocraftApp> {
    let mut h = Harness::builder().with_size(size).with_step_dt(1.0 / 60.0).build_ui_state(
        |ui, app: &mut PhotocraftApp| {
            let ctx = ui.ctx().clone();
            if !ctx.fonts(|f| f.families().contains(&egui::FontFamily::Name("medium".into()))) {
                return;
            }
            crate::panels::right_dock(app, ui);
            egui::CentralPanel::default().show(ui, |_| {});
            crate::panel_rail::peek_window(app, &ctx);
        },
        app,
    );
    PhotocraftApp::setup_context(&h.ctx, theme);
    h.state_mut().ui.theme = theme;
    h.run_steps(4);
    h
}

fn press(h: &mut Harness<'static, PhotocraftApp>, at: Pos2) {
    h.event(egui::Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE });
    h.run_steps(1);
    h.event(egui::Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE });
    h.run_steps(2);
}

/// UI-level ids (window.toggle.*, window.panel.peek…) dispatch through menus::invoke — the
/// menu bar and keybinding path; app.run reaches only the engine, which catalogs but does
/// not execute them.
fn ui_run(h: &mut Harness<'static, PhotocraftApp>, id: &str, params: Value) -> Value {
    let ctx = h.ctx.clone();
    crate::menus::invoke(h.state_mut(), &ctx, id, params).expect(id)
}

/// The same routing for a bare app (no harness): the engine cannot execute UI-level ids.
fn app_ui_run(app: &mut PhotocraftApp, id: &str, params: Value) -> Value {
    let ctx = egui::Context::default();
    crate::menus::invoke(app, &ctx, id, params).expect(id)
}

/// The rail icon button of `g` (the peek window's title carries the same label as a plain
/// text widget, so pin the role to the button).
fn icon_button<'a>(h: &'a Harness<'static, PhotocraftApp>, g: Group) -> egui_kittest::kittest::Query<'a, 'static> {
    h.query_by_role_and_label(egui::accesskit::Role::Button, g.label()).unwrap_or_else(|| panic!("no rail icon for {g:?}"))
}

/// Visual fixtures (PNG snapshots) land in `$PHOTOCRAFT_PANEL_RAIL_FIXTURES` when set.
fn shoot(h: &mut Harness<'static, PhotocraftApp>, name: &str) {
    if let Some(directory) = std::env::var_os("PHOTOCRAFT_PANEL_RAIL_FIXTURES").map(std::path::PathBuf::from) {
        std::fs::create_dir_all(&directory).unwrap();
        h.render().unwrap().save(directory.join(format!("panel-rail-{name}.png"))).unwrap();
    }
}

/// The toggle on top of the rail hides and shows the dock column itself, and the icon list is
/// the same constant rail in both states (the owner's mockup grammar — no popover, no labels).
#[test]
fn the_rail_toggle_hides_and_shows_the_dock_column() {
    let mut h = harness(app(), vec2(1200.0, 800.0), ThemeKind::Studio);
    shoot(&mut h, "icons");
    assert!(h.state().ui.panels.dock);
    assert!(h.state().ui.panels.color && h.state().ui.panels.properties && h.state().ui.panels.layers);
    assert!(!h.state().ui.panels.navigator && !h.state().ui.panels.history && !h.state().ui.panels.brushes && !h.state().ui.panels.character);
    // The expanded rail shows all seven groups.
    for g in Group::ALL {
        assert!(h.query_by_role_and_label(egui::accesskit::Role::Button, g.label()).is_some(), "no rail icon for {g:?}");
    }
    // One press on the toggle hides the column directly.
    h.get_by_label("Show/Hide Panels").click();
    h.run_steps(3);
    assert!(!h.state().ui.panels.dock, "the toggle hides the dock column");
    shoot(&mut h, "collapsed");
    // The collapsed rail is the same icon list — every group stays clickable, shown or not.
    for g in Group::ALL {
        assert!(h.query_by_role_and_label(egui::accesskit::Role::Button, g.label()).is_some(), "the collapsed rail lost the {g:?} icon");
    }
    // And back: the column returns, the icons never moved.
    h.get_by_label("Show/Hide Panels").click();
    h.run_steps(3);
    assert!(h.state().ui.panels.dock);
    for g in Group::ALL {
        assert!(h.query_by_role_and_label(egui::accesskit::Role::Button, g.label()).is_some(), "no rail icon for {g:?} after re-showing");
    }
}

/// A collapsed-rail icon opens its group as a floating peek — a hidden group's icon included
/// (the rail is the Window-menu-style affordance); re-clicking the lit icon, Escape, the close
/// box, a click outside and the dock coming back all end it, and `window.panel.peek` drives
/// the same state.
#[test]
fn a_rail_icon_opens_a_floating_peek_that_follows_focus() {
    let mut h = harness(app(), vec2(1200.0, 800.0), ThemeKind::Studio);
    ui_run(&mut h, "window.toggle.dock", json!({}));
    h.run_steps(3);
    assert!(!h.state().ui.panels.dock);
    // A hidden group's icon still peeks (Character is off by default).
    icon_button(&h, Group::Character).click();
    h.run_steps(3);
    assert_eq!(h.state().ui.panel_peek.as_ref().map(|p| p.group), Some(Group::Character));
    assert!(h.ctx.memory(|m| m.area_rect(peek_window_id(Group::Character))).is_some(), "the hidden group's peek window is drawn");
    h.event(egui::Event::Key { key: Key::Escape, physical_key: Some(Key::Escape), pressed: true, repeat: false, modifiers: Modifiers::NONE });
    h.run_steps(2);
    assert!(h.state().ui.panel_peek.is_none(), "Escape closes the peek");
    // The shown group's icon opens its peek, with a close box.
    icon_button(&h, Group::Layers).click();
    h.run_steps(3);
    assert_eq!(h.state().ui.panel_peek.as_ref().map(|p| p.group), Some(Group::Layers));
    assert!(h.ctx.memory(|m| m.area_rect(peek_window_id(Group::Layers))).is_some(), "the peek window is drawn");
    assert!(h.query_by_label("Close").is_some(), "the peek carries a close box");
    shoot(&mut h, "peek");
    // Clicking the lit icon again closes it (the no-params `window.panel.peek` parity).
    icon_button(&h, Group::Layers).click();
    h.run_steps(2);
    assert!(h.state().ui.panel_peek.is_none(), "re-clicking the peeked icon closes the peek");
    // A press outside the window and the rail closes the peek, like dock.rs's collapsed-group
    // peek.
    icon_button(&h, Group::Layers).click();
    h.run_steps(3);
    press(&mut h, Pos2::new(60.0, 400.0));
    assert!(h.state().ui.panel_peek.is_none(), "a click outside closes the peek");
    // The command reaches the same state (automation parity), without an anchor.
    ui_run(&mut h, "window.panel.peek", json!({"panel": "history", "show": true}));
    h.run_steps(3);
    assert_eq!(h.state().ui.panel_peek.as_ref().map(|p| p.group), Some(Group::History));
    shoot(&mut h, "peek-history");
    h.event(egui::Event::Key { key: Key::Escape, physical_key: Some(Key::Escape), pressed: true, repeat: false, modifiers: Modifiers::NONE });
    h.run_steps(2);
    assert!(h.state().ui.panel_peek.is_none(), "Escape closes the peek");
    // The close box ends it too, and the dock coming back ends it always.
    ui_run(&mut h, "window.panel.peek", json!({"panel": "layers"}));
    h.run_steps(3);
    h.get_by_label("Close").click();
    h.run_steps(2);
    assert!(h.state().ui.panel_peek.is_none(), "the close box closes the peek");
    ui_run(&mut h, "window.panel.peek", json!({"panel": "layers"}));
    h.run_steps(3);
    ui_run(&mut h, "window.toggle.dock", json!({}));
    h.run_steps(3);
    assert!(h.state().ui.panels.dock && h.state().ui.panel_peek.is_none(), "the dock returning ends the peek");
    // Tab hiding the chrome with the dock ends a peek too (the peek is a hidden-dock
    // affordance; the guard in peek_window clears it however the dock comes back).
    ui_run(&mut h, "window.toggle.dock", json!({}));
    h.run_steps(2);
    ui_run(&mut h, "window.panel.peek", json!({"panel": "layers"}));
    h.run_steps(3);
    assert!(h.state().ui.panel_peek.is_some());
    ui_run(&mut h, "window.togglePanels", json!({}));
    h.run_steps(3);
    assert!(h.state().ui.panel_peek.is_none(), "Tab's hidden chrome ends the peek");
    // A saved layout applied over an open peek ends it as well (dock::apply).
    ui_run(&mut h, "window.toggle.dock", json!({}));
    h.run_steps(2);
    ui_run(&mut h, "window.panel.peek", json!({"panel": "layers"}));
    h.run_steps(3);
    assert!(h.state().ui.panel_peek.is_some());
    let saved = serde_json::to_value(&h.state().ui).unwrap();
    crate::dock::apply(h.state_mut(), &saved);
    h.run_steps(2);
    assert!(h.state().ui.panel_peek.is_none(), "dock::apply ends a peek");
}

/// No panel command resurrects a hidden dock column (panel_docking used to re-assert it).
#[test]
fn panel_commands_do_not_resurrect_the_hidden_dock() {
    let mut a = app();
    // A customized layout: group-level activate/close through the shared docking tree.
    app_ui_run(&mut a, "window.panel.float", json!({"panel": "layers", "x": 320.0, "y": 120.0, "width": 280.0, "height": 300.0}));
    assert!(a.ui.docking.is_some());
    assert!(a.ui.panels.layers);
    app_ui_run(&mut a, "window.toggle.dock", json!({}));
    assert!(!a.ui.panels.dock);
    // An explicit show of a floating member must not bring the column back, and neither may the
    // no-params activate crop_ui fires (#P0-2 both ways: change() and apply_action()).
    app_ui_run(&mut a, "window.panel.layers", json!({"show": true}));
    assert!(!a.ui.panels.dock, "a panel command resurrected the hidden dock");
    app_ui_run(&mut a, "window.panel.layers", json!({}));
    assert!(!a.ui.panels.dock, "an activate resurrected the hidden dock");
}
