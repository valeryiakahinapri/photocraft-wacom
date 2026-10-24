#!/usr/bin/env python3
"""Insert 'Collapse Panel Dock' / 'Expand Panel Dock' rows into all ui-egui i18n catalogs.

Adapted from add_i18n_rows.py (which did the same for Hide/Show Dock) for the
REV5 reconciliation: the live agent's c8105921 shipped the two new tl! keys
with no catalog rows; English falls back to the key string, but every catalog
carries the pair. Rows go right after the 'Show/Hide Panels' anchor.
Format: context<TAB>source<TAB>translation, context empty for plain strings.
"""
import os

ROWS = {
    "de":  ("Dock einklappen", "Dock ausklappen"),
    "fr":  ("Réduire le dock", "Développer le dock"),
    "ja":  ("ドックを折りたたむ", "ドックを展開する"),
    "zh-hans": ("折叠停靠栏", "展开停靠栏"),
    "zh-hant": ("折疊停駐欄", "展開停駐欄"),
    "ru":  ("Свернуть панели", "Развернуть панели"),
    "es":  ("Contraer el dock", "Expandir el dock"),
    "it":  ("Comprimi il dock", "Espandi il dock"),
    "ko":  ("독 접기", "독 펼치기"),
    "pt-br": ("Recolher o dock", "Expandir o dock"),
    "nl":  ("Dock inklappen", "Dock uitklappen"),
    "pl":  ("Zwiń dock", "Rozwiń dock"),
    "tr":  ("Panelleri daralt", "Panelleri genişlet"),
    "uk":  ("Згорнути панелі", "Розгорнути панелі"),
    "vi":  ("Thu gọn bảng điều khiển", "Mở rộng bảng điều khiển"),
    "id":  ("Lipat dock", "Bentangkan dock"),
    "hi":  ("डॉक सिकोड़ें", "डॉक विस्तारित करें"),
    "hu":  ("Dock összecsukása", "Dock kibontása"),
    "el":  ("Σύμπτυξη dock", "Επέκταση dock"),
    "sr":  ("Skupi dock", "Proširi dock"),
    "cs":  ("Sbalit dock", "Rozbalit dock"),
}

I18N = "/home/z/pc-edit/crates/ui-egui/src/i18n"
ANCHOR = "Show/Hide Panels"


def ensure(path, collapse, expand):
    with open(path, encoding="utf-8") as fh:
        lines = fh.read().splitlines(keepends=False)
    have = {tuple(l.split("\t")[1:2]) for l in lines if l.startswith("\t")}
    out, added, anchored = [], 0, False
    for l in lines:
        out.append(l)
        if l.startswith("\t") and ANCHOR in l:
            for src, tr in (("Collapse Panel Dock", collapse), ("Expand Panel Dock", expand)):
                if (src,) not in have:
                    out.append(f"\t{src}\t{tr}")
                    added += 1
            anchored = True
    if not anchored:
        for src, tr in (("Collapse Panel Dock", collapse), ("Expand Panel Dock", expand)):
            if (src,) not in have:
                out.append(f"\t{src}\t{tr}")
                added += 1
    if added:
        with open(path, "w", encoding="utf-8") as fh:
            fh.write("\n".join(out) + "\n")
    return added


def main():
    total = 0
    for lang, (collapse, expand) in sorted(ROWS.items()):
        path = os.path.join(I18N, f"{lang}.tsv")
        if not os.path.isfile(path):
            print(f"MISSING catalog: {path}")
            continue
        n = ensure(path, collapse, expand)
        total += n
        print(f"{lang}: +{n}")
    print(f"total rows added: {total}")


if __name__ == "__main__":
    main()
