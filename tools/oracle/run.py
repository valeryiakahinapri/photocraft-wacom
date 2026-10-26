#!/usr/bin/env python3
"""oracle-run.py — the A/B oracle harness (REV7).

Pixel-exact comparison between the HTML mockup (oracle route) and the egui wasm app:

  L1  token probes   — 5x5 median samples at flat-region points vs the measured sheet
  L2  boundaries     — scanline fill-run transitions vs the region map, ±1px
  L3  corners        — corner-reveal area comparison (AA-robust), ±1.5px equivalent radius
  L4  masked diff    — per-region ΔE on content-masked chrome bands, ≥99% within 2.5

Isolation protocol (auditor conditions): one named agent-browser session per run, storage
cleared on the hub origin before each app navigation, DPR=1 + screenshot-size asserts,
fonts.ready + Geist check (mockup), #photocraft_loading removal + settle (app), ?webgl
backend pinned + asserted.

Usage:  python3 tools/oracle/run.py [--state default]
        (writes oracle-report.{json,png} + side-by-side + heatmap under download/oracle/)
"""
from __future__ import annotations

import json
import subprocess
import sys
import time
from pathlib import Path

import numpy as np
from PIL import Image

HUB = "http://localhost:3000"
APP_URL = f"{HUB}/photocraft/?oracle=1&webgl"
MOCKUP_URL = "http://localhost:8081/oracle"
SESSION = "oracle-ab"
OUT = Path("/home/z/my-project/download/oracle")
VIEWPORT = (1440, 900)
SETTLE_MS = 900

# The measured sheet (browser-rasterized; R1-0 law). Keep in lockstep with theme.rs's
# ModernCanvas block + the token test.
TOKENS = {
    "background": (0x07, 0x08, 0x0A),
    "workspace": (0x04, 0x04, 0x05),
    "surface": (0x12, 0x14, 0x16),
    "surface2": (0x1F, 0x21, 0x23),
    "surface3": (0x29, 0x2B, 0x2F),
    "popover": (0x1B, 0x1D, 0x20),
    "primary": (0x3B, 0x9C, 0xF6),
    "pill": (0x0E, 0x0F, 0x10),
    "track": (0x0D, 0x0E, 0x10),
}

# Region -> expected fill token (the chrome skeleton; content is masked in L4).
REGION_FILL = {
    "title-bar": "background",
    "options-bar": "surface",
    "toolbar": "surface",
    "document": "workspace",
    "tab-strip": "workspace",
    "workspace": "workspace",
    "right-island": "surface",
    "rail": "surface",
    "panel-column": "surface",
    "status-pill": "pill",
}


def ab(*args: str, check: bool = True) -> str:
    r = subprocess.run(["agent-browser", "--session", SESSION, *args], capture_output=True, text=True, timeout=120)
    if check and r.returncode != 0:
        raise RuntimeError(f"agent-browser {args} failed: {r.stderr or r.stdout}")
    return r.stdout


def eval_js(js: str) -> str:
    out = ab("eval", js)
    line = out.strip().split("\n")[-1]
    # agent-browser prints the result JSON-encoded: strip the surrounding quotes + unescape
    if line.startswith('"') and line.endswith('"'):
        line = line[1:-1].encode().decode("unicode_escape")
    return line


def boot_and_capture(url: str, is_app: bool) -> Image.Image:
    ab("open", url)
    if is_app:
        for _ in range(60):
            if "true" in eval_js("document.getElementById('photocraft_loading') === null"):
                break
            time.sleep(0.5)
        else:
            raise RuntimeError("READY_TIMEOUT: #photocraft_loading never removed")
    else:
        for _ in range(30):
            if "loaded" in eval_js("document.fonts.status"):
                break
            time.sleep(0.3)
        if "false" in eval_js("document.fonts.check('12.5px Geist')"):
            raise RuntimeError("FONT_MISSING: Geist did not load")
    dpr = eval_js("String(devicePixelRatio)")
    if "1" not in dpr:
        raise RuntimeError(f"DPR_MISMATCH: devicePixelRatio={dpr}")
    time.sleep(SETTLE_MS / 1000)
    shot = OUT / "capture.png"
    ab("screenshot", str(shot))
    img = Image.open(shot).convert("RGB")
    if img.size != VIEWPORT:
        raise RuntimeError(f"SIZE_MISMATCH: screenshot {img.size} != {VIEWPORT}")
    return img


REGIONS_JS = """
(() => {
  const out = {};
  for (const el of document.querySelectorAll('[data-oracle-region]')) {
    const b = el.getBoundingClientRect();
    out[el.dataset.oracleRegion] = [Math.round(b.x), Math.round(b.y), Math.round(b.width), Math.round(b.height)];
  }
  return JSON.stringify(out);
})()"""


