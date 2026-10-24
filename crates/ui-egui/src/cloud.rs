//! EMBED-DESIGN-CLOUD B1/B3 — the embedding host's cloud library.
//!
//! The web shell's host bridge installs a [`CloudLibrary`] into
//! [`crate::Services::cloud`] while a same-origin parent page embeds the
//! editor. While one is connected, the File menu grows the dynamic
//! cloud-library entries (Open from Library ▸, Save to Library — the
//! openRecent insertion pattern) and every save the app writes is routed to
//! the host by the shell's `write` service (the library is the save
//! target). A standalone editor never sees any of this: `Services::cloud`
//! stays `None`, so this module's menu rows are never built.
//!
//! The types are platform-neutral on purpose: the menu code, the save-all
//! loop and the save-name logic run (and are tested) on every target; only
//! the bridge itself lives in the web shell.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::PhotocraftApp;

/// One library asset the host lists for File ▸ Open from Library.
/// `kind` and `size` are the host's display strings ("image" | "psd", "64×64 · 2 layers").
#[derive(Clone, Debug, PartialEq)]
pub struct CloudEntry {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub size: String,
}

/// The host-side cloud library handle (Clone — all state behind Arc).
///
/// `open` asks the host for an asset's bytes; the host answers through the
/// normal open path (the shell's inbox), so the asset opens as a new
/// document tab. The closure re-fetches the browser window on each call
/// (the `download` precedent) so it captures nothing non-Send.
///
/// PHASE-B identity state (shared with the web-shell bridge):
/// `identity` maps doc name → originating asset id (first-wins, pruned to
/// the live docs, re-keyed to the save name on identity writes);
/// `pending` is the consume-on-write slot the save paths arm so the
/// bridge can echo the id on the `saved` message (an update-in-place,
/// not a sibling mint, on the host side).
pub struct CloudLibrary {
    /// The listing the host last pushed (File ▸ Open from Library's rows).
    pub entries: Arc<Mutex<Vec<CloudEntry>>>,
    /// Ask the host to open an asset by id.
    pub open: Arc<dyn Fn(&str) + Send + Sync>,
    /// doc name → the library asset the doc was opened from.
    pub identity: Arc<Mutex<HashMap<String, String>>>,
    /// The asset id armed for the next synchronous write.
    pub pending: Arc<Mutex<Option<String>>>,
}

impl Clone for CloudLibrary {
    fn clone(&self) -> Self {
        Self {
            entries: Arc::clone(&self.entries),
            open: Arc::clone(&self.open),
            identity: Arc::clone(&self.identity),
            pending: Arc::clone(&self.pending),
        }
    }
}

