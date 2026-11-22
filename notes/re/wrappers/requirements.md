# Requirements: replicating the original strategy-UI framework (Track B1)

From `ui-framework.md` (facts read from the player's own exe, 2026-11-22). One item per
behaviour. "Engine now" cites the code as of `main` 2bc263b. Pack keys are **proposals** for
the spec agent (`interface` / `screens.<id>` keys; R-UI numbering is theirs). Every item must
also work in the original-free `default` campaign, which supplies its own generated art and
font. Tests: **U** = unit test (pure logic in `content::screens` / `app::widgets` layout maths),
**M** = manual case for `implementation/testing/manual-tests.md`.

Build order is the order below: assets → text → widgets → lists/dialogs → navigation → CSD.

## A. Assets

### RQ-UI-01 Sprite-table asset reference
- **Original:** 1527 rows {index, resource, hot_x, hot_y, keep_format, surface}; one image per
  row; hot -1 = 0, -2 = −size/2, else offset; table found by content (CD 0x69bc18, 1.2 0x69b598).
- **Engine now:** only `original:<Dll>#<id>` (`src/app/assets.rs:61`); no hot spots; screens.yaml
  `spr:N` is a resource id.
- **Gap:** no way to name a row, its hot spot, or "row + n".
- **Pack key:** `original:sprite#<index>` and `original:sprite#<index>+<n>`; a widget `anchor:
  hotspot|centre|topleft` (default from the row's hot spot). The reader lives in `formats`
  (table by signature, as `tools/uikit.py find_sprite_table`), errors as `Result`.
- **Default campaign:** plain PNG references with an explicit `anchor`.
- **Test:** U — synthetic 20-byte table in the test: signature search, group ends at resource 0,
  hot −1/−2/offset resolution, bad table → error.

### RQ-UI-02 Multi-state images as consecutive rows
- **Original:** a button = base index + state count S; up = +0, down = +S−1; S = 1 means no
  up image (the background shows it); disabled image separate (scroll arrows 62/65).
- **Engine now:** `images: [up, down, hover, disabled]` (`src/content/screens.rs:92`),
  `strategy.rs:1806-1813` picks 1 = down, **2 = hover** (the original has no hover image).
- **Gap:** S = 1 semantics, latched state, disabled image, no hover.
- **Pack key:** `images: {sprite: original:sprite#372, states: 1}` or the explicit list;
  `disabled:`; hover image optional and off for `interceptor`.
- **Default:** generated up/down/disabled PNGs.
- **Test:** U — state selection function (enabled, pressed, inside, latched, S) → image or none.

## B. Text (user playtest priority: "sized differently and aligned with the UI elements")

### RQ-UI-03 Bitmap sheet fonts
- **Original:** nine SpritesDLL sheet fonts (23000 … 23014); square cells, glyph = trimmed
  cell, advance = glyph width, space = width of glyph 31; colour baked into the sheet
  (fonts.yaml). A tenth 6 px built-in font for text-button captions, drawn in any colour.
- **Engine now:** one TTF from the skin, `draw_text_ex(font_size = size as u16)` (`src/app/ui.rs:126-145`),
  any size; `measure_text` widths; pack sizes 10-12 px (`packs/interceptor/rules/screens.yaml:22-25,42`).
- **Gap:** different glyph shapes, sizes and advances; truncation of `size` to u16; no
  per-font colour.
- **Pack key:** `fonts: {<name>: {sheet: original:SpritesDLL.dll#23010, kind: grid}}` and per
  widget `font: <name>`; `kind: grid` = the sheet rules (pitch from the grid colour, 154 glyphs,
  char map); mod packs may give `kind: ttf` + `size`. The 6 px font: `kind: builtin6`
  (advance table in the exe is data; re-implementing the glyphs is original content — so the
  engine needs its **own** 6 px pixel font with the same advances as a fallback, see default).
- **Default:** a generated grid-sheet PNG font (tools/default_assets) in the same format, so the
  sheet reader is exercised without original data; plus an own 6 px font.
- **Test:** U — synthetic sheet built in the test: pitch detection, glyph trim, advance, space
  rule, char map, 'ß' → "SS".

### RQ-UI-04 Text anchor rules
- **Original:** `DrawText(x, y, flags)`: flag 1 centre, 4 right; glyph top = y − 5 (y − 7 for
  23010/23011/23013/23015). `DrawTextRect`: one line → x = left or centre, **y = rect top +
  h/2** (vertically centred); wrap at spaces, `~` = break, line step = lh + lh/3.
- **Engine now:** baseline = `r[1] + r[3]*0.5 + size*0.35` (`strategy.rs:1790`, also
  `widgets.rs:133`); wrapped text starts at `r[1] + size` with step `1.3*size`
  (`strategy.rs:1843-1844`); no `~`.
- **Gap:** vertical position depends on TTF metrics instead of the glyph-top rule; wrong line
  step; centred blocks not centred vertically.
- **Pack key:** widget `anchor: [x, y]` (DrawText form, values from the exe) or `rect` +
  `align: left|centre|right` + `valign: centre|top` (DrawTextRect form); `font:`.
- **Default:** same keys with its own font.
- **Test:** U — layout function (font metrics, rect, flags) → glyph positions; cases from the
  exe: CSD time at (561, 370) centred → top 363.

### RQ-UI-05 CSD readouts and status line exactly
- **Original:** font 23010, centred on x 561, y 370 / 390 / 410 / 430: time HH:MM:SS
  (monospace), weekday, date (language format), funds (S_DOLLARS). Status line = map object
  name, centred in [344,457,620,472].
- **Engine now:** pack rects measured from the bars, size 11, colour dark red [0.2,0,0]
  (`packs/interceptor/rules/screens.yaml:22-25`); status left-aligned, size 10, green (line 42).
- **Gap:** font, colour (sheet is white with green edges), anchors, monospace time, status
  alignment.
- **Pack key:** `value` widgets with `font`, `anchor`, `align: centre`, `monospace: true`.
- **Default:** its own CSD values with the same keys.
- **Test:** U — `value(time)` format; M — MT: CSD readouts overlay the bars (screenshot compare
  against the original by eye).

### RQ-UI-06 Captions tied to their control
- **Original:** TextButton caption = 6 px font RGB(200,200,200) centred on the widget rect
  inflated (0,−1); message-box capsule caption = 23010 at (centre, centre − 2), pressed +1,+1;
  check-row label at left + 20; dialog caption (style 3) at top + 54, centred.
- **Engine now:** `widgets::button` picks `size = (h*0.55).clamp(11,22)` (`src/app/widgets.rs:130`)
  and centres a TTF caption; art buttons have no caption; labels are separate absolute widgets.
- **Gap:** captions are not computed from the owning control's rect, size scales with height.
- **Pack key:** `label: <key>` on the button itself (no separate text widget), `caption:
  {font, offset: [0, -2], pressed_offset: [1, 1]}` with these defaults per widget kind.
- **Default:** same, own font.
- **Test:** U — caption position from rect for each kind; M — dialogs and Bases text buttons.

## C. Widgets

### RQ-UI-07 Button press semantics
- **Original:** press plays the sound, shows down while inside; **callback on release**
  (anywhere) unless fire-on-press; auto-repeat delay/interval in 60 Hz ticks; disabled buttons
  draw their disabled image and ignore input.
- **Engine now:** action on `clicked_in(r)` (`strategy.rs:1834`), no repeat, no disabled state.
- **Pack key:** `fire_on_press: true`, `repeat: [20, 5]` (ticks), `enabled_when: <condition>`.
- **Default:** same.
- **Test:** U — a pure state machine (press/move/release/time) → events.

### RQ-UI-08 Toggles and radio groups
- **Original:** latched S = 2 buttons; the screen latches the chosen one, unlatches siblings;
  first latched at start (E&P 300-303, 400-401; Equip Ships 302/304/306); check rows with lamp
  431/432.
- **Engine now:** none in screen sets (spec mentions toggles/radio groups, #98).
- **Pack key:** `group: <name>`, `selected: true`, `kind: check`.
- **Default:** same.
- **Test:** U — group selection logic.

### RQ-UI-09 Interface sounds per control
- **Original:** sound index per button (sprite-states.md): Click 0 default, 3 capsules, 4 dial,
  11 arrows, 15/17/18/19 per screen; 22 = none; names are soundfx.ini keys.
- **Engine now:** no UI sounds in `draw_set`.
- **Pack key:** `sound: <soundfx key>` per widget, `interface.sounds.default: Click`.
- **Default:** generated or silent sounds under the same keys.
- **Test:** U — resolution of the per-widget key with fallback; M — listen.

### RQ-UI-10 Tooltips ("floaters")
- **Original:** per-control LangFile key (179 registrations; CSD keys in screens.yaml);
  shown when the cursor rests in the rect and no button is down; box font 23011, `~` lines,
  16 px/line, width + 20, at (cursor x, rect.top − 19) or below if < 4, clamped to x ≤ 630,
  translucent RGB(0,0,100) with a frame.
- **Engine now:** none (the "status line" value is a selection text).
- **Pack key:** `tip: <text key>` per widget; `interface.tooltip: {font, bg, level}`.
- **Default:** its own text keys.
- **Test:** U — placement/clamp maths; M — hover each CSD control.

### RQ-UI-11 Time dial
- **Original:** six 20×20 holes (ids 32000-32005) at the verified coordinates, tooltips
  S_TIME_5SECS … S_TIME_24HOUR, sound RotarySwitch, keys 1-6; needle/hole sprites per speed.
- **Engine now:** `speed(n)` buttons possible; no dial widget.
- **Pack key:** `kind: dial, holes: [[x,y]…], images: {needle: …, lamps: …}`.
- **Default:** generated dial art.
- **Test:** U — hole hit test; M — click each hole.

### RQ-UI-12 Lamps and animations
- **Original:** id-list animations with a tick interval: CSD status lamp 5060-5064 every 6
  ticks; corner lamps 1900-1909 (interval unknown).
- **Engine now:** no animated images.
- **Pack key:** `kind: anim, frames: [...], interval: 6` (ticks).
- **Default:** generated frames.
- **Test:** U — frame index at time t.

## D. Lists and dialogs

### RQ-UI-13 Compact list style
- **Original:** row font 23011, header 23010; row pitch = line height + 2; visible =
  round((h − 7)/(lh + 2)); `~` columns, `#` editable column, "..." truncation; pulsing green
  selection bar; pressed row amber; blink 500 ms; arrows 60/63 disabled at the ends, repeat.
- **Engine now:** panel lists with `line = 21*k`, `size = 15*k`, title and detail rows
  (`strategy.rs:2048-2086`), wheel scroll by 2.
- **Pack key:** `kind: list, style: compact, columns: "<template key>", font, header_font,
  up: <widget>, down: <widget>`.
- **Default:** same keys.
- **Test:** U — visible-row count, column split, truncation, pulse level(n).

### RQ-UI-14 Dialog frames and buttons
- **Original:** frame style 1/2/3 kits (rows 163-178 / 179-188 / 189-198), body fill 5006,
  capsule buttons (rows 258-263) or text buttons stacked from bottom − 25, sizes per dialog in
  screens.yaml; clock paused while open.
- **Engine now:** `widgets::modal` in the skin (`src/app/widgets.rs:164`).
- **Pack key:** `dialog: {frame: 1|2|3 | nine-slice image, buttons: capsule|text, size}`.
- **Default:** generated nine-slice frame.
- **Test:** U — button stacking layout; M — each dialog type.

## E. Navigation

### RQ-UI-15 History stack
- **Original:** max 29; opening the CSD clears it; a screen already in the history is not
  pushed again; a run of menu-bar switches pushes once; `replace` does not push; back pops.
- **Engine now:** unbounded stack, dialogs popped on full-screen open, de-dupe only against
  the top (`strategy.rs:1884-1902`).
- **Pack key:** action forms `open(x)`, `menu(x)`, `replace(x)`, `back`; `interface.history: 29`;
  the first screen (`campaign.strategy_screen`) is the "home" that clears.
- **Default:** same.
- **Test:** U — stack operations against the rules above.

### RQ-UI-16 Clock versus screens and dialogs
- **Original:** leaving the CSD pauses the game clock; the CSD resumes it; message boxes pause
  while open.
- **Engine now:** the clock is independent of the open screen.
- **Pack key:** `screens.<id>.clock: run|pause` (CSD run, others pause), dialog `clock: pause`.
- **Default:** same.
- **Test:** U — clock state after a sequence of opens/closes.

## F. CSD star map

### RQ-UI-17 CSD map details (partly unknown)
- **Original:** icon group 434-563 (per-kind icons, kind mapping unknown), labels in
  23006-23008 (derived), zoom levels with a "removed zoom level" message, recentre via the
  minimap; status line = hovered object name.
- **Engine now:** generic map drawing.
- **Gap/next RE:** read CSD.cpp 0x4431ad, 0x447a14, 0x44c240 and the icon selection.
- **Pack key:** `map: {icons: {<kind>: original:sprite#N}, label_font, zoom_levels}`.
- **Default:** generated icons.
- **Test:** U — world↔map transforms once known; M — CSD hover names.
