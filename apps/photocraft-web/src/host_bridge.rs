//! The embedding host's bridge (EMBED-DESIGN-CLOUD B0/B2/B4): a same-origin
//! parent page (the vendored editor inside its host app) drives the editor
//! and reads its state over `postMessage`. Inert when the page is
//! standalone (`window.parent === window`): every method is a no-op and
//! [`CloudLibrary`](photocraft_ui_egui::cloud::CloudLibrary) stays `None`,
//! so the site behaves exactly like upstream.
//!
//! The vocabulary mirrors the host's protocol module verbatim (the parse
//! guard there hard-rejects malformed members): every message carries
//! `channel: "photocraft"`.
//!
//! Host → editor: `open {files:[{name, blob}]}` (the same inbox the drop
//! path feeds — the app opens each as a document tab), `library.list
//! {assets:[{id,name,kind,size}]}` (the File ▸ Open from Library rows),
//! `save-request {verb:'save'|'save-all'}` (a command queue the shell's
//! tick drains, gated on no open dialog), `save-ack` (accepted, logged).
//!
//! Editor → host: `ready` (install; the bridge id + app version), `booted`
//! / `boot-failed` (the runner's outcome), `saved {file:{name,size,type},
//! blob}` (the WRITE service while connected — the library is the save
//! target, in a real Blob so the host's structured-clone guard accepts
//! it), `docs {names}` (the MDI truth, on change), `dirty {dirty}` (on
//! change), `esc` (every Escape the raw input sees — the host owns the
//! double-press law), `library.list.request` (right after `ready`),
//! `library.open {id}` (the File-menu dispatch asking for an asset's
//! bytes; the host answers with a standard `open`), and `save-all-done
//! {saved, failed}` (the terminal completion event after the save-all
//! loop).
//!
//! Prefs pre-seed: while embedded, the localStorage prefs deep-merge the
//! embed seed BEFORE the app loads them (the shared key stays intact for
//! standalone use of the same origin).

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use photocraft_ui_egui::cloud::{CloudEntry, CloudLibrary};
use photocraft_ui_egui::PhotocraftApp;
use wasm_bindgen::JsCast as _;

type Inbox = photocraft_ui_egui::Inbox;

/// The single message channel both sides check first.
const CHANNEL: &str = "photocraft";
/// The bridge id the host's `ready` parse guard reads.
const BRIDGE_ID: &str = "photocraft-bridge-v2";
/// The app version the host's `ready` parse guard reads.
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
/// The localStorage key the app's prefs service owns (web.rs).
const PREFS_KEY: &str = "photocraft.preferences";

/// A host command, drained by the shell's tick (needs `&mut app`).
#[derive(Clone, Copy, PartialEq)]
enum HostCommand {
    /// The standard File › Save flow (the host's single-doc verb).
    Save,
    /// Every open document, in order (the host's "Save to Library (N)").
    SaveAll,
}

struct Inner {
    connected: bool,
    /// Filled at install (the app-creation closure owns the inbox).
    inbox: RefCell<Option<Inbox>>,
    /// Filled at install — wakes the frame loop when a message arrives.
    ctx: RefCell<Option<egui::Context>>,
    /// The host's last pushed listing (the File-menu submenu source).
    entries: Arc<Mutex<Vec<CloudEntry>>>,
    /// The host's queued commands.
    commands: RefCell<VecDeque<HostCommand>>,
    /// The last posted per-doc state vector (change detection): the D2
    /// `states` payloads — [{name, dirty, assetId?}] — so a dirty flip or
    /// a PHASE-B identity re-key repushes, and nothing posts when nothing
    /// changed (the F6 no-op law extends to `states`).
    last_docs: RefCell<Vec<serde_json::Value>>,
    /// The last posted dirty flag (change detection).
    last_dirty: RefCell<bool>,
    /// PHASE-B identity: doc name → the library asset it was opened from
    /// (first-wins on name collisions — a second same-named asset never
    /// steals another's identity; pruned to the live docs list on every
    /// docs change; re-keyed to the SAVE name when an identity write
    /// lands, because the engine renames the doc to the save name).
    identity: Arc<Mutex<HashMap<String, String>>>,
    /// PHASE-B pending: the asset id armed for the NEXT synchronous write
    /// (consume-on-write; armed ONLY on save paths that write inline —
    /// save-all per-doc and `file.save` with a known path — so exports,
    /// dialogs and Save-As can never observe an armed slot).
    pending: Arc<Mutex<Option<String>>>,
}

