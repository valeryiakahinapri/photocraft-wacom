#!/usr/bin/env python3
"""OKLCH -> sRGB hex conversion for modern-canvas-studio tokens."""
import math

def oklch_to_srgb(L, C, H, alpha=1.0):
    h = math.radians(H)
    a = C * math.cos(h)
    b = C * math.sin(h)
    l_ = L + 0.3963377774 * a + 0.2158037573 * b
    m_ = L - 0.1055613458 * a - 0.0638541728 * b
    s_ = L - 0.0894841775 * a - 1.2914855480 * b
    l = l_ ** 3; m = m_ ** 3; s = s_ ** 3
    r = +4.0767416621 * l - 3.5076117273 * m + 0.4308633970 * s
    g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s
    bl = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s
    def gamma(x):
        if x <= 0: return 0.0
        if x >= 1: return 1.0
        return 1.055 * x ** (1/2.4) - 0.055 if x > 0.0031308 else 12.92 * x
    r8 = round(gamma(r) * 255); g8 = round(gamma(g) * 255); b8 = round(gamma(bl) * 255)
    return r8, g8, b8, alpha

tokens = [
    ("background (page, island gaps)", (0.155, 0.004, 260)),
    ("workspace (canvas area)",        (0.125, 0.003, 260)),
    ("surface (island fill)",          (0.205, 0.005, 260)),
    ("surface-2 (hover/pill)",         (0.245, 0.006, 260)),
    ("surface-3 (active/selected)",    (0.29,  0.007, 260)),
    ("foreground (text)",              (0.94,  0.004, 260)),
    ("popover (flyout)",               (0.23,  0.006, 260)),
    ("primary (blue accent)",          (0.68,  0.16,  250)),
    ("primary-foreground",             (0.99,  0.0,   0)),
    ("muted-foreground",               (0.68,  0.008, 260)),
    ("faint (disabled)",               (0.5,   0.008, 260)),
    ("accent (selection bg)",          (0.3,   0.008, 260)),
    ("destructive",                    (0.64,  0.2,   25)),
    ("gradient-top",                   (0.72,  0.16,  250)),
    ("gradient-bottom",                (0.6,   0.17,  255)),
]
print(f"{'token':38s} {'hex':9s}  egui Color32")
for name, t in tokens:
    r, g, b, _ = oklch_to_srgb(*t)
    print(f"{name:38s} #{r:02X}{g:02X}{b:02X}     Color32::from_rgb({r}, {g}, {b})")

print()
print("== alpha composites (white-alpha overlays over their base) ==")
def over(white_a, base_rgb):
    return tuple(round(255 * white_a + b * (1 - white_a)) for b in base_rgb)

base_surface   = oklch_to_srgb(0.205, 0.005, 260)[:3]
base_bg        = oklch_to_srgb(0.155, 0.004, 260)[:3]
base_popover   = oklch_to_srgb(0.23, 0.006, 260)[:3]
base_workspace = oklch_to_srgb(0.125, 0.003, 260)[:3]

for label, wa, base in [
    ("hairline 5% white over surface", 0.05, base_surface),
    ("border    7% white over surface", 0.07, base_surface),
    ("input     9% white over surface", 0.09, base_surface),
    ("border    7% white over background", 0.07, base_bg),
    ("hairline  5% white over workspace", 0.05, base_workspace),
    ("border    7% white over popover", 0.07, base_popover),
]:
    r, g, b = over(wa, base)
    print(f"{label:44s} #{r:02X}{g:02X}{b:02X}")

r, g, b = over(0.10, base_surface)
print(f"{'scrollbar thumb white10% over surface':44s} #{r:02X}{g:02X}{b:02X}")

for L in (0.3, 0.24):
    r, g, b, _ = oklch_to_srgb(L, 0.0, 0.0)
    print(f"checker L={L}: #{r:02X}{g:02X}{b:02X}")

r, g, b = over(0.04, base_workspace)
print(f"{'grid dot white4% over workspace':44s} #{r:02X}{g:02X}{b:02X}")

pr, pg, pb, _ = oklch_to_srgb(0.68, 0.16, 250)
sel = tuple(round(pr * 0.15 + s * 0.85) for s in base_surface)
print(f"{'layer selected: primary 15% over surface':44s} #{sel[0]:02X}{sel[1]:02X}{sel[2]:02X}  (primary=#{pr:02X}{pg:02X}{pb:02X})")
