//! Browser-callable, in-memory counterpart to the CLI's `commands`, `info`, `run`, `convert`
//! and `batch` operations. Browser code supplies file bytes; this bridge never accesses paths.
//!
//! Two globals ship: `PhotoCraft` (this file's headless document bridge — its own session,
//! for scripted open/run/export) and `PhotoCraftUI` (live-editor control: the workspace the
//! visible app runs in, fed through [`UiInbox`] to the real app).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use photocraft_codecs::{DdsCompression, DpxBits, JpegFormat, TiffCompression};
use photocraft_engine::{Session, command_specs, file_cmds};
use photocraft_io::{ExportOptions, TgaBits};
use serde_json::{Map, Value, json};

/// Live-UI requests from JavaScript (`PhotoCraftUI`), drained by the web shell each frame:
/// workspace names to switch to (presets by alias, saved workspaces by exact name).
pub(crate) type UiInbox = Arc<Mutex<Vec<String>>>;

/// The live app's egui context, set once the shell's first frame runs: a JS call that queues a
/// request uses it to wake the app, or the queue would sit unread while the app sleeps.
pub(crate) type UiCtx = Arc<Mutex<Option<egui::Context>>>;

#[derive(Default)]
pub(crate) struct BrowserBridge {
    session: Session,
    warnings: HashMap<u64, Vec<String>>,
}

pub(crate) struct Exported {
    pub bytes: Vec<u8>,
    pub warnings: Vec<String>,
}

pub(crate) struct BatchInput {
    pub name: String,
    pub bytes: Vec<u8>,
}

pub(crate) struct BatchOutput {
    pub name: String,
    pub bytes: Vec<u8>,
    pub warnings: Vec<String>,
    pub commands: Value,
}

impl BrowserBridge {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn commands(&self, filter: Option<&str>) -> Value {
        Value::Array(
            command_specs()
                .iter()
                .map(|command| photocraft_engine::commands::list_entry(&self.session, command, self.session.disabled_reason(command.id)))
                .filter(|entry| filter.is_none_or(|needle| photocraft_engine::commands::list_entry_matches(entry, needle)))
                .collect(),
        )
    }

    pub(crate) fn open(&mut self, name: &str, bytes: &[u8]) -> Result<Value, String> {
        let mut imported = photocraft_io::import(name, bytes).map_err(|error| error.to_string())?;
        file_cmds::raw_defaults(&self.session, &imported.warnings, &mut imported.document);
        let mut warnings = imported.warnings;
        warnings.extend(missing_font_warnings(&imported.document));
        let index = self.session.add_document(imported.document, None);
        let state = self.session.documents().get(index).ok_or_else(|| "opened document was not added to the session".to_string())?;
        self.warnings.insert(state.doc.id.0, warnings.clone());
        Ok(json!({
            "index": index,
            "name": state.doc.name,
            "width": state.doc.size.width,
            "height": state.doc.size.height,
            "layers": state.doc.layer_count(),
            "warnings": warnings,
        }))
    }

    pub(crate) fn create(&mut self, params: Value) -> Result<Value, String> {
        self.session.execute("file.new", params).map_err(|error| error.to_string())
    }

    pub(crate) fn execute(&mut self, id: &str, params: Value) -> Result<Value, String> {
        let result = self.session.execute(id, params).map_err(|error| error.to_string())?;
        if let Some(error) = photocraft_engine::actions_cmds::nested_failure(id, &result) {
            return Err(error);
        }
        Ok(result)
    }

    pub(crate) fn run(&mut self, input: &Value) -> Result<Value, String> {
        let steps = photocraft_engine::automate_cmds::parse_action(input, "PhotoCraft.run").map_err(|error| error.to_string())?;
        self.run_steps(steps, "PhotoCraft.run")
    }

