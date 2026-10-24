# REV6 — Modern Canvas Studio visual adoption (2026-10-24, session 4)

Owner directive: "the current result are still quite far off from this mockup … now the
codespaces live agent is no longer working you should now take over and do a full audit
and polishing round to fully apply these ui styling changes to the egui." The mockup
(github.com/valeryiakahinapri/modern-canvas-studio, `src/components/ps/Shell.tsx` +
`src/styles.css`) is the vision. The fold/unfold mechanics shipped in REV4/REV5 are
E2E-proven and stay; this round is the **visual** adoption: tokens, radii, fills,
island shell, per-element styling.

## Facts (verified this session)

- Current default theme is `ProMedium` (#535353 Photoshop grey) — `state.rs:1624`,
  engine `prefs.rs:309/311`. VLM audit of the live build: edge-to-edge grey strips,
  sharp structural corners, pill CTA buttons (h/2 radius), no islands, no gaps.
- Mockup grammar (from source, not memory): page = `--background` with 8px padding +
  8px gaps; every major region is an **island** (`--surface` fill, 5px radius, island
  shadow = inset hairlines + big soft drop) EXCEPT the title/menu row which floats
  transparently on the page; document island contains the tab row + a `--workspace`
  inset (6px margins, 5px radius, grid dots); right side = ONE island containing the
  44px rail (hairline right border) + 316px column; left toolbar = ONE island
  (80px 2-col open / 48px 1-col folded); flyout = 300px popover (6px radius,
  150ms fade+slide); panel cards collapse via 300ms grid-rows tween; active tool =
  blue gradient + 3px accent bar; active doc tab = **workspace shade** (owner's
  flip fix); radii everywhere small: 3/4/5/6px only.
- Tokens converted OKLCH→sRGB (scripts/oklch_tokens.py, CSS Color 4 matrices;
  first attempt had a corrupted inverse-matrix row — row sums must equal 1):
  background #0B0C0E · workspace #060607 · surface #161719 · surface-2 #1F2123 ·
  surface-3 #2A2B2F · popover #1C1D20 · foreground #EAEBEE · muted #96989D ·
  faint #616368 · primary #5E9CF6 · primary-fg #FCFCFC · gradient #6AA9FF→#5280E2 ·
  destructive #EB4A49 · hairline(5% white on surface) #222324 · border7% #262729 ·
  input9% #2B2C2E · checker #2E2E2E/#1F1F1F · grid dot #101011 ·
  layer-selected (primary 15% on surface) #212223.
- Theme system: `Tokens` struct (theme.rs) drives every widget; `pro` bool switches
  Photoshop-grammar vs Studio-grammar layout branches; engine `prefs.rs` owns the
  `Theme` choice enum (serde ids), web persists whole prefs under localStorage key
  `photocraft.preferences` (web.rs:457) — **the owner's browser holds theme=proMedium
  from prior rounds; a default change alone will not reach them**.
- Shell (lib.rs:2433-2452): title_bar/options_bar/status_bar = `egui::Panel::top/
  bottom(...).frame(Frame::NONE.fill(t.chrome))`; toolbar = start_panel fill t.canvas
  radius chrome_panel_radius; right dock = Panel::right fill t.canvas radius + margin 4;
  CentralPanel fill = pasteboard/t.canvas. Panels are full-bleed strips today.
- Radius sites are token-driven (widgets.rs paints buttons/fields/tabs with
  `t.radius_sm`; dock cards `t.radius`; peek window `t.radius_lg` + shadow).
  CTA pill exception at widgets.rs:1498 (`h/2` when pro).
- Canvas tabs: pro_tabs (flat strips, active=t.card) vs studio tabs (canvas.rs:2665).
  Active-tab flip must land in BOTH paths' ModernCanvas rendering.

## Decisions

**D1 — New theme `modernCanvas`, not a retune of Studio.** Additive: 14 existing
themes keep their identity; upstream merges stay cheap; the mockup gets an exact
home. Engine `Theme`/`DarkTheme` gain `ModernCanvas = "modernCanvas"`;
`ThemeKind::ModernCanvas` with `pro: false` (Studio rounded-card grammar is the right
base: pill_tabs panel headers ≈ mockup Segmented, non-pro status/options/title
renderers match the mockup rows), `round_chips: false`, `bevel: false`,
radii `radius_sm: 3.0, radius: 5.0, radius_lg: 6.0` (mockup's small-side radii).

**D2 — Token map (exact).** page(#0B0C0E)·canvas(#060607)·card/surface(#161719)·
hover/surface-2(#1F2123)·pressed/surface-3(#2A2B2F)·popover(#1C1D20)·text(#EAEBEE)·
text_dim(#96989D)·text_faint(#616368)·accent(#5E9CF6)·accent_text(#FCFCFC)·
separator/hairline(#222324)·card_border(hairline #222324)·field_border(#2B2C2E)·
field(#1F2123)·row_selected(#212223)·accent_soft(primary 15% alpha)·danger(#EB4A49)·
canvas_dot(#101011)·tab_strip(TRANSPARENT — islands carry their own bg)·
shadow(black ~40%)·histogram bg #060607. New Tokens field `page: Color32`
(defaults to each theme's chrome — only ModernCanvas differs; used by island mode).

**D3 — Island shell mode.** New Tokens bool `islands: true` (ModernCanvas only).
In lib.rs shell + panels, when `t.islands`: every chrome strip's frame becomes
`Frame::NONE.fill(t.page)` with outer margins (scheme below) and the strip's content
paints an island: rounded rect (t.card, radius 5, hairline inset stroke,
soft shadow). Margin scheme (points): screen edges 8; island gaps 8 (4+4 between
adjacent strips). title_bar [8,8,4,8] and its content stays TRANSPARENT (mockup's
title row is not an island); options_bar [4,8,4,8] → island; status_bar [4,8,8,8] →
slim island (mockup has no separate bottom strip — status floats inside the
workspace; **deliberate deviation**: keeping our bottom status strip, restyled as a
slim island, avoids restructuring status_bar into the canvas this round);
toolbar [4,4,4,4→(t4,b4,l8,r4)]; right dock [4,4,4,4→(t4,b4,l4,r8)];
CentralPanel fill t.page, margins [4,4,4,4] (l/r gaps sum 8 with side panels).

**D4 — Document island.** CentralPanel content (canvas.rs documents) under
`t.islands`: island rect (surface, radius 5, shadow, hairline) = available rect;
inside: tab row (h≈32) then workspace rect inset 6px (t.canvas fill, radius 5,
grid dots 18px pitch) holding the actual document canvas. Active doc tab = rounded
pill fill **t.canvas** (owner's flip: active uses the workspace shade) + primary
dot; inactive = transparent, hover surface-2; strip bg = island surface. Home/empty
state draws on the workspace shade inside the island.

**D5 — Right side = ONE island.** panel_docking's right panel under `t.islands`:
strip frame fill t.page margins (t4,b4,l4,r8); content island (surface radius 5)
contains rail (44px, transparent, hairline RIGHT border) + column (the existing
show_collapsible dock column). panel_rail's own island fill/shadow/outer_margin(4)
from REV4 is absorbed (rail no longer paints its own island background — it sits on
the dock island). Peek window: fill t.popover(#1C1D20), radius 6, island shadow,
header hairline; entrance fade+slide 150ms via animate_value_with_time on the Area
offset (bonus; ship only if it doesn't fight the existing lifecycle).

**D6 — Left toolbar island.** Strip fill t.page; content island radius 5;
active tool button = vertical gradient mesh #6AA9FF→#5280E2 + white icon + 3px
accent bar (rounded, left edge, #5E9CF6); inactive hover = surface-2; the old
"subtle violet wash" bottom gradient is suppressed for ModernCanvas (mockup has no
decorative washes). Two-col open / one-col folded keeps the existing toolbar_double
+ animated width (300ms ease-out already ≈ mockup).

**D7 — Widget consistency sweep.** (a) CTA/primary buttons: kill the h/2 pill for
ModernCanvas — radius_sm (mockup Share = 4px, radius-sm family). (b) All icon
buttons radius_sm, hover surface-2, active surface-3 (token-driven already).
(c) Segmented/pill panel tabs: container hairline + p-[3px] look via pill_tabs
token retune (active = surface-3 fill + hairline stroke, radius 3). (d) Fields:
fill surface-2, border input9% #2B2C2E, radius 3. (e) Value chips/pills in options
bar: surface-2 fill + hairline, radius_sm. (f) Scrollbars: thumb ~10% white
(#2D2E30), 10px track — egui ScrollStyle token retune where reachable.

**D8 — Default + localStorage migration.** Defaults flip ProMedium → ModernCanvas
(engine `choice!` defaults, `Prefs::default`, state.rs). One-shot migration pref
`interface.modernMigrated` (bool, default false): at prefs load, if !migrated &&
theme == proMedium (the old default) → theme = modernCanvas, migrated = true.
Users who explicitly chose another theme keep it. This is the only way the owner's
existing `photocraft.preferences` localStorage shows the new look.

**D9 — Scope control / deliberate deviations.** Status bar stays a bottom strip
(slim island) instead of moving into the canvas (mockup deviation, noted); no
restructure of toolbar groups to mockup's exact tool set/order; other 13 themes
untouched except mechanical `page` field default; no left-panel fold (mockup's
left fold = our existing toolbar_double, already animated); tween timings stay on
egui primitives (show_collapsible, animate_value_with_time 0.22-0.3s).

**D10 — Verification.** (1) `cargo check` + existing test suite green; add a prefs
migration unit test + a theme token test (ModernCanvas exact hexes). (2) Trunk
release bundle via tools/trunk-bundle.py, bake, agent-browser screenshots at
1440×860 expanded/folded/peek/home, VLM side-by-side against ref-mockup.png.
(3) Regression: fold→peek→escape→expand pixel lifecycle (REV5 E2E), ⇧Tab
round-trip, reload persistence, localStorage migration path (seed old prefs,
reload, assert theme flipped). (4) Zero console errors; wasm size sanity ~30MB.

## Implementation order (est. touched files)

theme.rs (tokens+kind+page field) → engine prefs.rs (enum+default+migration) →
state.rs (default) → lib.rs (shell frames/island mode) → panels.rs (title/options/
status/toolbar islands + gradient tool button) → panel_docking.rs (dock island,
default width 316) → panel_rail.rs (absorb island, hairline border, peek retune) →
canvas.rs + tab_strip.rs (doc island + active-tab flip) → widgets.rs (CTA radius,
field/pill retunes) → i18n rows (theme label) → tests.