impl Inner {
    fn request_repaint(&self) {
        if let Some(ctx) = self.ctx.borrow().as_ref() {
            ctx.request_repaint();
        }
    }

    fn push_inbox(&self, name: &str, bytes: Vec<u8>) {
        if let Some(inbox) = self.inbox.borrow().as_ref() {
            inbox.lock().unwrap_or_else(|e| e.into_inner()).push((name.to_string(), bytes));
        }
    }

    /// Post one JSON message to the parent (same-origin target; numbers,
    /// strings, bools and arrays survive the round trip).
    fn post_json(&self, message: &serde_json::Value) {
        let text = message.to_string();
        if let Some(value) = js_sys::JSON::parse(&text).ok() {
            post_to_parent(&value);
        }
    }
}

/// The parent window (same-origin embed by design; `None` standalone —
/// `window.parent === window` means the site runs on its own).
fn parent_window() -> Option<web_sys::Window> {
    let window = web_sys::window()?;
    let parent = window.parent().ok().flatten();
    match parent {
        Some(parent) if parent != window => Some(parent),
        _ => None,
    }
}

/// `window.parent().postMessage(msg, <own origin>)` — the same-origin-strict
/// form: a cross-origin parent never receives our events.
fn post_to_parent(message: &wasm_bindgen::JsValue) {
    let js = |e: wasm_bindgen::JsValue| log::warn!("photocraft host bridge: post failed: {e:?}");
    let Some(window) = web_sys::window() else { return };
    let Some(parent) = parent_window() else { return };
    let target = window.location().origin().ok().filter(|o| !o.is_empty()).unwrap_or_else(|| "*".to_string());
    if let Err(e) = parent.post_message(message, &target) {
        js(e);
    }
}

