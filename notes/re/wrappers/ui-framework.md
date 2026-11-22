# The strategy-UI framework of Interceptor.exe (Track B1)

Read 2026-11-22 from the player's own `Interceptor.exe` (CD build, addresses below) and
`SpritesDLL.dll`, as data only. Facts only: ids, rectangles, timings, colours, table layouts,
addresses, short LangFile **keys**. No art, text or code is reproduced. Each fact is marked
**verified** (read from code or tables), **derived** (inferred from code plus pixels or context)
or **unknown**. Patch 1.2 is a full rebuild: the tables are found by content (see
`tools/uikit.py`), and the sprite and sound tables are byte-identical in both builds (verified).

Companion files: `sprite-states.md` (every sprite button: index, states, sound, repeat),
`fonts.yaml` (the nine sheet fonts' metrics), `requirements.md` (RQ-UI items), `screens.yaml`
(`ui_framework` block and the CSD text anchors).

## 1. The sprite table

**Location** (verified): `.data`, CD VA 0x69bc18, patch 1.2 VA 0x69b598; 1527 rows; row *i* has
index *i*. `tools/uikit.py sprites EXE [--sprites-dll DLL]` finds it by that signature.

**Row** (20 bytes, verified from the group loader 0x56e729 and the row loader 0x56ecc7):

| Offset | Type | Field | Meaning |
|---|---|---|---|
| +0 | i32 | index | == position (the signature) |
| +4 | i32 | resource | TARG id in any resource DLL (LoadImage 0x5fc999 searches the DLLs); **0 ends a group** |
| +8 | i32 | hot_x | -1 → 0 (left), -2 → -(width/2) (centre), anything else = pixel offset (e.g. -5, -6) |
| +12 | i32 | hot_y | as hot_x, with the height |
| +16 | u8 | keep_format | 0 → image converted to blit mode 3 (0x62af2c); 1 → kept as loaded. Set only on 434-563 and 842 (the CSD map-icon group: 8-bit colour-mapped and 16-bit icons) |
| +17 | u8 | surface | 1 → loaded as a whole DirectDraw surface (0x62fb32), not a sprite. Only 429, 430 (WrappersDLL 5005, 5006: the dialog body fills) |
| +18 | u16 | pad | 0 |

The hot spot is an **offset added to the draw position** (`obj+0x44/+0x48`, applied by
0x61ae83). Distribution: centre/centre 1031 rows, left/top 480, left/centre 14 (the text-button
end caps 148-161), two fixed offsets.

**One row = one image = one frame** (verified: every TARG resource is one TGA; no strips).
Multi-state and animated sprites are **consecutive rows**: a widget is given a base index and a
state count (§2.1), an animation an id list (§2.5).

**Groups** (verified, loader switch 0x56e8aa): 0 = 0-433, 1 = 434-1034, 2 = 1035-1068,
3 = 1069-1152, 4 = 1153-1272, 5 = 1273-1340, 6 = 1341-1412, 7 = 1413-1499, 8 = 1500-1525;
each ends on its resource-0 row. Group 0 is always loaded; the others per screen. Indices are
global, so a pack can address any row by index.

**Asset reference form (proposal):** `original:sprite#<index>` = the image of row *index*,
drawn at its hot spot; `original:sprite#<index>+<n>` = row index+n. The existing `spr:N`
notation in screens.yaml means **resource** N (`original:SpritesDLL.dll#N`); `sprite-states.md`
gives both.

## 2. Widget behaviour

### 2.1 Button base (vtable 0x65f420, ctor 0x417530) — verified

Fields: `+0x28` sound index (0 at construction = Click; TextButton sets 18), `+0x2c` cursor
inside, `+0x2d` pressed, `+0x30` state count S, `+0x34` latched (toggle/radio "on"),
`+0x38` fire on press, `+0x3c/+0x40` auto-repeat delay/interval (-1 = none), `+0x44` press
time, `+0x48` sprite group, `+0x4c` pre-callback, `+0x54` disabled sprite (433 = none),
window `+0xf` enabled (0x44c560 / 0x47a260 clear it), window `+0x14` callback(parent, id).

* **Press** (slots 2 and 10): if inside, play sound `+0x28` unless it is 22 (none); pressed = 1;
  stamp time; redraw; if fire-on-press, call the callback now.
* **Release** (slot 4): if pressed: pressed = 0; redraw; if not fire-on-press, call the callback
  (so a click fires on **release**, wherever the release is — the inside flag only changes the
  drawing).
* **Leave / enter** (slots 7 / 9): inside = 0 / 1, redraw while pressed (pressed image only
  while the cursor is inside).
* **Auto-repeat** (0x41767b, per frame): while pressed and inside, after `delay` ticks since the
  press and at most every `interval` ticks (one global last-repeat time 0x670c38), call the
  callback again. Ticks are 1/60 s (`0x61926a` = GetTickCount × 0.06). Every list scroll arrow
  uses delay 20, interval 5 (333 ms, then 12 per second) and fires on press.
* **Drawing** (0x41773f, 0x417e79, 0x417f81): disabled → disabled sprite if set, else state 0
  (only if S ≠ 1). Enabled → state S-1 if (pressed and inside) or latched, else state 0 —
  **and nothing at all when S = 1** (the background art is the up state). State images are
  drawn **centred on the widget rect** (the hot spot is temporarily set to -w/2,-h/2).
* **No hover image** exists (verified: no state is chosen on inside alone).
* The widget rect is the sprite size + 1 px on every side (SetRect(-1,-1,w+1,h+1)), centred on
  (cx, cy) for SpriteButtonC (0x417c6e) or at (x, y) for SpriteButtonTL (0x417d7c).

### 2.2 Toggles and radio groups — verified

A toggle/radio is an S = 2 sprite button whose `+0x34` latch is set with 0x53eb40 / cleared with
0x53eb70; the screen callback latches the chosen one and unlatches its siblings (Equipment &
Personnel 300-303 and 400-401, Equip Ships tabs 302/304/306, team-select 905). The first of a
group is latched at creation. TextButton2 (0x419997, vtable 0x65f600) is a check row: lamp
sprite 431 (off) / 432 (on) at the left, label 20 px to the right (§3.3).

### 2.3 Time dial (TimeDial 0x4cca65, Tools2.cpp) — verified

110 × 108 widget centred at (562, 304); six 20 × 20 hole hot spots (ids 32000-32005) centred on
the hole coordinates in navigation.md; each registers a tooltip key: S_TIME_5SECS, S_TIME_1MIN,
S_TIME_5MIN, S_TIME_30MIN, S_TIME_1HOUR, S_TIME_24HOUR (speed 0..5). Turning plays sound 4
(RotarySwitch). Keys '1'..'6' do the same. The per-speed rate is the key's meaning (game time
per real second: **derived** from the key names).

