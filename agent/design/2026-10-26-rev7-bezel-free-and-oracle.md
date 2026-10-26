# REV7 — Modern Canvas, second adoption pass (bezel-free + one-surface right side) + the A/B oracle

**Date:** 2026-10-26 · **Status:** draft for audit
**Inputs:** mockup @ 73e735b ("Refined bezels & radius scale" + "Unified rounded radius system"), owner directives of 2026-10-26, egui 0.37 base (merge cb8634b06).

## 0. Why a second pass

The owner rejected the REV6 result as "still very far off" after the upstream-integration build
(which, unknown to us at the time, did not even contain REV6 — the Copilot merge took upstream
main only). Two separate threads meet here:

- **T1 — the port:** feature branch (REV5 mechanics + REV6 visuals, egui 0.36) had to land on
  the new main (egui 0.37). Done: merge cb8634b06, lib compiles clean, all eight conflicts
  resolved keeping our structure and their 0.37 idioms.
- **T2 — the mockup moved:** the owner simplified the visual language further. The delta from
  the sheet REV6 implemented is structural, not just token values. This doc specifies the
  second pass (REV7) and the pixel-exact A/B "oracle" infra the owner asked for.

## 1. What changed in the mockup (facts from source, not memory)

Token sheet (`styles.css`, 2026-10-26):

| token | REV6 value | mockup now | sRGB |
|---|---|---|---|
| `--radius` | 5 | **12** | 12/10/8/6 scale below |
| `--background` | 0.14 0.004 260 | 0.135 0.004 260 | `#07090A` |
| `--workspace` | 0.115 0.003 260 | 0.11 0.003 260 | `#040505` |
| `--surface` | 0.20 0.005 260 | 0.19 0.005 260 | `#131416` |
| `--surface-2` | 0.235 0.006 260 | 0.245 0.006 260 | `#1F2124` |
| `--surface-3` | 0.27 0.007 260 | 0.29 0.007 260 | `#2A2C2F` |
| `--popover` | 0.22 0.006 260 | 0.23 0.006 260 | `#1C1E20` |
| `--primary` | 0.65 0.16 250 | 0.68 0.16 250 | `#5EB9FE` |
| `--muted-foreground` | — | 0.68 0.008 260 | `#969A9E` |
| `--faint` | — | 0.5 0.008 260 | `#616568` |
| `--border` | — | white 5% | hairline material |
| `--hairline` | — | white 3.5% | |
| `--input` | — | white 9% | |

Radius usage (from `Shell.tsx`, all literal): islands + document + flyout + task bar = **12**;
segmented track + status bar = **9**; active doc tab = **8**; IconBtn/pill/menu/tab/layer-row =
**7**; segmented pill + thumb/inner chips = **6**. One ladder, consistently applied — this is
the owner's "everything consistent on radius".

Shadows: island = inset white-4% top line + `0 10px 30px -14px black/55%` + `0 1px 3px black/30%`;
canvas image = `0 24px 70px -24px black/80%`; active tool = inset primary-50% ring +
`0 4px 14px -4px primary/55%`. Dots grid = white-2.5%, 18px pitch.

Structural facts (DOM-measured at 1440×900):

- page: `p-2` (8px) padding, `gap-2` (8px) between the three rows.
- **title bar h-10 (40px), TRANSPARENT — sits directly on the page background.** Only its
  search field is a small island (h-8, surface fill, radius 9, island shadow). Menus and
  icon buttons are transparent with surface-2 hover pills.
- options bar: island h-11 (44px) full width.
- main row: left toolbar island (**80px expanded / 48px collapsed, width-tween 300ms**),
  document `rounded-12 bg-workspace` with the dot grid, right island 360px.
- **the right island CONTAINS the 44px rail AND the 316px panel column in one surface.
  No seam, no hairline, no border between them.** Panel cards separate only by
  `divide-y divide-foreground/[0.04]` (white 4%). The REV6 seam hairline must go.