/// A string field of a JS object ("" when absent or not a string).
fn str_at(data: &wasm_bindgen::JsValue, key: &str) -> String {
    js_sys::Reflect::get(data, &wasm_bindgen::JsValue::from_str(key))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

/// An object field as a JS array.
fn array_at(data: &wasm_bindgen::JsValue, key: &str) -> Option<js_sys::Array> {
    js_sys::Reflect::get(data, &wasm_bindgen::JsValue::from_str(key))
        .ok()
        .and_then(|v| v.dyn_into::<js_sys::Array>().ok())
}

/// One inbound message (already origin-checked).
fn handle_message(inner: &Rc<Inner>, data: &wasm_bindgen::JsValue) {
    if str_at(data, "channel") != CHANNEL {
        return;
    }
    match str_at(data, "type").as_str() {
        "open" => {
            let Some(files) = array_at(data, "files") else { return };
            let mut to_open: Vec<(String, web_sys::Blob, Option<String>)> = Vec::new();
            for i in 0..files.length() {
                let item = files.get(i);
                let name = str_at(&item, "name");
                let blob = js_sys::Reflect::get(&item, &wasm_bindgen::JsValue::from_str("blob"))
                    .ok()
                    .and_then(|v| v.dyn_into::<web_sys::Blob>().ok());
                // PHASE-B: the host attaches the originating asset id when
                // this open IS an asset open (modal launch, popout picker,
                // the in-app library browser's answer).
                let asset_id = str_at(&item, "assetId");
                let asset_id = if asset_id.is_empty() { None } else { Some(asset_id) };
                if name.is_empty() {
                    continue;
                }
                match blob {
                    Some(blob) => to_open.push((name, blob, asset_id)),
                    None => log::error!("photocraft host bridge: open file \"{name}\" carries no blob"),
                }
            }
            if to_open.is_empty() {
                return;
            }
            // One after another, in the order sent — the drop path's law.
            // The identity insert happens only AFTER the bytes read (a
            // failed read leaves no phantom identity).
            let inner = inner.clone();
            wasm_bindgen_futures::spawn_local(async move {
                for (name, blob, asset_id) in to_open {
                    match wasm_bindgen_futures::JsFuture::from(blob.array_buffer()).await {
                        Ok(buffer) => {
                            inner.push_inbox(&name, js_sys::Uint8Array::new(&buffer).to_vec());
                            if let Some(id) = asset_id {
                                let mut identity = inner.identity.lock().unwrap_or_else(|e| e.into_inner());
                                // First-wins: a second same-named asset never
                                // steals a live identity (the map prunes with
                                // the docs list anyway).
                                identity.entry(name).or_insert(id);
                            }
                            inner.request_repaint();
                        }
                        Err(e) => log::error!("photocraft host bridge: couldn't read \"{name}\": {e:?}"),
                    }
                }
            });
        }
        "library.list" => {
            let Some(assets) = array_at(data, "assets") else { return };
            let mut entries = Vec::with_capacity(assets.length() as usize);
            for i in 0..assets.length() {
                let item = assets.get(i);
                let id = str_at(&item, "id");
                if id.is_empty() {
                    continue;
                }
                entries.push(CloudEntry {
                    id,
                    name: str_at(&item, "name"),
                    kind: str_at(&item, "kind"),
                    size: str_at(&item, "size"),
                });
            }
            *inner.entries.lock().unwrap_or_else(|e| e.into_inner()) = entries;
            inner.request_repaint();
        }
        "save-request" => {
            let command = if str_at(data, "verb") == "save-all" { HostCommand::SaveAll } else { HostCommand::Save };
            inner.commands.borrow_mut().push_back(command);
            inner.request_repaint();
        }
        "save-ack" => {
            log::debug!("photocraft host bridge: save-ack {} {}", str_at(data, "name"), str_at(data, "ok"));
        }
        _ => {}
    }
}

/// `File {name, size, type}` + a real Blob — the parse-guard-exact `saved`
/// message (the host's `blob instanceof Blob` check accepts it after the
/// structured clone). PHASE-B: when the write consumed an armed asset id
/// it is echoed (`assetId`) and the identity map RE-KEYS to this save name
/// — the engine renames the doc to exactly this name (`saved_to`), so the
/// in-session repeat saves keep hitting the identity instead of going
/// stale after the first `photo.png → photo.psd` save.
fn post_saved(path: &str, bytes: &[u8], identity: &Arc<Mutex<HashMap<String, String>>>, asset_id: Option<&str>) -> Result<(), String> {
    let js = |e: wasm_bindgen::JsValue| format!("{e:?}");
    let name = std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "photocraft".into());
    let message = js_sys::Object::new();
    let set = |o: &js_sys::Object, k: &str, v: wasm_bindgen::JsValue| {
        js_sys::Reflect::set(o as &wasm_bindgen::JsValue, &wasm_bindgen::JsValue::from_str(k), &v).map_err(js)
    };
    set(&message, "channel", wasm_bindgen::JsValue::from_str(CHANNEL))?;
    set(&message, "type", wasm_bindgen::JsValue::from_str("saved"))?;
    if let Some(id) = asset_id {
        set(&message, "assetId", wasm_bindgen::JsValue::from_str(id))?;
        let mut map = identity.lock().unwrap_or_else(|e| e.into_inner());
        map.insert(name.clone(), id.to_string());
    }
    let file = js_sys::Object::new();
    set(&file, "name", wasm_bindgen::JsValue::from_str(&name))?;
    set(&file, "size", wasm_bindgen::JsValue::from_f64(bytes.len() as f64))?;
    set(&file, "type", wasm_bindgen::JsValue::from_str(crate::web::mime_for(&name)))?;
    set(&message, "file", file.into())?;
    let blob = web_sys::Blob::new_with_u8_array_sequence(&js_sys::Array::of1(&js_sys::Uint8Array::from(bytes))).map_err(js)?;
    set(&message, "blob", blob.into())?;
    post_to_parent(&message);
    Ok(())
}

/// The File ▸ Open from Library dispatch (the CloudLibrary::open closure):
/// ask the host for an asset's bytes — it answers through a standard
/// `open`, so the asset opens as a new document tab. Re-fetches the window
/// per call (the download precedent) so nothing non-Send is captured.
fn post_library_open(id: &str) {
    let message = serde_json::json!({
        "channel": CHANNEL,
        "type": "library.open",
        "id": id,
    });
    if let Some(value) = js_sys::JSON::parse(&message.to_string()).ok() {
        post_to_parent(&value);
    }
}