### 2.4 Lists (ListBox 0x51b7c6, ListBox.cpp) — verified unless marked

* Fonts: rows `+0x7a8` = 23011, header/other `+0x7ac` and `+0x7a4` = 23010 (setters 0x51fdea /
  0x51fe38 / 0x51fe86).
* Row height = row font line height (23011: 14, derived) ; **visible rows** =
  round((rect height − 7) / (line height + 2)).
* Columns: x offsets in `+0x198[]` from the template string (`~` separates columns, `#` marks an
  editable column; see screens.yaml Equipment & Personnel); cell text is truncated with "..."
  when wider than its column. Text is drawn left-aligned at the column x, vertically centred
  on the row.
* Selection bar: a full-row fill in green whose level pulses: counter n steps every 2 ticks,
  0..75; level = 150·(1 − n/74) for n < 37, 150·(0.5 + (n−37)/74) after (≈75-153 of 255,
  period ≈ 2.5 s). A pressed row flashes amber RGB(200,150,0) on alternate frames.
* Blinking rows: a flagged row's text toggles every 500 ms.
* Row icon sprites 295/296 (2450/2451, 16 × 16: check off/on, derived) and 416-419
  (17200-17203, 14 × 13).
* Scroll arrows are separate SpriteButtonC 60 (up, 27500/27501, disabled 27502) and 63 (down,
  27503/27504, disabled 27505), sound 11 (Beep8), repeat 20,5; they are disabled at the ends.
  Scroll step: one row per callback (derived: the callbacks call the list's line up/down).

### 2.5 Animations and lamps (Animation.cpp, class ctor 0x46de60) — verified

An animation holds an id list (0x46d548(count, ids)) and a position (0x46ddce(x, y)); each
frame it advances when now ≥ next, next = now + interval (`+0x28`, ticks). CSD: the status lamp
(618, 453) cycles 5060-5064 (an alternate set 5065-5069 exists, use unknown) every **6 ticks
(100 ms)**; the four map-corner lamps cycle SpritesDLL 1900-1909 at (59,70), (348,70),
(348,354), (59,354) — one frame every **4 ticks (≈ 67 ms)**, looping (0.67 s per cycle; `0x440240`, see
`../unknowns-2026-11.md` §1.4).

### 2.6 Interface sounds — verified

Table (40-byte rows {index, key[32], voices}, CD VA 0x68b2c0, patch 0x68ac40), keys map to
`soundfx.ini`: 0 Click, 1 MsgBoxOpen, 2 MsgBoxOpen2, 3 MsgBoxBtn, 4 RotarySwitch, 5-15 Beep1..
Beep13, 16 BeepLow, 17 sndBeep22, 18 sndBeep29b, 19 sndBeep3025; 22 = silent. Per control
(sprite-states.md): CSD screen buttons 17, launch buttons 18, scroll arrows 11, Equip Ships tabs
19, E&P category radios 17 and mode radios 19, Bases side buttons 17, Research download 15,
message-box capsules 3, text buttons 18, dial 4. Other screens play their own tables (title
0x68acd8, briefing 0x68ef30 with T4Button, UFOpedia 0x68fe78; flight 0x675b08, 48 rows).

### 2.7 Hover help ("floaters", Floaters.cpp) — verified

Not a status line: a **floating tooltip box**. `Window::AddTip(id, rect, text, x, y, show_now)`
= 0x48cf19 (179 calls; every key is in the scan output, the main ones are in screens.yaml
`tooltips`). On each frame 0x48d51f finds the tip whose rect contains the cursor; when the
cursor has stopped (same x, y as last frame) and no mouse button is down, it opens the tip;
moving out closes it. Box (0x4b2ed5): text in font 23011, lines split at `~`, width = widest
line + 20, 16 px per line, at (cursor x or the given x, rect.top − 19 − 16·(lines−1); if that is
above y = 4, rect.bottom), pushed left to stay inside x ≤ 630; background RGB(0,0,100) filled
translucently (level 75, 0x63634e), then a 1 px frame; one line is centred in the box.
The CSD status bar [344,457,620,472] shows the **name of the map object under the cursor**
(0x447921: font 23010, centred, DrawTextRect flag 1).

## 3. Text

### 3.1 Fonts — verified

Text is drawn with **bitmap sheet fonts from SpritesDLL**, not GDI (GDI's Times New
Roman/Arial/Courier/System helper 0x64453b is not used by the strategy screens). Nine fonts
are loaded at start (0x5ff91c) into objects 0x775c28 + n·0x2acc: 23000, 23006, 23007, 23008,
23010, 23011, 23013, 23015, 23014. Sheet layout (loader 0x5fd750): pixel (0,0) is the grid
colour, the bottom-right pixel the transparent key; the cell pitch is the first row whose
column-1 pixel is the grid colour; square cells, glyph box = pitch − 2 at (col·pitch+1,
row·pitch+1); 154 glyphs; char → glyph map at VA 0x6af488 (A-Z = 1-26, a-z = 27-52, digits
53-62, punctuation, accented Latin-1 93-141; 'ß' is drawn as "SS"); each glyph is trimmed, its
width is its advance; **space advance = width of glyph 31 ('e')**. Metrics per font in
`fonts.yaml`. Colour is **baked into the sheet** (no tint parameter): 23010 white with green
edges, 23011 white/grey, 23013 and 23015 yellow, 23014 large yellow, 23000 cream (derived from
the most frequent glyph pixels).

Uses (verified call sites): 23010 — CSD readouts, CSD status line, message-box captions,
list headers, title-screen texts, UFOpedia; 23011 — the general body font (lists' rows,
tooltips, CSD panels, Bases info, reports' titles, dialogs); 23013 — Research values, Bases
view-mode labels (selected), E&P; 23015 — Equip Ships, Production, Bases highlights; 23014 —
Base Stats page titles; 23006/23007/23008 — CSD map labels (derived from CSD-only use).

A tenth, **built-in 6 px font** is compiled into the exe (glyph routines 0x614b8c, advances
0x6174a4: 2-9 px, e.g. 'I' 2, 'A' 6, 'W' 8, space 3) and drawn in any 16-bit colour
(`PortDrawString` 0x5ffa81). It is used for **text-button captions** (§3.3), the report graphs,
the briefing grid labels, the title-screen button labels and the CSD scale text.

### 3.2 Placement rules — verified

* `DrawText(surface, text, x, y, flags)` = 0x5fe4ad: flag 1 → x −= width/2; flag 4 → x −=
  width; no other flags. **y is not the top**: glyph top = y − 5 for glyph < 93 and y − 4 for
  accented glyphs, minus a further 2 for 23010/23011/23013/23015 (i.e. top = y − 7). A
  monospace flag (0x44c420) centres each glyph in a max-width cell.
* `DrawTextRect(surface, text, rect, flags)` = 0x5fed2b: one line (fits and no `~`): x = left
  (+ width/2 if flag 1, centred), y = top + height/2 → the text is **vertically centred** in
  the rect. Otherwise word-wrap at spaces, `~` forces a break; line step = line height +
  line height/3; flag 1: block centred vertically on the rect, else first y = top + line
  height/2 + line height; +4 for the four small fonts. No clipping beyond the wrap (a too-long
  block overflows).
* Small font: x −= width/2 with flag 1 (and y −= 2), x −= width with flag 4; y is the glyph top.

### 3.3 Captions belong to their controls — verified

* **TextButton** (0x419122, vtable 0x65f5b0, paint 0x41932d): label rect = widget rect
  inflated by (0, −1); caption in the **6 px font, RGB(200,200,200), centred on the label-rect
  centre**. Frame: the rect inflated by (0,−4); when pressed, filled RGB(0,200,0)
  translucently (level 50); two horizontal lines RGB(0,150,0) along top and bottom (rect
  inflated (−5,+2)); end caps sprites 151/154 (left up/down: 29203/29220) at (left, centre y)
  and 158/161 (29209/29221) at (right − cap width − 1, centre y). Creation animation: the caps
  start at the centre and slide apart 10 px every 2 ticks until (width − 20), with cap frames
  148+n / 155+n, n = 0..5 every 3 ticks; then static.
* **Message-box capsule** (SpriteWidget 0x418191, paint 0x41835a): left cap sprite N (258 =
  6520), tiled middle N+1, right cap N+2; pressed N+3..N+5 (6523-6525). Caption in **font
  23010, centred at (rect centre x, centre y − 2)**, pressed (centre x + 1, centre y − 1).
* **TextButton2** (check row): lamp 431/432 at the left, label in the 6 px font at x = left +
  20, colour member `+0x6c` or RGB(200,200,200).
* **CSD readouts** (0x43fb0f): font 23010, x = centre of rect [493,363,629,442] = 561, flag 1;
  y = 370, 390, 410, 430 (glyph tops 363, 383, 403, 423). Order: time "HH:MM:SS" (monospaced),
  weekday ((days+5) mod 7), date (per-language format), funds (S_DOLLARS).
* Research: topic texts at (row origin + 10, + 11) and (+15, + 22) in 23000; values in 23013
  with labels in 23011 (S_RES_STATUS, S_RES_FILESIZE, S_RES_AMOUNT, S_RES_COMPLETION).
* Bases view-mode labels: DrawTextRect centred in the three 152 × 18 slots, 23013 when selected
  and 23010 otherwise. Base Stats page title in 23014. UFOpedia category title at (475, 61)
  centred in 23010.
* Dialog caption (frame style 3): window top + 54, centred on the client rect, font member
  `+0x614` (per dialog).

## 4. Dialog frames (MsgBox.cpp) — verified unless marked

The MsgBox paint 0x518604 intersects the dirty rect with four frame strips around the client
rect (`+0xe0..+0xec`), draws the body (0x519622, fill from surface row 430 = 5006, derived), the
frame (0x51941f) and per style (`+0xf4`): style 1 = rows 163-178 (6560-6575, 16 pieces; the
style-1 branch also places row 171 twice, mirrored about the centre); style 2 = rows 179-188
(6580-6589; its branch places rows 181, 182, 185, 186, 187, 188 and 189, stacked by their
heights); style 3 = rows 189-198 (6590-6599, red) with a top ornament row 198 at (centre − 54,
top − 47). Which row is which corner/edge is **derived** from the art, not settled here and the caption
(§3.3). Buttons are message-box capsules (§3.3) or TextButtons 20 px tall, width − 20, stacked
upward from bottom − 25 (Interceptor Orders, max 6). Opening plays MsgBoxOpen (1) — derived from
the table name; the capsule press plays 3. A message box **pauses the game clock** while open and
resumes it on close (0x4120b7 / 0x41207f, two calls each in MsgBox.cpp, derived pairing).

## 5. Navigation semantics — verified (0x4b5bfe-0x4b5e5b)

* History: up to **29** screen ids at 0x6faa80, count 0x68236c.
* `push(n)` 0x4b5c4e: n = 17 (CSD) → **clear the history** and switch (no pause). Otherwise, if
  n ≠ current: pause the clock, push current **unless it is already in the history or the
  history is full**, switch.
* `menu(n)` 0x4b5cf2: like push, but only the **first** of a run of menu-bar switches pushes
  (flag 0x682370); later menu switches replace.
* `replace(n)` 0x4b5bfe: pause the clock, switch, no push. `back()` 0x4b5da5: pop and switch.
* The clock runs on the CSD only: every switch away pauses it (0x4120b7); the CSD resumes it
  (21 resume sites in CSD.cpp, derived).

## 6. The CSD star map (what this pass added)

* Readouts: §3.3 (order verified). Status line: object name under the cursor (§2.7).
* Tooltips: pan control S_CSD_PANCONTROL, view knob S_CSD_VIEW, launch probe/mission,
  minimap S_CSD_MINIMAP (rect member +0x555c), the six screen buttons (S_CSD_BASES,
  S_CSD_RESEARCH, S_CSD_STATUSREPORTS, S_CSD_DATABANK, S_CSD_EQUIPSHIP, S_CSD_EQUIPPERS), panel
  close S_CSD_CLOSEPANEL, the six report buttons (S_CSD_INCOME_REPORT … S_CSD_PLAYER_REPORT),
  the nine UFOpedia buttons DATABANK_CATEGORY1-9.
* Map icons, labels, zoom levels, the world↔screen transform and the map counters: **settled** in
  `../unknowns-2026-11.md` §1 (galaxy 60,000 units square; zoom 0 = 2.5 ly and 1 = 1.67 ly across the round
  333 px window; star icons by class; hover-only labels in the 6 px font; a removed zoom level shows a
  fixed message, DrawTextRect 23011).
* Corner and status lamps: §2.5.

## 7. Still unknown

The clock rate per
speed; which tooltips are disabled by an option (window flag +0xd7); the alternate status-lamp
set 5065-5069; exact glyph trim (assumed right/bottom trim of the key colour); the 23006-23008
roles; the per-dialog caption font (+0x614) for each dialog type.
