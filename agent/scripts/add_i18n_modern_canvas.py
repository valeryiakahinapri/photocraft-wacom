#!/usr/bin/env python3
"""Insert the 'Modern Canvas' theme-label row into every ui-egui i18n catalog
(REV6). Anchored after each catalog's 'Pro (Medium Gray)' row — the theme labels
cluster there. Format: context<TAB>source<TTAB>translation, empty context.
"""
import os

BASE = "/home/z/pc-edit/crates/ui-egui/src/i18n"
TRANSLATIONS = {
    "cs": "Moderní plátno",
    "de": "Modern Canvas",
    "el": "Modern Canvas",
    "es": "Lienzo moderno",
    "fr": "Canevas moderne",
    "hi": "मॉडर्न कैनवास",
    "hu": "Modern vászon",
    "id": "Kanvas modern",
    "it": "Tela moderna",
    "ja": "モダンキャンバス",
    "ko": "모던 캔버스",
    "nl": "Modern canvas",
    "pl": "Nowoczesne płótno",
    "pt-br": "Tela moderna",
    "ru": "Современный холст",
    "sr": "Moderno platno",
    "tr": "Modern tuval",
    "uk": "Сучасне полотно",
    "vi": "Khung vẽ hiện đại",
    "zh-hans": "现代画布",
    "zh-hant": "現代畫布",
}
SOURCE = "Modern Canvas"
ANCHOR = "Pro (Medium Gray)"

patched = 0
for fname in sorted(os.listdir(BASE)):
    if not fname.endswith(".tsv"):
        continue
    lang = fname[:-4]
    path = os.path.join(BASE, fname)
    lines = open(path, encoding="utf-8").read().splitlines(keepends=True)
    if any(l.startswith("\t" + SOURCE + "\t") for l in lines):
        print(f"skip (has row): {fname}")
        continue
    out = []
    hit = False
    for line in lines:
        out.append(line)
        if line.startswith("\t" + ANCHOR + "\t"):
            out.append(f"\t{SOURCE}\t{TRANSLATIONS.get(lang, SOURCE)}\n")
            hit = True
    if not hit:
        print(f"MISS (no anchor): {fname}")
        continue
    open(path, "w", encoding="utf-8").write("".join(out))
    patched += 1
    print(f"patched: {fname} -> {TRANSLATIONS.get(lang, SOURCE)}")
print(f"\n{patched} catalogs patched")