def extract_regions() -> dict:
    raw = eval_js(REGIONS_JS)
    return json.loads(raw)


# ---------------------------------------------------------------- comparison layers
def rgb(img: Image.Image) -> np.ndarray:
    return np.asarray(img, dtype=np.int16)


def l1_probes(app: np.ndarray, mck: np.ndarray, regions: dict) -> list[dict]:
    findings = []
    for name, (x, y, w, h) in regions.items():
        token = REGION_FILL.get(name)
        if not token or w < 16 or h < 16:
            continue
        cx, cy = x + w // 2, y + h // 2
        for side, arr in (("app", app), ("mck", mck)):
            patch = arr[cy - 2 : cy + 3, cx - 2 : cx + 3].reshape(-1, 3)
            med = [int(v) for v in np.median(patch, axis=0)]
            exp = list(TOKENS[token])
            ok = all(abs(med[i] - exp[i]) <= 2 for i in range(3))
            findings.append({"layer": "L1", "region": name, "side": side, "sampled": med, "expected": exp, "ok": ok})
    return findings


def l2_scanlines(app: np.ndarray, mck: np.ndarray, regions: dict) -> list[dict]:
    findings = []

    def runs(arr: np.ndarray, y: int) -> list[tuple[int, int, tuple]]:
        row = arr[y]
        out, start = [], 0
        for x in range(1, len(row)):
            if not np.array_equal(row[x], row[start]):
                out.append((start, x, tuple(int(v) for v in row[start])))
                start = x
        out.append((start, len(row), tuple(int(v) for v in row[start])))
        return [r for r in out if r[1] - r[0] >= 4]

    ys = {
        "title-mid": regions["title-bar"][1] + 20,
        "options-mid": regions["options-bar"][1] + 22,
        "main-mid": 500,
    }
    for label, y in ys.items():
        ra, rm = runs(app, y), runs(mck, y)
        ta = sorted([r[0] for r in ra] + [r[1] for r in ra])
        tm = sorted([r[0] for r in rm] + [r[1] for r in rm])
        same_count = len(ra) == len(rm)
        max_shift = max((abs(a - b) for a, b in zip(ta, tm)), default=0) if same_count else 99
        ca = sorted(ra, key=lambda r: r[1] - r[0], reverse=True)[:3]
        cm = sorted(rm, key=lambda r: r[1] - r[0], reverse=True)[:3]
        colors_ok = all(
            all(abs(ca[i][2][j] - cm[i][2][j]) <= 2 for j in range(3)) for i in range(min(len(ca), len(cm)))
        )
        findings.append({"layer": "L2", "scanline": label, "y": y, "app_runs": len(ra), "mck_runs": len(rm),
                         "max_shift": int(max_shift), "colors_ok": bool(colors_ok),
                         "ok": bool(same_count and max_shift <= 1 and colors_ok)})
    return findings


def l3_corners(app: np.ndarray, mck: np.ndarray, regions: dict) -> list[dict]:
    findings = []
    corners = []
    for name in ("toolbar", "options-bar", "document", "right-island", "status-pill"):
        if name not in regions:
            continue
        x, y, w, h = regions[name]
        corners += [
            (f"{name}-tl", x, y), (f"{name}-tr", x + w - 13, y),
            (f"{name}-bl", x, y + h - 13), (f"{name}-br", x + w - 13, y + h - 13),
        ]
    for label, cx, cy in corners:
        aa = app[cy : cy + 13, cx : cx + 13].reshape(-1, 3)
        am = mck[cy : cy + 13, cx : cx + 13].reshape(-1, 3)
        fill = TOKENS[REGION_FILL.get(label.rsplit("-", 1)[0], "surface")]

        def area(px):
            return int(sum(1 for p in px if any(abs(int(p[i]) - fill[i]) > 6 for i in range(3))))

        a, m = area(aa), area(am)
        r = max(2.0, ((a + m) / 2 / (1 - np.pi / 4)) ** 0.5)
        dr = abs(a - m) / (2 * r * (1 - np.pi / 4)) if r > 0 else 99
        findings.append({"layer": "L3", "corner": label, "area_app": a, "area_mck": m,
                         "equiv_radius": round(float(r), 1), "dr": round(float(dr), 2), "ok": bool(dr <= 1.5)})
    return findings