    fn run_steps(&mut self, steps: Vec<(String, Value)>, caller: &str) -> Result<Value, String> {
        if steps.is_empty() {
            return Err(format!("{caller} needs at least one command"));
        }
        let mut results = Vec::with_capacity(steps.len());
        let mut reported = std::collections::HashSet::new();
        for (id, params) in steps {
            let result = self.execute(&id, params)?;
            let warnings = self
                .session
                .active()
                .map(|state| missing_font_warnings(&state.doc))
                .unwrap_or_default()
                .into_iter()
                .filter(|warning| reported.insert(warning.clone()))
                .collect::<Vec<_>>();
            let mut entry = json!({"command": id, "result": result});
            if !warnings.is_empty() {
                entry["warnings"] = json!(warnings);
            }
            results.push(entry);
        }
        Ok(Value::Array(results))
    }

    pub(crate) fn info(&self) -> Result<Value, String> {
        let state = self.session.active().ok_or_else(|| "no document open".to_string())?;
        let mut result = photocraft_engine::inspect::document(state);
        if let Value::Object(fields) = &mut result {
            for key in ["history", "canUndo", "canRedo", "revision"] {
                fields.remove(key);
            }
            fields.insert("file".into(), Value::String(state.doc.name.clone()));
            let warnings = self.warnings.get(&state.doc.id.0).cloned().unwrap_or_default();
            fields.insert("warnings".into(), json!(warnings));
        }
        Ok(result)
    }

    pub(crate) fn export(&self, format: &str, options: &Value) -> Result<Exported, String> {
        let state = self.session.active().ok_or_else(|| "no document open".to_string())?;
        export_document(&state.doc, format, options)
    }

    pub(crate) fn convert(&self, name: &str, bytes: &[u8], format: &str, options: &Value) -> Result<Exported, String> {
        let mut imported = photocraft_io::import(name, bytes).map_err(|error| error.to_string())?;
        file_cmds::raw_defaults(&Session::new(), &imported.warnings, &mut imported.document);
        let output_name = with_extension(name, &normalized_format(format)?);
        let mut exported = export_named(&imported.document, &output_name, options)?;
        imported.warnings.extend(missing_font_warnings(&imported.document));
        imported.warnings.append(&mut exported.warnings);
        exported.warnings = imported.warnings;
        Ok(exported)
    }

    pub(crate) fn batch(&self, inputs: &[BatchInput], steps: &Value, format: &str, options: &Value) -> Result<Vec<BatchOutput>, String> {
        let parsed_steps = photocraft_engine::automate_cmds::parse_action(steps, "PhotoCraft.batch").map_err(|error| error.to_string())?;
        if parsed_steps.is_empty() {
            return Err("PhotoCraft.batch needs at least one command".into());
        }
        let format = normalized_format(format)?;
        let mut outputs = Vec::with_capacity(inputs.len());
        for input in inputs {
            let mut bridge = BrowserBridge::new();
            let opened = bridge.open(&input.name, &input.bytes).map_err(|error| format!("{}: {error}", input.name))?;
            let results = bridge.run_steps(parsed_steps.clone(), "PhotoCraft.batch").map_err(|error| format!("{}: {error}", input.name))?;
            let output_name = with_extension(&input.name, &format);
            let active = bridge.session.active().ok_or_else(|| format!("{}: no document open after commands", input.name))?;
            let output = export_named(&active.doc, &output_name, options).map_err(|error| format!("{}: {error}", input.name))?;
            let mut warnings =
                opened.get("warnings").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).map(str::to_owned).collect::<Vec<_>>();
            if let Some(command_results) = results.as_array() {
                warnings.extend(
                    command_results
                        .iter()
                        .filter_map(|result| result.get("warnings").and_then(Value::as_array))
                        .flatten()
                        .filter_map(Value::as_str)
                        .map(str::to_owned),
                );
            }
            let mut seen = std::collections::HashSet::new();
            warnings.retain(|warning| seen.insert(warning.clone()));
            warnings.extend(output.warnings);
            outputs.push(BatchOutput { name: output_name, bytes: output.bytes, warnings, commands: results });
        }
        Ok(outputs)
    }

    pub(crate) fn close(&mut self) -> Value {
        self.session = Session::new();
        self.warnings.clear();
        json!({"closed": true})
    }
}