impl CloudLibrary {
    /// The current listing (a poisoned lock reads as empty — a stale menu
    /// beats a panic).
    pub fn listing(&self) -> Vec<CloudEntry> {
        self.entries.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// PHASE-B — the asset a document was opened from, if any.
    pub fn identity_of(&self, doc_name: &str) -> Option<String> {
        self.identity.lock().unwrap_or_else(|e| e.into_inner()).get(doc_name).cloned()
    }

    /// PHASE-B — arm the pending slot for a document's next synchronous
    /// write (no-op without an identity — the write then mints, today's
    /// behavior). LOAD-BEARING: the web shell installs neither `save_file`
    /// nor `background_jobs`, so `save_as_with` writes inline on the UI
    /// thread and the slot is consumed within the same call (see
    /// EMBED-DESIGN-PHASEB F2/F8).
    pub fn arm_save(&self, doc_name: &str) {
        let id = self.identity_of(doc_name);
        *self.pending.lock().unwrap_or_else(|e| e.into_inner()) = id;
    }

    /// PHASE-B — clear the armed slot (a save path that failed before its
    /// write; the next write must never echo a stale id).
    pub fn clear_pending(&self) {
        *self.pending.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
}

/// The id prefix of the File ▸ Open from Library submenu rows
/// (`file.cloudLibrary.{asset id}`).
pub const LIBRARY_ITEM: &str = "file.cloudLibrary.";

/// File ▸ Save to Library — the menu id (save-ALL semantics: identical to
/// the host's "Save to Library (N)" button).
pub const SAVE_TO_LIBRARY: &str = "file.saveToLibrary";

/// The save name for one document in a save-all: a layered source keeps
/// its own form (.psd/.psb/.pcraft), a Paint.NET/Krita import maps to the
/// format that keeps its features, everything else saves as the editable
/// .psd source — and same-stem collisions disambiguate with " 2", " 3"…
/// so two documents never write one name (a duplicate mint AND the
/// path-detach re-dirty mid-loop).
pub fn cloud_save_name(doc_name: &str, used: &mut HashSet<String>) -> String {
    let ext = std::path::Path::new(doc_name)
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    let target = match ext.as_str() {
        "psb" | "psd" | "pcraft" | "ora" => ext,
        "pdn" => "pcraft".to_string(),
        "kra" | "krz" => "ora".to_string(),
        _ => "psd".to_string(),
    };
    let stem = std::path::Path::new(doc_name)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| doc_name.to_string());
    let base = format!("{stem}.{target}");
    if !used.contains(&base) {
        return base;
    }
    for n in 2.. {
        let candidate = format!("{stem} {n}.{target}");
        if !used.contains(&candidate) {
            return candidate;
        }
    }
    unreachable!("the loop returns once a free name is found")
}

/// The layered identity formats — the saves that carry a doc's
/// source-of-truth (and never park in a format prompt, so their writes
/// land inline: EMBED-DESIGN-PHASEB F2).
fn is_identity_format(ext: &str) -> bool {
    matches!(ext, "psd" | "psb" | "pcraft" | "ora")
}

/// PHASE-B — arm the ACTIVE doc's identity for a save to `path`, but only
/// when the path's format is one of the layered identity formats: they
/// never park in a format prompt, so `save_as_with`/`save_in_place` write
/// inline and the slot is consumed in the same call (EMBED-DESIGN-PHASEB
/// F2). A flat-format save (jpeg/png/tga…) is an export-shaped copy — it
/// correctly MINTS, so it must never consume an identity.
pub fn arm_save_for_path(app: &PhotocraftApp, path: &str) {
    let Some(cloud) = app.services.cloud.as_ref() else { return };
    let ext = std::path::Path::new(path)
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if !is_identity_format(&ext) {
        return;
    }
    let Some(state) = app.session.active() else { return };
    let name = state.doc.name.clone();
    cloud.arm_save(&name);
}

/// PHASE-B — clear the armed slot (a save path that failed before its
/// write; the next write must never echo a stale id).
pub fn clear_pending(app: &PhotocraftApp) {
    if let Some(cloud) = app.services.cloud.as_ref() {
        cloud.clear_pending();
    }
}

/// File ▸ Save to Library (EMBED-DESIGN-CLOUD B3): activate every open
/// document in order and save it under its cloud name — an explicit path
/// skips the save dialog, and the shell's `write` service hands the bytes
/// to the connected host. Per-document failures are collected into the
/// status line, never fatal to the rest; returns (saved, failed).
pub fn save_all_to_library(app: &mut PhotocraftApp) -> (usize, usize) {
    let names: Vec<String> = app.session.documents().iter().map(|d| d.doc.name.clone()).collect();
    let mut used: HashSet<String> = HashSet::new();
    let (mut saved, mut failed) = (0usize, 0usize);
    for (i, doc_name) in names.into_iter().enumerate() {
        let path = cloud_save_name(&doc_name, &mut used);
        used.insert(path.clone());
        if !app.session.set_active(i) {
            failed += 1;
            continue;
        }
        // PHASE-B: arm this doc's identity so the write echoes the asset id
        // (the host updates the asset in place). The doc name here is the
        // PRE-save name — the identity map still carries it (the re-key
        // happens inside the write itself).
        if let Some(cloud) = app.services.cloud.as_ref() {
            cloud.arm_save(&doc_name);
        }
        match app.save_as_with(Some(path), crate::FormatChoices::default()) {
            Ok(_) => saved += 1,
            Err(e) => {
                failed += 1;
                app.ui.status = e;
                app.ui.status_error = true;
                if let Some(cloud) = app.services.cloud.as_ref() {
                    cloud.clear_pending();
                }
            }
        }
    }
    (saved, failed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn used(names: &[&str]) -> HashSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn identity_formats_are_the_layered_sources() {
        // The F2 gate: only layered formats arm the identity echo — a flat
        // save (jpeg/png/tga…) is an export-shaped copy and must mint.
        assert!(is_identity_format("psd"));
        assert!(is_identity_format("psb"));
        assert!(is_identity_format("pcraft"));
        assert!(is_identity_format("ora"));
        assert!(!is_identity_format("png"));
        assert!(!is_identity_format("jpg"));
        assert!(!is_identity_format("tga"));
        assert!(!is_identity_format(""));
    }

    #[test]
    fn identity_state_arms_and_consumes() {
        // The CloudLibrary identity/pending laws (first-wins insert is the
        // bridge's arm; here: arm → consume → re-arm clears).
        let cloud = CloudLibrary {
            entries: Arc::default(),
            open: Arc::new(|_| {}),
            identity: Arc::new(
                [("photo.png".to_string(), "asset-7".to_string())].into_iter().collect(),
            ),
            pending: Arc::default(),
        };
        assert_eq!(cloud.identity_of("photo.png"), Some("asset-7".to_string()));
        assert_eq!(cloud.identity_of("other.psd"), None);
        cloud.arm_save("photo.png");
        assert_eq!(*cloud.pending.lock().unwrap(), Some("asset-7".to_string()));
        cloud.clear_pending();
        assert_eq!(*cloud.pending.lock().unwrap(), None);
        // Arming an identity-less doc is a no-op (the write mints).
        cloud.arm_save("other.psd");
        assert_eq!(*cloud.pending.lock().unwrap(), None);
    }

    #[test]
    fn layered_sources_keep_their_own_form() {
        assert_eq!(cloud_save_name("story-layers.psd", &mut used(&[])), "story-layers.psd");
        // A .psb document stays real PSB — never folded into a same-stem .psd.
        assert_eq!(cloud_save_name("big-canvas.psb", &mut used(&[])), "big-canvas.psb");
    }

    #[test]
    fn flat_and_imported_names_map_to_the_editable_source() {
        assert_eq!(cloud_save_name("photo.png", &mut used(&[])), "photo.psd");
        assert_eq!(cloud_save_name("Untitled-1", &mut used(&[])), "Untitled-1.psd");
        assert_eq!(cloud_save_name("paint.pdn", &mut used(&[])), "paint.pcraft");
        assert_eq!(cloud_save_name("draw.kra", &mut used(&[])), "draw.ora");
    }

    #[test]
    fn same_stem_documents_disambiguate() {
        // The E2E pair: story-layers.psd + story-layers.psb must never
        // collapse onto one name (duplicate mint + detach re-dirty).
        let mut used = HashSet::new();
        let a = cloud_save_name("story-layers.psd", &mut used);
        used.insert(a.clone());
        let b = cloud_save_name("story-layers.psb", &mut used);
        assert_eq!(a, "story-layers.psd");
        assert_eq!(b, "story-layers.psb");
        // Two extension-less Untitled docs get distinct .psd names.
        let mut used = HashSet::new();
        let a = cloud_save_name("Untitled-1", &mut used);
        used.insert(a.clone());
        assert_eq!(cloud_save_name("Untitled-1", &mut used), "Untitled-1 2.psd");
    }

    #[test]
    fn save_all_saves_every_document_and_reports_counts() {
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
        // Two tiny documents with different stems.
        for (name, w, h) in [("a.png", 4u32, 4u32), ("b.png", 3u32, 5u32)] {
            let doc = photocraft_doc::Document::new(
                name,
                photocraft_geom::Size::new(w, h),
                photocraft_doc::ColorMode::Rgb,
                photocraft_doc::SampleType::U8,
            );
            app.session.add_document(doc, None);
        }
        // The write service collects what a save wrote.
        let written: Arc<Mutex<Vec<String>>> = Arc::default();
        let sink = written.clone();
        app.services.write = Some(Box::new(move |path: &str, _bytes: &[u8]| {
            sink.lock().unwrap_or_else(|e| e.into_inner())
                .push(std::path::Path::new(path).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| path.into()));
            Ok(())
        }));
        app.services.export = Some(Box::new(|_doc: &photocraft_doc::Document, _path: &str, _settings: &crate::ExportSettings| {
            // The format follows the path's extension; a real encode is not
            // needed to observe the names and the counts.
            Ok((Vec::new(), Vec::new()))
        }));
        let (saved, failed) = save_all_to_library(&mut app);
        assert_eq!((saved, failed), (2, 0));
        let got = written.lock().unwrap_or_else(|e| e.into_inner()).clone();
        assert_eq!(got, vec!["a.psd".to_string(), "b.psd".to_string()], "{got:?}");
        // Every save left the documents clean (confirmed writes) and both
        // documents stayed open.
        assert_eq!(app.session.documents().len(), 2);
    }
}
