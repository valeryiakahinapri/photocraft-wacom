//! Panel rail tests: the visibility popover on top of the rail, the collapsed-dock's rotated
//! labels, and the floating peek they open.

use egui::{Key, Modifiers, PointerButton, Pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use serde_json::json;

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

/// Visual fixtures (PNG snapshots) land in `$PHOTOCRAFT_PANEL_RAIL_FIXTURES` when set.
fn shoot(h: &Harness<'static, PhotocraftApp>, name: &str) {
    if let Some(directory) = std::env::var_os("PHOTOCRAFT_PANEL_RAIL_FIXTURES").map(std::path::PathBuf::from) {
        std::fs::create_dir_all(&directory).unwrap();
        h.render().unwrap().save(directory.join(format!("panel-rail-{name}.png"))).unwrap();
    }
}

/// The popover lists every group, hides and shows the dock column, and the collapsed rail
/// carries one rotated label per group still shown.
#[test]
fn the_rail_toggle_hides_and_shows_the_dock_column() {
    let mut h = harness(app(), vec2(1200.0, 800.0), ThemeKind::Studio);
    shoot(&h, "icons");
    assert!(h.state().ui.panels.dock);
    assert!(h.state().ui.panels.color && h.state().ui.panels.properties && h.state().ui.panels.layers);
    assert!(!h.state().ui.panels.navigator && !h.state().ui.panels.history && !h.state().ui.panels.brushes && !h.state().ui.panels.character);
    // The icons mode shows all seven groups on the rail.
    for g in Group::ALL {
        assert!(h.query_by_role_and_label(egui::accesskit::Role::Button, g.label()).is_some(), "no rail icon for {g:?}");
    }
    // The toggle's popover: the dock row plus one checkbox per group.
    h.get_by_label("Show/Hide Panels").click();
    h.run_steps(2);
    shoot(&h, "popover");
    for g in Group::ALL {
        assert!(h.query_by_role_and_label(egui::accesskit::Role::CheckBox, g.label()).is_some(), "no popover row for {g:?}");
    }
    h.get_by_label("Hide Dock").click();
    h.run_steps(3);
    assert!(!h.state().ui.panels.dock, "the row hides the dock column");
    shoot(&h, "labels");
    // Collapsed: only the shown groups keep a (rotated label) chip; the icons are gone.
    for g in [Group::Color, Group::Properties, Group::Layers] {
        assert!(h.query_by_role_and_label(egui::accesskit::Role::Button, g.label()).is_some(), "no label chip for the shown {g:?}");
    }
    for g in [Group::Navigator, Group::History, Group::Brushes, Group::Character] {
        assert!(h.query_by_role_and_label(egui::accesskit::Role::Button, g.label()).is_none(), "hidden {g:?} still has a chip");
    }
    // Back the other way: the popover's row reads Show Dock now, and the icons return.
    h.get_by_label("Show/Hide Panels").click();
    h.run_steps(2);
    h.get_by_label("Show Dock").click();
    h.run_steps(3);
    assert!(h.state().ui.panels.dock);
    for g in Group::ALL {
        assert!(h.query_by_role_and_label(egui::accesskit::Role::Button, g.label()).is_some(), "no rail icon for {g:?} after re-showing");
    }
}

/// A rail label opens its group as a floating peek; Escape, the close box, a click outside and
/// the dock coming back all end it, and `window.panel.peek` drives the same state.
#[test]
fn a_rail_label_opens_a_floating_peek_that_follows_focus() {
    let mut h = harness(app(), vec2(1200.0, 800.0), ThemeKind::Studio);
    h.state_mut().run("window.toggle.dock", json!({})).unwrap();
    h.run_steps(3);
    assert!(!h.state().ui.panels.dock);
    h.get_by_label("Layers").click();
    h.run_steps(3);
    assert_eq!(h.state().ui.panel_peek.as_ref().map(|p| p.group), Some(Group::Layers));
    assert!(h.ctx.memory(|m| m.area_rect(peek_id())).is_some(), "the peek window is drawn");
    assert!(h.query_by_label("Close").is_some(), "the peek carries a close box");
    shoot(&h, "peek");
    // A press outside the window and the rail closes the peek, like dock.rs's collapsed-group
    // peek; the label chip keeps its accent tint while open.
    press(&mut h, Pos2::new(60.0, 400.0));
    assert!(h.state().ui.panel_peek.is_none(), "a click outside closes the peek");
    // The command reaches the same state (automation parity), without an anchor.
    h.state_mut().run("window.panel.peek", json!({"panel": "history", "show": true})).unwrap();
    h.run_steps(3);
    assert_eq!(h.state().ui.panel_peek.as_ref().map(|p| p.group), Some(Group::History));
    shoot(&h, "peek-history");
    h.event(egui::Event::Key { key: Key::Escape, pressed: true, modifiers: Modifiers::NONE });
    h.run_steps(2);
    assert!(h.state().ui.panel_peek.is_none(), "Escape closes the peek");
    // The close box ends it too, and the dock coming back ends it always.
    h.state_mut().run("window.panel.peek", json!({"panel": "layers"})).unwrap();
    h.run_steps(3);
    h.get_by_label("Close").click();
    h.run_steps(2);
    assert!(h.state().ui.panel_peek.is_none(), "the close box closes the peek");
    h.state_mut().run("window.panel.peek", json!({"panel": "layers"})).unwrap();
    h.run_steps(3);
    h.state_mut().run("window.toggle.dock", json!({})).unwrap();
    h.run_steps(3);
    assert!(h.state().ui.panels.dock && h.state().ui.panel_peek.is_none(), "the dock returning ends the peek");
    // A saved layout applied over an open peek ends it as well (dock::apply).
    h.state_mut().run("window.toggle.dock", json!({})).unwrap();
    h.run_steps(2);
    h.state_mut().run("window.panel.peek", json!({"panel": "layers"})).unwrap();
    h.run_steps(3);
    assert!(h.state().ui.panel_peek.is_some());
    let saved = serde_json::to_value(&h.state().ui).unwrap();
    crate::dock::apply(h.state_mut(), &saved);
    h.run_steps(2);
    assert!(h.state().ui.panel_peek.is_none(), "dock::apply ends a peek");
}

/// `set_group_shown` covers every group in the legacy dock and in a customized layout, and no
/// panel command resurrects a hidden dock column (panel_docking used to re-assert it).
#[test]
fn set_group_shown_covers_every_group_in_both_dock_modes() {
    let mut a = app();
    for g in Group::ALL {
        set_group_shown(&mut a, g, true);
        assert!(g.shown(&a.ui.panels), "{g:?} not shown");
        set_group_shown(&mut a, g, false);
        assert!(!g.shown(&a.ui.panels), "{g:?} not hidden");
    }
    // A customized layout: group-level activate/close through the shared docking tree.
    a.run("window.panel.float", json!({"panel": "layers", "x": 320.0, "y": 120.0, "width": 280.0, "height": 300.0})).unwrap();
    assert!(a.ui.docking.is_some());
    assert!(a.ui.panels.layers);
    a.run("window.toggle.dock", json!({})).unwrap();
    assert!(!a.ui.panels.dock);
    // What crop_ui.rs fires on a crop overlay must not bring the column back (#P0-2).
    a.run("window.panel.layers", json!({})).unwrap();
    assert!(!a.ui.panels.dock, "a panel command resurrected the hidden dock");
    set_group_shown(&mut a, Group::Layers, false);
    assert!(!a.ui.panels.layers, "the customized group did not close");
    set_group_shown(&mut a, Group::Layers, true);
    assert!(a.ui.panels.layers, "the customized group did not come back");
    assert!(!a.ui.panels.dock, "restoring a panel resurrected the hidden dock");
}