fn export_document(doc: &photocraft_doc::Document, format: &str, options: &Value) -> Result<Exported, String> {
    let name = with_extension(&doc.name, &normalized_format(format)?);
    export_named(doc, &name, options)
}

fn export_named(doc: &photocraft_doc::Document, name: &str, options: &Value) -> Result<Exported, String> {
    let opts = ExportOptions { read_linked_files: false, ..export_options(options)? };
    let result = photocraft_io::export(doc, name, &opts).map_err(|error| error.to_string())?;
    Ok(Exported { bytes: result.bytes, warnings: result.warnings })
}

fn export_options(input: &Value) -> Result<ExportOptions, String> {
    let fields = match input {
        Value::Null => Map::new(),
        Value::Object(fields) => fields.clone(),
        _ => return Err("`options` must be an object".into()),
    };
    const OPTIONS: &[&str] = &["quality", "tga_bits", "dpx_bits", "tiff_compression", "jpeg_format", "dds_compression", "dds_mipmaps", "tiff_layers"];
    if let Some(key) = fields.keys().find(|key| !OPTIONS.contains(&key.as_str())) {
        return Err(format!("unknown export option `{key}`"));
    }
    let mut options = ExportOptions { tiff_layers: bool_option(&fields, "tiff_layers")?, ..ExportOptions::default() };
    if let Some(quality) = integer_option(&fields, "quality")? {
        let quality = u8::try_from(quality).ok().filter(|quality| (1..=100).contains(quality)).ok_or("bad `quality`: expected a whole number from 1 to 100")?;
        options.encode.jpeg_quality = quality;
        options.encode.webp_quality = quality;
        options.encode.webp_lossless = false;
    }
    if let Some(bits) = integer_option(&fields, "tga_bits")? {
        options.tga_bits = Some(TgaBits::from_bits(bits).ok_or("bad `tga_bits`: expected 24 or 32")?);
    }
    if let Some(bits) = integer_option(&fields, "dpx_bits")? {
        options.encode.dpx_bits = Some(DpxBits::from_bits(bits).ok_or("bad `dpx_bits`: expected 8, 10, 12 or 16")?);
    }
    if let Some(compression) = string_option(&fields, "tiff_compression")? {
        options.encode.tiff_compression = match compression {
            "none" => TiffCompression::None,
            "lzw" => TiffCompression::Lzw,
            "zip" => TiffCompression::Deflate,
            _ => return Err("bad `tiff_compression`: expected none, lzw or zip".into()),
        };
    }
    if let Some(format) = string_option(&fields, "jpeg_format")? {
        options.encode.jpeg_format = JpegFormat::parse(format).ok_or("bad `jpeg_format`: expected baseline, optimized or progressive")?;
    }
    if let Some(compression) = string_option(&fields, "dds_compression")? {
        options.encode.dds.compression = DdsCompression::from_name(compression).ok_or("bad `dds_compression`: expected none, bc1, bc2 or bc3")?;
    }
    options.encode.dds.mipmaps = bool_option(&fields, "dds_mipmaps")?;
    Ok(options)
}

fn integer_option(fields: &Map<String, Value>, key: &str) -> Result<Option<u64>, String> {
    fields.get(key).map_or(Ok(None), |value| value.as_u64().map(Some).ok_or_else(|| format!("`{key}` must be a non-negative integer")))
}

fn string_option<'a>(fields: &'a Map<String, Value>, key: &str) -> Result<Option<&'a str>, String> {
    fields.get(key).map_or(Ok(None), |value| value.as_str().map(Some).ok_or_else(|| format!("`{key}` must be a string")))
}

fn bool_option(fields: &Map<String, Value>, key: &str) -> Result<bool, String> {
    fields.get(key).map_or(Ok(false), |value| value.as_bool().ok_or_else(|| format!("`{key}` must be a boolean")))
}

fn normalized_format(format: &str) -> Result<String, String> {
    let format = format.trim_start_matches('.').to_ascii_lowercase();
    if format.is_empty() || !format.chars().all(|character| character.is_ascii_alphanumeric()) {
        return Err("`format` must be a file extension".into());
    }
    Ok(format)
}