def l4_masked(app: np.ndarray, mck: np.ndarray, regions: dict) -> list[dict]:
    findings = []
    mask = np.ones(app.shape[:2], dtype=bool)  # True = compare

    for name in ("options-bar", "toolbar", "right-island", "status-pill", "tab-strip", "title-bar"):
        if name not in regions:
            continue
        x, y, w, h = regions[name]
        x0, y0, x1, y1 = x + 8, y + 8, x + w - 8, y + h - 8
        if x1 > x0 and y1 > y0:
            mask[y0:y1, x0:x1] = False
    if "workspace" in regions:
        x, y, w, h = regions["workspace"]
        mask[int(y + h * 0.2) : int(y + h * 0.8), int(x + w * 0.2) : int(x + w * 0.8)] = False

    def oklab(px):
        c = px / 255.0
        lin = np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)
        l = 0.4122214708 * lin[..., 0] + 0.5363325363 * lin[..., 1] + 0.0514459929 * lin[..., 2]
        m_ = 0.2119034982 * lin[..., 0] + 0.6806995451 * lin[..., 1] + 0.1073969566 * lin[..., 2]
        s = 0.0883024619 * lin[..., 0] + 0.2817188376 * lin[..., 1] + 0.6299787005 * lin[..., 2]
        l, m_, s = np.cbrt(l), np.cbrt(m_), np.cbrt(s)
        return np.stack([0.2104542553 * l + 0.7936177850 * m_ - 0.0040720468 * s,
                         1.9779984951 * l - 2.4285922050 * m_ + 0.4505937099 * s,
                         0.0259040371 * l + 0.7827717662 * m_ - 0.8086757660 * s], axis=-1)

    de = np.sqrt(np.sum((oklab(app.astype(np.float32)) - oklab(mck.astype(np.float32))) ** 2, axis=-1))

    for name in REGION_FILL:
        if name not in regions:
            continue
        x, y, w, h = regions[name]
        sub_mask = mask[y : y + h, x : x + w]
        sub_de = de[y : y + h, x : x + w]
        n = int(sub_mask.sum())
        if n < 50:
            continue
        intol = float((sub_de[sub_mask] <= 2.5).mean())
        mean = float(sub_de[sub_mask].mean())
        findings.append({"layer": "L4", "region": name, "compared_px": n, "in_tol": round(intol, 4),
                         "mean_dE": round(mean, 2), "ok": bool(intol >= 0.99 and mean <= 1.0)})
    g = de[mask]
    findings.append({"layer": "L4", "region": "GLOBAL", "compared_px": int(mask.sum()),
                     "in_tol": round(float((g <= 2.5).mean()), 4), "mean_dE": round(float(g.mean()), 2),
                     "ok": bool(float((g <= 2.5).mean()) >= 0.995)})

    heat = np.zeros((*de.shape, 3), dtype=np.uint8)
    heat[..., 0] = np.clip(de * 40, 0, 255)
    Image.fromarray(heat).save(OUT / "oracle-diff-heatmap.png")
    return findings


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    state = sys.argv[sys.argv.index("--state") + 1] if "--state" in sys.argv else "default"
    print(f"[oracle] state={state} — session {SESSION}")

    ab("close", check=False)
    ab("set", "viewport", str(VIEWPORT[0]), str(VIEWPORT[1]))

    print("[oracle] rendering mockup /oracle …")
    mck = boot_and_capture(MOCKUP_URL, is_app=False)
    mck.save(OUT / "oracle-mockup.png")
    regions = extract_regions()
    print(f"[oracle] regions: {sorted(regions)}")

    print("[oracle] rendering app ?oracle=1 …")
    ab("open", HUB + "/")
    time.sleep(0.6)
    ab("storage", "local", "clear")
    app = boot_and_capture(APP_URL, is_app=True)
    app.save(OUT / "oracle-app.png")

    a, m = rgb(app), rgb(mck)
    findings = []
    findings += l1_probes(a, m, regions)
    findings += l2_scanlines(a, m, regions)
    findings += l3_corners(a, m, regions)
    findings += l4_masked(a, m, regions)

    side = Image.new("RGB", (VIEWPORT[0] * 2 + 8, VIEWPORT[1]), (24, 24, 24))
    side.paste(mck, (0, 0))
    side.paste(app, (VIEWPORT[0] + 8, 0))
    side.save(OUT / "oracle-side-by-side.png")

    report = {"state": state, "viewport": VIEWPORT,
              "tokens": {k: "#%02X%02X%02X" % v for k, v in TOKENS.items()},
              "regions": regions, "findings": findings,
              "pass": bool(all(f["ok"] for f in findings))}
    (OUT / "oracle-report.json").write_text(json.dumps(report, indent=1))
    fails = [f for f in findings if not f["ok"]]
    print(f"[oracle] {'PASS' if report['pass'] else 'FAIL'} — {len(fails)} failing of {len(findings)}")
    for f in fails[:24]:
        print("   ", json.dumps(f))
    ab("close", check=False)
    return 0 if report["pass"] else 1


if __name__ == "__main__":
    sys.exit(main())