- document tab strip: transparent on workspace; active tab = `bg-surface/70 rounded-8`
  pill with primary status dot + zoom/color suffix + close X; inactive = transparent,
  hover surface-2, radius 7.
- status bar: FLOATS inside the workspace, `inset-x-2 bottom-2`, radius 9,
  `bg-surface/70 backdrop-blur`.
- left toolbar (expanded): 2-col grid of 32px buttons (radius 7), `h-px w-4` separators at
  6% white between groups, gradient pill + 3px primary left bar on the active tool,
  colour chips + collapse toggle at the bottom.
- right rail: 28px icon buttons (radius 7), `h-px w-4` separators, collapse toggle on top.

## 2. REV7 — the change plan

### D1 Token retune (theme.rs `ModernCanvas` block)

Replace the REV6 values with the sheet above. Radius ladder: `radius_sm=6, radius=7,
radius_lg=9` (wait — see A3 below; final: sm 6, md 7, lg 9, island 12 via
`chrome_panel_radius`/`paint_island` reading a new `radius_island=12` token). Add
`hairline` (white 3.5% over surface ⇒ precomputed solid per fill) and `surface_2/3` map onto
the existing `hover/pressed/field` slots. `popover` = `#1C1E20`. Keep `page`, `islands`,
`popover` fields from REV6; add `workspace` (the doc area fill, REV6 used canvas for this —
now split: workspace `#040505` for the doc area, canvas stays the image backdrop).

### D2 Bezel-free law (the owner's #1 caveat, restated)

No `card_border` strokes anywhere in the ModernCanvas block except: (a) the white-alpha
hairlines that the mockup itself has (segmented ring, divide-y between cards, rail/toolbar
group separators at 6%, popover edge at hairline 3.5%); (b) NOTHING on islands, cards, doc
tabs, pills. Where REV6 drew a border, REV7 either drops it or replaces it with a shade step
(surface → surface-2 → surface-3). All existing border fields stay populated for the other 14
themes; ModernCanvas sets them to `Color32::TRANSPARENT`/`Stroke::NONE` and the hairline
materials go through the new `hairline` token.

### D3 Title bar becomes transparent chrome

`title_bar()` in panels.rs: under `t.islands`, drop the island frame entirely — the panel
frame becomes `Frame::NONE` (no fill, no rounding) so the bar paints directly on the page.
Widgets keep their hover pills (surface-2, radius 7). The app's real title-bar content (logo,
menus, doc tabs in pro mode, window buttons) stays — only the surface changes. The search
field: the app has no search; skip (content, not chrome).

### D4 The right side is ONE surface (kill the seam)

Keep the REV6 segment mechanism (two egui panels painting halves of one rounded rect — it is
E2E-proven and 0.37-safe), but: (a) delete the seam hairline; (b) rail and column frames both
fill `surface` (not page!) — the whole right side is a single `surface` island with radius 12
on the OUTER corners only; (c) the rail's screen-edge margin animation (fold reveal) keeps
working: when collapsed, the rail island rounds ALL corners (complete island); when expanded,
the column carries the right rounding and the rail carries only the left. (d) The dock's
`divide-y` between panel cards: hairline white-4% over surface (precomputed solid
`#1E2022`-ish; exact value in the spec table).

### D5 Document area = the dark workspace island