fn with_extension(name: &str, extension: &str) -> String {
    let leaf = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let stem = leaf.rsplit_once('.').map_or(leaf, |(stem, _)| stem);
    format!("{stem}.{extension}")
}

fn missing_font_warnings(doc: &photocraft_doc::Document) -> Vec<String> {
    photocraft_engine::type_extra_cmds::missing_fonts(doc)
        .into_iter()
        .map(|font| format!("font '{font}' is not installed; text may render using a fallback face"))
        .collect()
}

#[cfg(target_arch = "wasm32")]
mod wasm_api {
    use std::cell::RefCell;

    use js_sys::{Array, Function, Object, Reflect, Uint8Array};
    use serde_json::{Value, json};
    use wasm_bindgen::{JsCast, JsValue, closure::Closure, prelude::wasm_bindgen};

    use super::{BrowserBridge, Exported};

    #[wasm_bindgen]
    pub struct PhotoCraftBridge {
        inner: BrowserBridge,
    }

    impl Default for PhotoCraftBridge {
        fn default() -> Self {
            Self { inner: BrowserBridge::new() }
        }
    }

    #[wasm_bindgen]
    impl PhotoCraftBridge {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            Self::default()
        }

        pub fn commands(&self, filter: Option<String>) -> Result<JsValue, JsValue> {
            to_js(&self.inner.commands(filter.as_deref()))
        }

        pub fn open(&mut self, name: String, bytes: Vec<u8>) -> Result<JsValue, JsValue> {
            let result = self.inner.open(&name, &bytes).map_err(js_error)?;
            to_js(&result)
        }

        pub fn create(&mut self, params: JsValue) -> Result<JsValue, JsValue> {
            let params = from_js(&params)?;
            let result = self.inner.create(params).map_err(js_error)?;
            to_js(&result)
        }

        pub fn execute(&mut self, id: String, params: JsValue) -> Result<JsValue, JsValue> {
            let params = from_js(&params)?;
            let result = self.inner.execute(&id, params).map_err(js_error)?;
            to_js(&result)
        }

        pub fn run(&mut self, steps: JsValue) -> Result<JsValue, JsValue> {
            let steps = from_js(&steps)?;
            let result = self.inner.run(&steps).map_err(js_error)?;
            to_js(&result)
        }

        pub fn info(&self) -> Result<JsValue, JsValue> {
            to_js(&self.inner.info().map_err(js_error)?)
        }

        pub fn export(&self, format: String, options: JsValue) -> Result<JsValue, JsValue> {
            let options = from_js(&options)?;
            exported_to_js(self.inner.export(&format, &options).map_err(js_error)?)
        }

        pub fn convert(&self, name: String, bytes: Vec<u8>, format: String, options: JsValue) -> Result<JsValue, JsValue> {
            let options = from_js(&options)?;
            exported_to_js(self.inner.convert(&name, &bytes, &format, &options).map_err(js_error)?)
        }

        pub fn batch(&self, inputs: JsValue, steps: JsValue, format: String, options: JsValue) -> Result<JsValue, JsValue> {
            let inputs = batch_inputs(&inputs)?;
            let steps = from_js(&steps)?;
            let options = from_js(&options)?;
            let result = self.inner.batch(&inputs, &steps, &format, &options).map_err(js_error)?;
            batch_to_js(result)
        }

