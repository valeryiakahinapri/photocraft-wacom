import assert from "node:assert/strict";
import {createRequire} from "node:module";
import {mkdtemp, mkdir, readFile, rm, writeFile, copyFile} from "node:fs/promises";
import {existsSync} from "node:fs";
import {tmpdir} from "node:os";
import {dirname, join, resolve} from "node:path";
import {fileURLToPath} from "node:url";
import {spawnSync} from "node:child_process";
import {createHash} from "node:crypto";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const cli = process.env.PHOTOCRAFT_CLI ?? join(root, "target/debug/photocraft-cli");
const wasm = process.env.PHOTOCRAFT_WEB_WASM ?? join(root, "target/wasm32-unknown-unknown/wasm-release/photocraft-web.wasm");
const wasmBindgen = process.env.WASM_BINDGEN ?? "wasm-bindgen";

for (const [name, path] of [["photocraft-cli", cli], ["web WASM", wasm]]) {
    assert.ok(existsSync(path), `${name} not found at ${path}`);
}

function run(command, args) {
    const result = spawnSync(command, args, {encoding: "utf8"});
    assert.equal(result.status, 0, `${command} ${args.join(" ")} failed:\n${result.stderr}`);
    return result.stdout.trim();
}

function runCli(...args) {
    return run(cli, args);
}

function digest(bytes) {
    return createHash("sha256").update(bytes).digest("hex");
}

function withoutVolatileInfo(info) {
    const stable = structuredClone(info);
    delete stable.file;
    delete stable.path;
    delete stable.warnings;
    return stable;
}

function withoutGeneratedLayerId(result) {
    const stable = structuredClone(result);
    if (stable.result && typeof stable.result === "object") {
        delete stable.result.layer;
    }
    return stable;
}

function withoutEnvironmentCommandState(commands) {
    // Host capability and active-document state differ between the browser and CLI.
    return commands.map(({enabled, disabledReason, needsDocument, ...command}) => command);
}

const work = await mkdtemp(join(tmpdir(), "photocraft-js-bridge-"));
try {
    const generated = join(work, "wasm-bindgen");
    await mkdir(generated);
    run(wasmBindgen, ["--target", "nodejs", "--out-dir", generated, wasm]);

    const require = createRequire(import.meta.url);
    const glue = require(join(generated, "photocraft-web.js"));
    assert.equal(typeof glue.PhotoCraftBridge, "function", "wasm-bindgen must export PhotoCraftBridge");
    assert.equal(typeof globalThis.PhotoCraft?.commands, "function", "the browser page must publish globalThis.PhotoCraft");
    const bridge = new glue.PhotoCraftBridge();

    const source = join(work, "source.pcraft");
    const params = {name: "Browser parity"};
    runCli("run", "--new", JSON.stringify({width: 32, height: 24}), "--cmd", "layer.new.layer", "--params", JSON.stringify(params), "--out", source);
    const sourceBytes = new Uint8Array(await readFile(source));

    const cliCommands = JSON.parse(runCli("commands", "--json"));
    const stableCliCommands = withoutEnvironmentCommandState(cliCommands);
    assert.deepEqual(withoutEnvironmentCommandState(bridge.commands()), stableCliCommands, "command registry metadata should match the CLI");
    assert.deepEqual(withoutEnvironmentCommandState(globalThis.PhotoCraft.commands()), stableCliCommands, "page-global command metadata should match the CLI");
    const filteredCommands = JSON.parse(runCli("commands", "--json", "--filter", "layer.new"));
    const stableFilteredCommands = withoutEnvironmentCommandState(filteredCommands);
    assert.deepEqual(withoutEnvironmentCommandState(bridge.commands("layer.new")), stableFilteredCommands, "filtered command registry metadata should match the CLI");
    assert.deepEqual(withoutEnvironmentCommandState(globalThis.PhotoCraft.commands({filter: "layer.new"})), stableFilteredCommands, "page-global filtered command metadata should match the CLI");
    globalThis.PhotoCraft.create({width: 4, height: 3});
    globalThis.PhotoCraft.run({steps: [["layer.new.layer", {name: "Global API"}]]});
    assert.equal(globalThis.PhotoCraft.info().layers[0].name, "Global API");
    globalThis.PhotoCraft.close();

    bridge.open("source.pcraft", sourceBytes);
    assert.deepEqual(withoutVolatileInfo(bridge.info()), withoutVolatileInfo(JSON.parse(runCli("info", source, "--compact"))), "document info should match the CLI");

    const steps = [["layer.new.layer", {name: "Executed from JS"}]];
    const webRun = bridge.run(steps);
    const cliRun = JSON.parse(runCli("run", source, "--cmd", steps[0][0], "--params", JSON.stringify(steps[0][1])));
    assert.deepEqual(withoutGeneratedLayerId(webRun[0]), withoutGeneratedLayerId(cliRun), "engine command results should match the CLI");

    const webExport = bridge.export("png", {});
    const exportedPath = join(work, "cli-export.png");
    runCli("run", source, "--cmd", steps[0][0], "--params", JSON.stringify(steps[0][1]), "--out", exportedPath);
    assert.equal(digest(webExport.bytes), digest(await readFile(exportedPath)), "PNG export bytes should match the CLI");

    const converted = bridge.convert("source.pcraft", sourceBytes, "png", {});
    const convertedPath = join(work, "cli-convert.png");
    runCli("convert", source, convertedPath);
    assert.equal(digest(converted.bytes), digest(await readFile(convertedPath)), "conversion bytes should match the CLI");

    const batchIn = join(work, "batch-in");
    const batchOut = join(work, "batch-out");
    await mkdir(batchIn);
    await copyFile(source, join(batchIn, "source.pcraft"));
    const actions = join(work, "actions.json");
    await writeFile(actions, JSON.stringify(steps));
    runCli("batch", "--actions", actions, "--in", batchIn, "--out", batchOut, "--format", "png");
    const webBatch = bridge.batch([{name: "source.pcraft", bytes: sourceBytes}], steps, "png", {});
    const cliBatchBytes = await readFile(join(batchOut, "source.png"));
    assert.equal(webBatch.length, 1);
    assert.equal(webBatch[0].name, "source.png");
    assert.equal(digest(webBatch[0].bytes), digest(cliBatchBytes), "batch output bytes should match the CLI");

    assert.throws(() => bridge.export("png", {unknown: true}), /unknown export option/);
    assert.throws(() => bridge.run([]), /at least one command/);
    console.log(`JavaScript bridge parity passed: commands, filtered commands, info, run, export, convert, batch (SHA-256 ${digest(converted.bytes)})`);
} finally {
    await rm(work, {recursive: true, force: true});
}
