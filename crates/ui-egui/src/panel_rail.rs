//! The right dock's panel rail: the narrow strip of panel-group buttons beside the canvas.
//!
//! Icons while the dock column shows; the rail stays reachable when the dock itself is hidden
//! (`window.toggle.dock`) so hidden panels can be brought back, and it hides with the whole chrome
//! when Tab does or an embedding app turns `panels.rail` off. Extracted from `panels::right_dock`
//! so rail behaviours (visibility toggling, collapsed-dock labels, floating peeks) can live
//! beside the dock machinery without growing the panels module.

use egui::{pos2, Stroke};

use crate::PhotocraftApp;
use crate::dock::Group;
use crate::state::Panels;
use crate::theme::Tokens;

/// Draw the rail. `customized` is whether the shared (dragged-out) panel layout owns visibility.
pub(crate) fn rail(app: &mut PhotocraftApp, ui: &mut egui::Ui, p: &Panels, customized: bool) {
    let t = Tokens::get(ui.ctx());
    let (rw, rb) = if t.pro { (36.0, 28.0) } else { (44.0, 32.0) };
    crate::direction::end_panel("rail")
        .resizable(false)
        .exact_size(rw)
        .frame(
            egui::Frame::NONE
                .fill(t.chrome)
                .corner_radius(crate::panels::chrome_panel_radius(&t))
                .inner_margin(egui::Margin::symmetric(4, 8)),
        )
        .show(ui, |ui| {
            // The separator is on the canvas side.
            let r = ui.max_rect();
            let x = if crate::direction::is_rtl() { r.right() + 6.0 } else { r.left() - 6.0 };
            ui.painter().line_segment([pos2(x, r.top() - 8.0), pos2(x, r.bottom() + 8.0)], Stroke::new(1.0, t.separator));
            ui.spacing_mut().item_spacing.y = 4.0;
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
                let on = if customized { crate::panel_docking::rail_on(app, g) } else { g.shown(p) && !(docked && app.ui.dock.is_collapsed(g)) };
                if crate::icons::rail_button(ui, icon, rb, on, name).clicked() {
                    if customized {
                        crate::panel_docking::rail_click(app, g);
                    } else {
                        crate::dock::rail_click(app, g, docked);
                    }
                }
            }
        });
}