        pub fn close(&mut self) -> Result<JsValue, JsValue> {
            to_js(&self.inner.close())
        }
    }

    type JsMethod = Closure<dyn FnMut(JsValue) -> Result<JsValue, JsValue>>;

    thread_local! {
        static GLOBAL_BRIDGE: RefCell<BrowserBridge> = RefCell::new(BrowserBridge::new());
        static GLOBAL_METHODS: RefCell<Vec<JsMethod>> = const { RefCell::new(Vec::new()) };
    }

    pub(crate) fn install_global(ui_inbox: super::UiInbox, ui_ctx: super::UiCtx) -> Result<(), JsValue> {
        let api = Object::new();
        add_method(&api, "commands", |input| {
            let filter = if input.is_undefined() || input.is_null() {
                None
            } else if let Some(filter) = input.as_string() {
                Some(filter)
            } else {
                Some(string_field(&input, "filter")?)
            };
            with_global(|bridge| to_js(&bridge.commands(filter.as_deref())))
        })?;
        add_method(&api, "open", |input| {
            let name = string_field(&input, "name")?;
            let bytes = bytes_field(&input, "bytes")?;
            with_global(|bridge| to_js(&bridge.open(&name, &bytes).map_err(js_error)?))
        })?;
        add_method(&api, "create", |input| with_global(|bridge| to_js(&bridge.create(from_js(&input)?).map_err(js_error)?)))?;
        add_method(&api, "execute", |input| {
            let id = string_field(&input, "id")?;
            let params = optional_field(&input, "params")?;
            with_global(|bridge| to_js(&bridge.execute(&id, from_js(&params)?).map_err(js_error)?))
        })?;
        add_method(&api, "run", |input| {
            let steps = field(&input, "steps")?;
            with_global(|bridge| to_js(&bridge.run(&from_js(&steps)?).map_err(js_error)?))
        })?;
        add_method(&api, "info", |_| with_global(|bridge| to_js(&bridge.info().map_err(js_error)?)))?;
        add_method(&api, "export", |input| {
            let format = string_field(&input, "format")?;
            let options = optional_field(&input, "options")?;
            with_global(|bridge| exported_to_js(bridge.export(&format, &from_js(&options)?).map_err(js_error)?))
        })?;
        add_method(&api, "convert", |input| {
            let name = string_field(&input, "name")?;
            let bytes = bytes_field(&input, "bytes")?;
            let format = string_field(&input, "format")?;
            let options = optional_field(&input, "options")?;
            with_global(|bridge| exported_to_js(bridge.convert(&name, &bytes, &format, &from_js(&options)?).map_err(js_error)?))
        })?;
        add_method(&api, "batch", |input| {
            let inputs = field(&input, "inputs")?;
            let steps = field(&input, "steps")?;
            let format = string_field(&input, "format")?;
            let options = optional_field(&input, "options")?;
            let inputs = batch_inputs(&inputs)?;
            let result = with_global(|bridge| bridge.batch(&inputs, &from_js(&steps)?, &format, &from_js(&options)?).map_err(js_error))?;
            batch_to_js(result)
        })?;
        add_method(&api, "close", |_| with_global(|bridge| to_js(&bridge.close())))?;

        let global = js_sys::global();
        Reflect::set(&global, &JsValue::from_str("PhotoCraft"), &api)
            .and_then(|set| if set { Ok(()) } else { Err(js_error("could not publish `globalThis.PhotoCraft`")) })?;

        // `PhotoCraftUI`: the LIVE editor's control surface (the app the page shows — not the
        // headless `PhotoCraft` session above). v1: the workspace — set it to a preset (aliases
        // welcome: "paint" → Painting) or a saved workspace name; the shell applies the request
        // on its next frame and republishes `.state`. `workspace()` with no argument returns
        // the last published state.
        let ui = Object::new();
        let state = Object::new();
        Reflect::set(&state, &JsValue::from_str("workspace"), &JsValue::from_str("Essentials"))?;
        Reflect::set(&ui, &JsValue::from_str("state"), &state)?;
        let (for_get, inbox, wake) = (ui.clone(), ui_inbox, ui_ctx);
        add_method(&ui, "workspace", move |input| {
            if input.is_undefined() || input.is_null() {
                return Reflect::get(&for_get, &JsValue::from_str("state"));
            }
            let name = if let Some(name) = input.as_string() {
                name
            } else {
                string_field(&input, "name")?
            };
            if name.trim().is_empty() {
                return Err(js_error("pass a workspace name, or nothing to read the current one"));
            }
            // Presets resolve to their canonical key for the reply; a name that isn't one may
            // still be a saved workspace — the app decides when it drains the queue.
            let reply = photocraft_ui_egui::workspace_ui::canonical(&name)
                .map(str::to_string)
                .unwrap_or_else(|| name.clone());
            inbox.lock().unwrap_or_else(|e| e.into_inner()).push(name);
            if let Some(ctx) = wake.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
                ctx.request_repaint();
            }
            to_js(&json!({"workspace": reply, "queued": true}))
        })?;
        Reflect::set(&global, &JsValue::from_str("PhotoCraftUI"), &ui)
            .and_then(|set| if set { Ok(()) } else { Err(js_error("could not publish `globalThis.PhotoCraftUI`")) })
    }

    fn add_method(api: &Object, name: &str, f: impl FnMut(JsValue) -> Result<JsValue, JsValue> + 'static) -> Result<(), JsValue> {
        let method = Closure::wrap(Box::new(f) as Box<dyn FnMut(JsValue) -> Result<JsValue, JsValue>>);
        let function: &Function = method.as_ref().unchecked_ref();
        let set = Reflect::set(api, &JsValue::from_str(name), function)?;
        if !set {
            return Err(js_error(&format!("could not publish PhotoCraft.{name}")));
        }
        GLOBAL_METHODS.with(|methods| -> Result<(), JsValue> {
            let mut methods = methods.try_borrow_mut().map_err(|_| js_error("JavaScript bridge initialization is already active"))?;
            methods.push(method);
            Ok(())
        })?;
        Ok(())
    }

    fn with_global<T>(f: impl FnOnce(&mut BrowserBridge) -> Result<T, JsValue>) -> Result<T, JsValue> {
        GLOBAL_BRIDGE.with(|bridge| {
            let mut bridge = bridge.try_borrow_mut().map_err(|_| js_error("PhotoCraft JavaScript calls cannot be nested"))?;
            f(&mut bridge)
        })
    }

    fn field(input: &JsValue, name: &str) -> Result<JsValue, JsValue> {
        Reflect::get(input, &JsValue::from_str(name))
    }

    fn optional_field(input: &JsValue, name: &str) -> Result<JsValue, JsValue> {
        let value = field(input, name)?;
        if value.is_undefined() || value.is_null() { Ok(JsValue::NULL) } else { Ok(value) }
    }

    fn string_field(input: &JsValue, name: &str) -> Result<String, JsValue> {
        field(input, name)?.as_string().ok_or_else(|| js_error(&format!("`{name}` must be a string")))
    }

    fn bytes_field(input: &JsValue, name: &str) -> Result<Vec<u8>, JsValue> {
        let bytes = field(input, name)?;
        if !bytes.is_instance_of::<Uint8Array>() {
            return Err(js_error(&format!("`{name}` must be a Uint8Array")));
        }
        Ok(Uint8Array::new(&bytes).to_vec())
    }

    fn batch_inputs(value: &JsValue) -> Result<Vec<super::BatchInput>, JsValue> {
        if !Array::is_array(value) {
            return Err(js_error("`inputs` must be an array"));
        }
        Array::from(value)
            .iter()
            .map(|input| {
                let name = string_field(&input, "name")?;
                let bytes = bytes_field(&input, "bytes")?;
                Ok(super::BatchInput { name, bytes })
            })
            .collect()
    }

    fn from_js(value: &JsValue) -> Result<Value, JsValue> {
        if value.is_undefined() || value.is_null() {
            return Ok(Value::Null);
        }
        let text = js_sys::JSON::stringify(value)?.as_string().ok_or_else(|| js_error("JSON.stringify did not return a string"))?;
        serde_json::from_str(&text).map_err(|error| js_error(&format!("invalid JSON value: {error}")))
    }

    fn to_js(value: &Value) -> Result<JsValue, JsValue> {
        let text = serde_json::to_string(value).map_err(|error| js_error(&error.to_string()))?;
        js_sys::JSON::parse(&text)
    }

    fn exported_to_js(output: Exported) -> Result<JsValue, JsValue> {
        let result = Object::new();
        Reflect::set(&result, &JsValue::from_str("bytes"), &Uint8Array::from(output.bytes.as_slice()))?;
        Reflect::set(&result, &JsValue::from_str("warnings"), &to_js(&json!(output.warnings))?)?;
        Ok(result.into())
    }

    fn batch_to_js(items: Vec<super::BatchOutput>) -> Result<JsValue, JsValue> {
        let output = Array::new();
        for item in items {
            let entry = Object::new();
            Reflect::set(&entry, &JsValue::from_str("name"), &JsValue::from_str(&item.name))?;
            Reflect::set(&entry, &JsValue::from_str("bytes"), &Uint8Array::from(item.bytes.as_slice()))?;
            Reflect::set(&entry, &JsValue::from_str("warnings"), &to_js(&json!(item.warnings))?)?;
            Reflect::set(&entry, &JsValue::from_str("commands"), &to_js(&item.commands)?)?;
            output.push(&entry);
        }
        Ok(output.into())
    }

    fn js_error(error: impl std::fmt::Display) -> JsValue {
        JsValue::from_str(&error.to_string())
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) use wasm_api::install_global;

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use photocraft_engine::Session;

    use super::{BatchInput, BrowserBridge, export_options};

    #[test]
    fn run_matches_the_engine_command_results() {
        let mut bridge = BrowserBridge::new();
        bridge.create(json!({"width": 12, "height": 9})).unwrap();
        let result = bridge.run(&json!([["layer.new.layer", {"name": "Oracle"}]])).unwrap();
        let mut oracle = Session::new();
        oracle.execute("file.new", json!({"width": 12, "height": 9})).unwrap();
        let expected = oracle.execute("layer.new.layer", json!({"name": "Oracle"})).unwrap();
        assert_eq!(without_allocated_layer_id(result[0]["result"].clone()), without_allocated_layer_id(expected));
        assert_eq!(bridge.info().unwrap()["name"], "Untitled");
        assert_eq!(bridge.info().unwrap()["layers"][0]["name"], "Oracle");
    }

    fn without_allocated_layer_id(mut result: Value) -> Value {
        if let Some(fields) = result.as_object_mut() {
            fields.remove("layer");
        }
        result
    }

    #[test]
    fn command_failures_and_bad_export_options_are_errors() {
        let mut bridge = BrowserBridge::new();
        assert!(bridge.execute("not.a.command", json!({})).is_err());
        assert!(export_options(&json!({"quality": 101})).is_err());
        assert!(export_options(&json!({"dds_compression": "invalid"})).is_err());
        assert_eq!(bridge.close(), json!({"closed": true}));
        assert!(bridge.info().is_err());
    }

    #[test]
    fn command_list_filter_matches_the_cli_filter() {
        let bridge = BrowserBridge::new();
        let all = bridge.commands(None);
        let filtered = bridge.commands(Some("layer.new"));
        assert_eq!(
            filtered,
            Value::Array(all.as_array().unwrap().iter().filter(|entry| photocraft_engine::commands::list_entry_matches(entry, "layer.new")).cloned().collect())
        );
        assert!(filtered.as_array().is_some_and(|entries| !entries.is_empty()));
    }

    #[test]
    fn browser_batch_runs_in_memory_and_names_exports_by_format() {
        let mut source = BrowserBridge::new();
        source.create(json!({"width": 3, "height": 2})).unwrap();
        let bytes = source.export("png", &Value::Null).unwrap().bytes;
        let batch = BrowserBridge::new();
        let inputs = [BatchInput { name: "photo.jpg".into(), bytes }];
        let outputs = batch.batch(&inputs, &json!([["layer.new.layer", {"name": "Batch"}]]), "png", &Value::Null).unwrap();
        assert_eq!(outputs[0].name, "photo.png");
        assert!(!outputs[0].bytes.is_empty());
        assert!(outputs[0].warnings.iter().all(|warning| !warning.trim().is_empty()));
        assert_eq!(outputs[0].commands[0]["command"], "layer.new.layer");
    }

    #[test]
    fn browser_convert_returns_encoded_bytes() {
        let mut bridge = BrowserBridge::new();
        bridge.create(json!({"width": 3, "height": 2})).unwrap();
        let bytes = bridge.export("png", &Value::Null).unwrap().bytes;
        let converted = bridge.convert("photo.png", &bytes, "bmp", &Value::Null).unwrap();
        assert!(!converted.bytes.is_empty());
    }
}