/// The prefs pre-seed: deep-merge the embed seed into the localStorage
/// prefs BEFORE the app loads them (the key is shared with the standalone
/// site on this origin — a standalone user's settings are never clobbered).
fn seed_prefs() {
    let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) else { return };
    let existing = storage.get_item(PREFS_KEY).ok().flatten().unwrap_or_default();
    let mut prefs: serde_json::Value = serde_json::from_str(&existing).unwrap_or_else(|_| serde_json::json!({}));
    fn merge(target: &mut serde_json::Value, patch: &serde_json::Value) {
        match (target.as_object_mut(), patch.as_object()) {
            (Some(target), Some(patch)) => {
                for (key, value) in patch {
                    merge(target.entry(key.clone()).or_insert(serde_json::Value::Null), value);
                }
            }
            _ => {
                if !patch.is_null() {
                    *target = patch.clone();
                }
            }
        }
    }
    merge(
        &mut prefs,
        &serde_json::json!({
            "general": { "autoShowHomeScreen": false },
            "colorSettings": { "askOnMismatch": false },
        }),
    );
    if let Err(e) = storage.set_item(PREFS_KEY, &prefs.to_string()) {
        log::warn!("photocraft host bridge: prefs seed failed: {e:?}");
    }
}

/// The host bridge (Clone — one shared [`Inner`]). Created before the
/// web runner starts (the prefs pre-seed must win the app's prefs load),
/// installed inside the app-creation closure (where the context and the
/// inbox exist), ticked by the shell every frame.
#[derive(Clone)]
pub struct HostBridge {
    inner: Rc<Inner>,
}

impl Default for HostBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl HostBridge {
    /// Detect the embed and pre-seed the prefs. Standalone → fully inert.
    pub fn new() -> Self {
        let connected = parent_window().is_some();
        if connected {
            seed_prefs();
        }
        Self {
            inner: Rc::new(Inner {
                connected,
                inbox: RefCell::default(),
                ctx: RefCell::default(),
                entries: Arc::default(),
                commands: RefCell::default(),
                last_docs: RefCell::default(),
                last_dirty: RefCell::default(),
                identity: Arc::default(),
                pending: Arc::default(),
            }),
        }
    }

    /// Whether a same-origin parent page embeds the editor.
    pub fn is_connected(&self) -> bool {
        self.inner.connected
    }

    /// The `Services::cloud` handle (None standalone — the File-menu cloud
    /// entries never appear).
    pub fn cloud_handle(&self) -> Option<CloudLibrary> {
        if !self.inner.connected {
            return None;
        }
        Some(CloudLibrary {
            entries: self.inner.entries.clone(),
            open: Arc::new(post_library_open),
            identity: self.inner.identity.clone(),
            pending: self.inner.pending.clone(),
        })
    }

    /// The write service while connected: the bytes go to the host as the
    /// parse-guard-exact `saved` message (NO download — the library IS the
    /// save target; confirmed, so the document settles saved). PHASE-B:
    /// CONSUMES the armed asset id (one write = one identity echo — a
    /// later export can never pick up a stale slot) and re-keys the
    /// identity map to the save name.
    pub fn write(&self, path: &str, bytes: &[u8]) -> Result<(), String> {
        if self.inner.connected {
            let armed = self.inner.pending.lock().unwrap_or_else(|e| e.into_inner()).take();
            post_saved(path, bytes, &self.inner.identity, armed.as_deref())
        } else {
            crate::web::download(path, bytes)
        }
    }

