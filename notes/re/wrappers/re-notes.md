# How Interceptor.exe builds a strategy screen

Facts read from the player's own `Interceptor.exe` (32-bit PE, image base 0x400000, `.text`
VA 0x401000 / file 0x400; `.rdata` VA 0x65f000 / file 0x25dc00; `.data` VA 0x66f000 / file
0x26d800). Virtual address = file offset - section raw offset + section VA + 0x400000.
All code is MSVC 5/6 C++ (`thiscall`, `ecx` = this; SEH frames; asserts carry
`D:\ss\Yanceyville\Interceptor\<File>.cpp`, which names every screen's source file).

The UI DLL (`WrappersDLL.dll`) holds no code: only 581 `TARG` image resources.
`xcgen.dll` is the 3D engine and `XDaemon.dll` networking; neither touches the UI.

## Object model

* **Window base class** (Wrappers.cpp). Base vtable `0x65f650`, 17 slots (meanings *derived* from how CSD and Title override them; the slot numbers and base addresses are verified):

  | Slot (offset) | Base impl | Meaning |
  |---|---|---|
  | 0 (+0x00) | 0x48c613 | left button down (x, y) |
  | 1 (+0x04) | 0x48c62b | left button up (x, y) |
  | 2-3 | 0x48c643 / 0x48c65b | right button down / up (*derived*) |
  | 4 (+0x10) | 0x48c673 | mouse move (x, y) |
  | 5 (+0x14) | 0x48c68b | right/second button (x, y) (*derived*) |
  | 8 (+0x20) | 0x48c717 | **OnCommand(id)**: menu ids, hot keys (ASCII), child-widget ids |
  | 12 | - | scalar deleting destructor |
  | 15 (+0x3c) | - | per-frame update / paint of the frame (dialogs: 0x518604 draws the frame kit) |
  | 16 (+0x40) | - | paint / update |

  Child-window event types are dispatched by `0x48dc6d` (switch on type 0..8 -> slots 3,4,5,6,9,10,12,11).
  A window's own rect is at `this+0x10..+0x1c`; dialogs keep their client rect at `+0xe0..+0xec`.
* **Screens** are subclasses created by a factory switch in `0x4b54c5` on the global
  current-screen id `0x682368` (table in navigation.md). Pattern, e.g. CSD:
  `0x437730`: `new(0x638c)` -> ctor `0x4377f0` (sets vtable `0x65fbc0`) -> `Init 0x438101`
  -> `0x48a621`, `0x48a321` (attach to the window manager).
* **Init** loads the full-screen background with
  `Surface::LoadResource(id)` = `0x62fb32` (`push id; lea ecx, surface; call 0x62fb32`) and then
  blits it into the screen's back surface (`0x62c33d`). Failure strings: (an error message about the CSD art failing to load).
* **Fixed rectangles** are member RECTs filled with `USER32!SetRect` (IAT 0x7bc980) and moved with
  `OffsetRect` (IAT 0x7bc97c). Hit tests are inlined `left <= x <= right && top <= y <= bottom`
  compares against those members inside the mouse slots (no `PtInRect`).
* **Widgets** are child windows. Every widget's Create ends in
  `Window::CreateChild(id, RECT*, parent)` = `0x48dba7` (3 args). Known Create functions:

  | Fn | Kind | Arguments (in push order reversed = C order) |
  |---|---|---|
  | 0x417c6e | SpriteButtonC | (id, centre_x, centre_y, parent, sprite_index, states) -- rect = sprite size centred |
  | 0x417d7c | SpriteButtonTL | (id, x, y, parent, sprite_index, states) -- rect = sprite size at x,y |
  | 0x418191 | SpriteWidget (MsgBox buttons) | (id, x, y, parent, sprite_index, rect) |
  | 0x419122 | TextButton | (id, RECT*, label_text, parent) |
  | 0x419997 | TextButton2 | (id, RECT*, label_text, parent, extra) |
  | 0x4cca65 | TimeDial | (id, centre_x, centre_y, parent) -- 110x108, six holes |
  | 0x4cd8a4 | small toggle | (id, x, y, parent) |
  | 0x5067a8 | SpriteButton3 | (id, centre_x, centre_y, parent) -- sprite set beforehand |
  | 0x506c16 | SpriteToggle | (id, mode, x, y, sprite_index, parent) |
  | 0x511a9b | DropDown | (id, x, y, width, parent) -- the base selector at (475,13) w150 |
  | 0x513075 | ImageButton | (id, x, y, w, image_id_or_h, parent) -- loads its own image (0x5fc999) |
  | 0x5156ad | PanWidget | (id, x, y, ?, parent) -- loads 1800+n |
  | 0x51b7c6 | ListBox | (id, RECT*, scroll_x, scroll_y, parent) (-1,-1 = default scroll position) |
  | 0x56db90 | Spinner | 8 args (row quantity +/-) |
  | 0x5ec463 | EditBox | (id, RECT*, parent, flags) |

  After Create the screen calls, on the same widget object:
  `0x44c4c0(sprite_table)` (which sprite group to use, usually `0x722890`),
  `0x41765a(n)` (sound / tooltip index, *derived*), and
  `0x44c490(callback)` -- **callback(parent, id)** receives every click.
  Typical callback bodies are a `switch(id)` (MSVC compare tree + jump table, sometimes with a
  byte index table), e.g. CSD `0x4476eb` -> `0x445291`.
* **Screen switching**: `0x4b5c4e(n)` push+switch, `0x4b5cf2(n)` switch from menu,
  `0x4b5bfe(n)` replace, `0x4b5da5()` back (pop of the stack at `0x6faa80`).
* **Menu bar**: global widget `0x6f3638`, built from `S_MAINMENU` by `0x4a5ed3`
  (format `Label[id](submenu)`, `_` separator); callback `0x4a7fb6` (switch at 0x4a8ab5).
* **Strings**: code pushes the *key* (`"S_CSD_INTERCPORDERS"`) and calls `GetString` =
  `0x41a796`; message boxes are `0x51aacd(text, "(S_OK)(S_CANCEL)")`.

## Image loading

* `LoadImage(surface*, id, RECT* out, -1, flag)` = `0x5fc999` (cdecl, 5 args) loads any TARG id
  from whichever resource DLL owns it (Wrappers 5000-8132, Sprites 900-29xxx, Cockpit 12000+).
  949 call sites; loops use `base + i` (e.g. `5112 + i`, `7601 + i`).
* **Sprite table**: static array at VA **0x69bc18** (file 0x29a018), 20-byte records
  `{int32 index, int32 resource_id, int32 hot_x, int32 hot_y, int8 flag_a, int8 flag_b, pad}`,
  1527 entries (index == position; terminated where index != position). -2 hot spot = centre,
  -1 = top-left. It is split into 9 groups loaded per screen by `0x56e729(group)`:

  | Group | Index range | Table object | Loaded by |
  |---|---|---|---|
  | 0 | 0-433 | 0x722890 | global (0x570706) -- UI kit, frame kits |
  | 1 | 434-1034 | 0x722868 | CSD |
  | 2 | 1035-1068 | 0x7228b0 | Equipment & Personnel |
  | 3 | 1069-1152 | per-screen | Research |
  | 4 | 1153-1272 | 0x722880 | Equip Ships |
  | 5 | 1273-1340 | 0x7228c0 | Combat-sim setup |
  | 6 | 1341-1412 | 0x722848 | Bases |
  | 7 | 1413-1499 | 0x722858 | Bases, Equip Ships |
  | 8 | 1500-1525 | 0x7228a0 | multiplayer (NetPlayers, TeamSel) |

  Methods: `Draw(index, x, y, surface)` = `0x56f246`; `Width(index)` = `0x56f2f2`;
  `Height(index)` = `0x56f3ab`; `GetRect(index)` = `0x56f5d4`.
  WrappersDLL entries in it: indices 35-44, 163-198, 234-263, 363-370, 387-415, 429-430
  (group 0) and 891-892 (group 1); see catalogue.md.
* UFOpedia picture ids (5200-5356) are a table of `mov [reg+..], id` in `0x52eb7b`.

## Where the tables live (for load-time re-reading from the player's exe)

| What | Address (VA) | How to read |
|---|---|---|
| Sprite table | 0x69bc18 | 20-byte records, stop when record.index != position |
| Screen factory switch | 0x4b581a | `dec; cmp 0x24; ja; jmp [eax*4+0x4b5831]` -> per-screen create fn |
| Menu command dispatcher | 0x4a8ab5 | compare tree + jump tables (resolver in `wrappers/switch.py`) |
| CSD init (bg 5030, map/minimap rects) | 0x438101 | SetRect constants |
| CSD widget layout (dial, toggle, pan, 600/601) | 0x440d01 | widget Create calls |
| CSD nav/panel buttons | 0x4416dc | 0x417c6e calls (id, cx, cy, parent, sprite, states) |
| CSD button actions | 0x445291 | switch on id |
| CSD lamps / panels / markers | 0x44068d | 0x5fc999 + 0x46d548(count, ids[]) + 0x46ddce(x, y) |
| Time-dial hole coordinates | 0x4ccae5 | 6 `mov word [eax+0x36+4n], x` / `+0x38` y |
| Bases layout | 0x47c74d (rects), 0x47d031 (widgets), 0x47f1d9 (callback) | |
| Equip Ships | 0x535566 (rects, socket arrays), 0x538633 (widgets), 0x53d10b (callback) | |
| Equipment & Personnel | 0x563a7f, 0x563ecf, 0x569cb9 | |
| Research | 0x4684a8, 0x46892b, 0x46ab88 | |
| Base Stats | 0x50179b, 0x501cad / 0x501f81, 0x504abf | |
| UFOpedia | 0x530b6b | |
| Status reports | 0x48fef4, 0x4903f8, 0x491a88, 0x493147, 0x4901ec | |
| Briefing / Debriefing | 0x522398 / 0x53ee96 | |
| Game configuration | 0x52a555 (images, rects), 0x52b63f (widgets), 0x52e762 (callback) | |
| Combat-sim setup | 0x544d1a, 0x5449fc, 0x5456fd, 0x54718e | |
| Title | 0x50efd7 (rects, anims), 0x510cd0 (labels), 0x51131f (actions) | |
| Dialogs (CSD) | csd2.cpp 0x5dde41-0x5ec463 | see screens.yaml `dialogs` |
| Dialogs (other) | MiscWrappers.cpp 0x4b9b3e-0x4caca1 | |
| MsgBox frame painter | 0x518604 | style member +0xf4 selects frame kit |

Robust re-location: search `.text` for `68 <id32> 8D 4D ?? E8` (push bg id; lea ecx; call
LoadResource) for backgrounds, and for `68 <x> 68 <y>`-style push runs before `E8 rel32` whose
target is one of the Create functions above. The call targets themselves can be found by their
`D:\ss\...\<File>.cpp` assert strings.

## Tools written (scratchpad, stdlib only)

`wrappers/pe.py` (PE sections, imports, VA<->offset), `wrappers/dis.py` (x86-32 decoder for the
MSVC subset used here), `wrappers/sweep.py` (linear sweep of .text, 708k instructions),
`wrappers/args.py` (call-site argument recovery), `wrappers/switch.py` + `wrappers/handler.py`
(MSVC switch resolver by concrete simulation), `wrappers/widgets.py` (all widget creations ->
`widgets.txt`), `wrappers/rects.py` / `rects2.py` (SetRect/OffsetRect constants ->
`setrects.txt`), `wrappers/spr.py` (sprite table), `wrappers/gencat.py` (catalogue),
`wrappers/msheet.py` (labelled contact sheets).