`canvas.rs` + panels.rs central panel: the document region paints `workspace` (#040505) with
radius 12 (island corners), the dot grid (white 2.5%, 18px pitch — REV6 has it at pitch 18
already), and the tab strip transparent on it. Active tab pill = `surface/70 over workspace`
solid = `#0E1011`, radius 8, primary dot, radius-7 hover for inactive. The doc image keeps
`shadow-canvas`. Status bar moves INSIDE the workspace as a floating pill: `surface/70` solid
`#0E1011`, radius 9, `inset-x-2 bottom-2` (8px inset, 8px from bottom), monospace readouts.

### D6 Left toolbar: 2-col expanded / 1-col collapsed + restyle

The toolbar already folds via `toolbar_double` (two-column when doubled). Map the mockup's
grammar onto it: expanded (double) = 80px 2-col; collapsed (single) = 48px 1-col; the header
chevron becomes the collapse toggle (PanelLeftClose/Open semantics, moves to the bottom like
the mockup). Buttons 32px (bx=32 → w1 recompute), radius 7 via `t.radius`, separators
`h-px w-4` at 6% white, active tool = gradient pill + 3px primary bar at the reading-start
edge (paint_tool_gradient gains the bar). Island frame radius 12, fill surface, no borders.

### D7 Options bar retune

Keep the island (radius 12, surface fill, island shadow, no border); inner controls get the
radius ladder (pills 7, groups 7 with surface-2 track, switches primary). Padding `px-1.5`.

### D8 Widget radius consistency sweep

`icon_button` 7, `pill_tab` 6 on track 9, `value_field` 6, `card_ex` cards borderless radius 9
(card = a section on the island, radius between divide hairlines), layer rows 7, thumbs 6,
segmented in panel headers 9/6. One pass over widgets.rs/tab_strip.rs/dock.rs replacing the
`radius_sm/radius` uses with the ladder.

### D9 What deliberately does NOT change

The rail/fold/peek MECHANICS (menus::invoke dispatch, dock_expanded split, show_collapsible
animation, ⇧Tab), the 14 other themes, the engine prefs schema (ModernCanvas id
`modernCanvas` stays), i18n ids, the storybook/annotakit bake pipeline.

## 3. The oracle — pixel-exact A/B against the mockup

**Goal (owner's words):** "create exact-ness between egui and the ui mockup, so we can do
pixel level match… you may have to change the ui mockup so it becomes oracle… html mockup is
10x-100x easier to build and iterate… a one-time investment."

### 3.1 Why not naive full-image diff

Fonts (Geist vs the app's default), icon sets (lucide vs photocraft's), text content and the
document image itself can never match bit-for-bit. A full-image diff would always fail and
teach us nothing. What CAN match exactly: every fill colour, every region boundary, every
radius, every gap, every hairline — the chrome. So the oracle compares the CHROME exactly and
masks content.

### 3.2 The oracle state (deterministic on both sides)

- Viewport **1440×900** (mockup's measured design size).
- App: theme locked `modernCanvas` (URL seed or prefs seed), dock expanded, standard panels,
  one document open at a fixed size (seed via `Ctrl+N` create flow), English locale, zoom
  fitted. Recorded as a one-page "state spec" JSON.
- Mockup: a new route/state (`/?oracle=1` or an OracleShell variant) that renders the same
  chrome decisions the app makes where the mockup was free (doc tab title text = the app's
  default "untitled-1", panel card set = the app's default dock cards, status bar text = the
  app's real readouts). The mockup is CHANGED to match the app where they differed — that is
  the owner's instruction ("change the ui mockup so it becomes oracle").
- Translucency/blur pinned to solids: `surface/70` → the precomputed composite (e.g.
  `#0E1011`), `popover/95 + backdrop-blur` → solid `#1B1D1F`-family. Both sides render the
  same solid. (Browser blur has no egui equivalent; compositing over a flat fill is what the
  blur visually produces there anyway.)

### 3.3 The spec (single source of truth)

`tools/oracle/spec.json` (in the photocraft repo, consumed by the harness): tokens (sRGB),
region rects at 1440×900 (page, title, options, toolbar, document, tab strip, workspace,
right island, rail, panel column, status bar), probe points (flat-area pixel coords per
token), corner-radius measurement sites, content masks (rects excluded from pixel diff:
icon/text/image areas), tolerances (flat probe ≤ 2/channel; boundary ±1px; radius ±1.5px;
masked-diff ≥ 99% pixels within ΔE 2.5).

### 3.4 The harness (one command: `tools/oracle/run.sh`)

1. Render app: agent-browser → local hub `/photocraft?…oracle seed`, 1440×900, wait network
   idle + 2 repaints, screenshot PNG.
2. Render mockup: agent-browser → vite dev server `/`, 1440×900, screenshot PNG.
3. Compare (python + numpy, no browser deps):
   - **L1 token probes** — sample both PNGs at spec probe points; compare hex; report table.
   - **L2 boundaries** — for spec scanlines (e.g. y=200 crossing toolbar/doc/right island),
     detect fill-run transitions; compare x/y coordinates ±1px.
   - **L3 corners** — walk the AA edge at each island corner; fit radius; compare ±1.5px.
   - **L4 masked diff** — zero-out content masks in both; per-pixel ΔE (oklab); report
     % within 2.5, mean, max, and a heatmap PNG (side-by-side + diff).
   - **L5 fallback VLM** — only what L1–L4 cannot see (hover states, flyout animation feel);
     never the primary judge.
4. Report: `oracle-report.json` + `oracle-report.md` + PNGs; exit code 0/1 for CI-ability.
5. Verdict rule: PASS = L1 100%, L2 all ±1, L3 all ±1.5, L4 ≥99% in-tolerance. Anything else
   is a FAIL with a named region — the report says whether to fix the app (usual) or the
   mockup (oracle drift).

### 3.5 Why this is exact (and VLM is not)

L1–L3 are arithmetic on pixels — no judgement. L4 is arithmetic on 1.1M pixel pairs with
content masked. The only subjective surface left (L5) is the fallback the owner allowed.
The oracle also becomes the REGRESSION net for every future visual change: rerun one command.

### 3.6 Cost & risks

- Mockup edits: pin solids, add the oracle route, mirror the app's real chrome content.
  Half a day. The mockup stays the design surface (owner iterates HTML; we port to egui;
  the oracle proves the port).
- App-side determinism: fonts fixed, no animations at rest (tweens settle in <400ms; wait
  800ms), locale pinned. The dock width must become spec-exact (316px content width) and the
  toolbar 80/48 — those numbers are part of D4/D6 anyway.
- egui shadows/AA differ from CSS: islands are compared by fill+radius (L1/L3), shadows only
  in L4 where they bleed into unmasked area — tolerance absorbs it; if not, masks extend.
- Risk: the app's real panels have scrollbars/content the mockup lacks → masks cover.

## 4. Order of work

1. Oracle harness skeleton + spec.json from the MOCKUP side only (mockup renders, probes
   measured) — proves the tooling before the app changes.
2. REV7 D1–D8 implementation on the merged base.
3. Mockup oracle route alignment (solids pinned, chrome content mirrored).
4. A/B run → fix loop until PASS.
5. wasm release bake → hub → E2E (fold lifecycle + ⇧Tab regression) → owner review.

## 5. Audit questions for the fresh-context reviewers

- Does D4's segment scheme survive the no-seam requirement (two panels, one surface) — any
  egui 0.37 panel behavior that paints between SidePanels (separator line, resize bezel)?
- Is the radius ladder assignment (D8) consistent with every widget call site? Name any site
  the sweep misses (card_ex, pill_tab, tab_strip, dock.rs, panel_rail peek, dialog frames).
- Does D6's toolbar rework conflict with the existing `toolbar_double` persistence and the
  0.37 panel-state migration (PANEL_STATE_KEYS)?
- Oracle: is 1440×900 the right canonical size (vs 1280×800)? Is the PASS rule strict enough
  to catch the "arbitrary dark fills" class of bug (it should fail on any wrong-fill region)?
- Any place where a `surface/70 + blur` composite CANNOT be precomputed solid (busy
  background under the status bar) — what does the mockup pin instead?