    /// Arm the message listener and announce `ready` — called inside the
    /// app-creation closure, before the first frame (the host only speaks
    /// after `ready`, so no pre-boot buffering exists).
    pub fn install(&self, ctx: egui::Context, inbox: Inbox) {
        if !self.inner.connected {
            return;
        }
        *self.inner.ctx.borrow_mut() = Some(ctx);
        *self.inner.inbox.borrow_mut() = Some(inbox);
        let Some(window) = web_sys::window() else { return };
        let own_origin = window.location().origin().ok().unwrap_or_default();
        let inner = self.inner.clone();
        let cb = wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |e: web_sys::MessageEvent| {
            // Same-origin only (the standalone site receives cross-origin
            // noise from embedders it never accepted).
            if e.origin() != own_origin {
                return;
            }
            handle_message(&inner, &e.data());
        });
        if window.add_event_listener_with_callback("message", cb.as_ref().unchecked_ref()).is_ok() {
            cb.forget();
        } else {
            drop(cb);
            return;
        }
        self.inner.post_json(&serde_json::json!({
            "channel": CHANNEL,
            "type": "ready",
            "bridge": BRIDGE_ID,
            "photocraft": APP_VERSION,
        }));
        self.inner.post_json(&serde_json::json!({
            "channel": CHANNEL,
            "type": "library.list.request",
        }));
    }

    /// The runner started and the loading element is gone.
    pub fn post_booted(&self) {
        if !self.inner.connected {
            return;
        }
        self.inner.post_json(&serde_json::json!({ "channel": CHANNEL, "type": "booted" }));
    }

    /// The runner failed (no canvas / no WebGPU or WebGL2).
    pub fn post_boot_failed(&self, reason: &str) {
        if !self.inner.connected {
            return;
        }
        self.inner.post_json(&serde_json::json!({ "channel": CHANNEL, "type": "boot-failed", "reason": reason }));
    }

    /// Every Escape the raw input observes (the host owns the
    /// double-press-to-close law; host-DOM presses never reach eframe).
    pub fn post_esc(&self) {
        if !self.inner.connected {
            return;
        }
        self.inner.post_json(&serde_json::json!({ "channel": CHANNEL, "type": "esc" }));
    }

    /// The shell's every-frame tick (EMBED-DESIGN-CLOUD B4): drain the
    /// host's queued commands (gated on no open dialog — a request that
    /// arrives mid-dialog defers), then post the MDI truth on change
    /// (`docs`, now carrying the D2 per-doc `states`) and the unsaved-work
    /// truth on change (`dirty`).
    pub fn tick(&self, app: &mut PhotocraftApp, ctx: &egui::Context) {
        if !self.inner.connected {
            return;
        }
        loop {
            let command = self.inner.commands.borrow_mut().pop_front();
            let Some(command) = command else { break };
            if !photocraft_ui_egui::menus::host_command_allowed(app, "file.save") {
                // Defer: a dialog/prompt owns the app right now.
                self.inner.commands.borrow_mut().push_front(command);
                break;
            }
            match command {
                HostCommand::Save => {
                    if let Err(e) =
                        photocraft_ui_egui::menus::invoke_from_host(app, ctx, "file.save", serde_json::json!({}))
                    {
                        log::warn!("photocraft host bridge: save failed: {e}");
                    }
                }
                HostCommand::SaveAll => {
                    let (saved, failed) = photocraft_ui_egui::cloud::save_all_to_library(app);
                    self.inner.post_json(&serde_json::json!({
                        "channel": CHANNEL,
                        "type": "save-all-done",
                        "saved": saved,
                        "failed": failed,
                    }));
                }
            }
        }
        let docs: Vec<(String, bool)> = app
            .session
            .documents()
            .iter()
            .map(|d| (d.doc.name.clone(), d.is_dirty()))
            .collect();
        // D2 (EMBED-DESIGN-MDI): the per-doc truth rides the docs event —
        // `states: [{name, dirty, assetId?}]`, name-aligned and additive
        // (old hosts ignore it; their parse guard drops unknown fields).
        // The change detector compares the full tuple vector, so a dirty
        // flip or an identity re-key repushes while a pure no-op frame
        // never posts.
        let states: Vec<serde_json::Value> = {
            let identity = self.inner.identity.lock().unwrap_or_else(|e| e.into_inner());
            docs.iter()
                .map(|(name, dirty)| match identity.get(name) {
                    Some(id) => serde_json::json!({ "name": name, "dirty": dirty, "assetId": id }),
                    None => serde_json::json!({ "name": name, "dirty": dirty }),
                })
                .collect()
        };
        let docs_changed = {
            let mut last = self.inner.last_docs.borrow_mut();
            if *last != states {
                *last = states.clone();
                true
            } else {
                false
            }
        };
        if docs_changed {
            // PHASE-B: prune the identity map to the live docs (closed docs
            // release their identity; the save-name re-key keeps the entry
            // aligned with saved_to's rename). A freshly-inserted identity
            // whose doc hasn't landed yet can fall here — benign: that open
            // degrades to a mint instead of an update.
            {
                let mut identity = self.inner.identity.lock().unwrap_or_else(|e| e.into_inner());
                identity.retain(|name, _| docs.iter().any(|(n, _)| n == name));
            }
            let names: Vec<&str> = docs.iter().map(|(n, _)| n.as_str()).collect();
            self.inner.post_json(&serde_json::json!({
                "channel": CHANNEL,
                "type": "docs",
                "names": names,
                "states": states,
            }));
        }
        let dirty = app.has_unsaved_work();
        let dirty_changed = {
            let mut last = self.inner.last_dirty.borrow_mut();
            if *last != dirty {
                *last = dirty;
                true
            } else {
                false
            }
        };
        if dirty_changed {
            self.inner.post_json(&serde_json::json!({ "channel": CHANNEL, "type": "dirty", "dirty": dirty }));
        }
    }
}
