#!/usr/bin/env python3
"""Insert 'Hide Dock' / 'Show Dock' rows into all ui-egui i18n catalogs.

Format (context<TAB>source<TAB>translation, context empty for plain strings).
Rows go right after the 'Show/Hide Panels' row when present, else appended
under a comment. Translations follow each catalog's existing dock/panel
vocabulary (see the 'Dock Panel' / 'Enable floating document window docking'
rows in the same files).
"""
import os, sys

ROWS = {
    "de":  ("Dock ausblenden", "Dock einblenden"),
    "fr":  ("Masquer le dock", "Afficher le dock"),
    "ja":  ("ドックを非表示", "ドックを表示"),
    "zh-hans": ("隐藏停靠栏", "显示停靠栏"),
    "zh-hant": ("隱藏停駐欄", "顯示停駐欄"),
    "ru":  ("Скрыть панели", "Показать панели"),
    "es":  ("Ocultar el dock", "Mostrar el dock"),
    "it":  ("Nascondi il dock", "Mostra il dock"),
    "ko":  ("독 숨기기", "독 표시"),
    "pt-br": ("Ocultar o dock", "Mostrar o dock"),
    "nl":  ("Dock verbergen", "Dock tonen"),
    "pl":  ("Ukryj dock", "Pokaż dock"),
    "tr":  ("Panelleri gizle", "Panelleri göster"),
    "uk":  ("Приховати панелі", "Показати панелі"),
    "vi":  ("Ẩn bảng điều khiển", "Hiện bảng điều khiển"),
    "id":  ("Sembunyikan dock", "Tampilkan dock"),
    "hi":  ("डॉक छिपाएँ", "डॉक दिखाएँ"),
    "hu":  ("Dock elrejtése", "Dock megjelenítése"),
    "el":  ("Απόκρυψη dock", "Εμφάνιση dock"),
    "sr":  ("Sakrij dock", "Prikaži dock"),
    "cs":  ("Skrýt dock", "Zobrazit dock"),
}

I18N = "/home/z/pc-edit/crates/ui-egui/src/i18n"
ANCHOR = "Show/Hide Panels"

def ensure(path, hide, show):
    with open(path, encoding="utf-8") as fh:
        lines = fh.read().splitlines(keepends=False)
    have = {tuple(l.split("\t")[1:2]) for l in lines if l.startswith("\t")}
    out, added, anchored = [], 0, False
    for l in lines:
        out.append(l)
        if l.startswith("\t") and ANCHOR in l:
            for src, tr in (("Hide Dock", hide), ("Show Dock", show)):
                if ("Hide Dock",) not in have and ("Show Dock",) not in have:
                    out.append(f"\t{src}\t{tr}")
                    added += 1
            anchored = True
    if not anchored:
        for src, tr in (("Hide Dock", hide), ("Show Dock", show)):
            out.append(f"\t{src}\t{tr}")
            added += 1
    if added:
        with open(path, "w", encoding="utf-8") as fh:
            fh.write("\n".join(out) + "\n")
    return added

total = 0
for code, (hide, show) in sorted(ROWS.items()):
    path = os.path.join(I18N, f"{code}.tsv")
    n = ensure(path, hide, show)
    total += n
    print(f"{code}: +{n}")
print(f"total rows added: {total}")
