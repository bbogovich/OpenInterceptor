# Strategy-screen navigation (Interceptor.exe)

All edges are **verified** from the exe unless marked *(derived)*. Screen numbers are the exe's
own ids (global `0x682368`); `0x4b5c4e(n)` pushes the current screen on a history stack
(`0x6faa80`, max 29) and switches, `0x4b5cf2(n)` switches from the menu bar, `0x4b5bfe(n)`
switches without pushing, `0x4b5da5()` pops ("back"). Screen 17 (CSD) clears the stack.

## Screen ids -> create function -> background

| Id | Screen | Create fn | Class vtable | Bg |
|---|---|---|---|---|
| 1 | intro movie | 0x4b4568 | 0x661a38 | - |
| 2 | title / main menu | 0x50ef10 | 0x6617f0 | 7600 |
| 3 | game configuration | 0x52a0a0 | 0x661b60 | 7020 |
| 4 | combat simulator setup | 0x543e50 | 0x661d80 | 7010 |
| 5, 15, 28 | real-time flight (28 shows 6542, then 8100/8116 splash) | 0x550347 / 0x45bbc7 / 0x45cf65 | - | 6542 |
| 6 | Bases | 0x482bec | 0x6602d8 | 6105 |
| 7, 8 | Equip Ships | 0x53d93d | 0x661c18 | 6100 |
| 9 | Research | 0x4681e0 | 0x65fef8 | 5101 |
| 10 | credits | 0x4b4f32 | 0x6609a8 | 8132 |
| 12 | multiplayer home | 0x41b3b0 | 0x65f698 | 5000 |
| 13 | (no class) | 0x4e99f0 | - | - |
| 14 | multiplayer game list | 0x6008b0 | 0x663558 | - |
| 17 | CSD | 0x437730 | 0x65fbc0 | 5030 |
| 18 | Equipment & Personnel | 0x563996 | 0x661ef8 | 6110 |
| 19 | (flight-side) | 0x559670 | - | - |
| 21 | MiscWrappers screen | 0x4ba19a | 0x660ab8 | 5090 (not in WrappersDLL) |
| 22 | Base Stats | 0x5014d0 | 0x6615a8 | 6330 |
| 23 | UFOpedia | 0x53025b (plays alientrans1.smk) | 0x661bb8 | 5107 |
| 24 | (unknown) | 0x417520 | - | - |
| 25 | Briefing | 0x522030 | 0x661af8 | 6800 |
| 26 | Debriefing | 0x53eba0 | 0x661c98 | 6900 |
| 27 | campaign start | 0x4b5fa0 | 0x660a08 | 7500+n |
| 29 | multiplayer players | 0x50c0f5 | 0x661708 | 7800 |
| 31 | multiplayer team select | 0x5aa900 | 0x662990 | 7900 |
| 32-37 | status reports | 0x499000, 0x4990e1, 0x4991c2, 0x4992a5, 0x499469, 0x49954a | 0x660520 family | 7700 |

## Graph

```
Title(2)
  SINGLE  -> difficulty dialog -> CSD(17)
  LOAD    -> load dialog (0x4665f9)
  MULTI   -> 29
  SIM     -> 4 --(70 / 208 start)--> Briefing(25)
  SETUP   -> 3
  INTRO   -> 1
  CREDITS -> 0x4b5bfe (credits)
  EXIT    -> quit confirm

CSD(17)
  504 -> Bases(6)            502 -> Research(9)
  515 -> Equip Ships(8)      516 -> Equipment & Personnel(18)
  500 -> report panel: 800->32 Income, 801->33 Finance, 802->34 Alien activity,
                       803->35 X-COM activity, 805->36, 806->37, 700 close
  503 -> UFOpedia panel: 505..513 -> UFOpedia(23) with category 0..8, 700 close
  600 -> Launch Mission dialog -> Select Destination (LAUNCH) -> ... -> Briefing(25)  (derived chain)
  601 -> choose-probe pick list

Bases(6)
  204 -> 18     206 -> 7     208 -> 22     210/212 -> 17     500 -> back
  200 new base, 202/400/401/402 module placement, 700 dismantle, 701 rename
Equipment & Personnel(18): 106 -> Production dialog; 103 cancel order; back via menu/close
Equip Ships(7/8): 100 -> back
Research(9): 104 -> back
Base Stats(22): 100 -> back
Status reports(32-37): 100 -> back
Briefing(25): 208/209 launch -> flight (28); 210 abort (yes/no) -> back
Debriefing(26): 208-210 -> back
Alien Transmission dialog: 200 (Allocate New Research) -> Research(9)
Research-complete dialog: Allocate research / View UFOpedia
```

The menu bar is a global widget at `0x6f3638`, built by `0x4a7f26` from the language string
`S_MAINMENU` (or `S_MAINMENUSIMPLE` before a game exists) via the parser `0x4a5ed3`; its callback
`0x4a7fb6` is registered with `0x44c660`. Each screen's own OnCommand (vtable slot 8) handles a
few ids itself (210 = back to CSD on most screens) and passes the rest to the base `0x48c717`.

## Menu-bar commands (global dispatcher 0x4a7fb6)

Ids come from `S_MAINMENU` in the language file (`Label[id]`).

| Cmd | Label | Action |
|---|---|---|
| 110 | New Game | confirm, then 0x5709ab |
| 120 / 130 | Load / Save Game | 0x4665f9 / 0x46684d |
| 140 | Game Configuration | push screen 3 |
| 150 | Pause | pause box |
| 160 | Quit | confirm quit |
| 171-174 | voice / system cursor on/off | flags |
| 210 | CSD | push screen 17 (every screen's own OnCommand also handles 210 as "back to CSD") |
| 221-226 | Income / Finance / Alien Activity / X-COM Activity / Outpost / Player Status report | screen 32-37 |
| 230 | Equip Ships | screen 7 |
| 240 | Research | screen 9 |
| 250 | Bases | screen 6 |
| 280 | (production) | 0x516180 |
| 281 | Equipment and Personnel | screen 18 |
| 282 | (training) | screen 4 |
| 284 | (base stats) | screen 22 |
| 801-809 | UFOpedia categories | 0x5303d6(n-801), screen 23 |
| 401 | Help Wizards | 0x4a8a5d (help/tour, *derived*) |
| 602-604, 610, 620, 631-639 | debug / sound test | various |
| 701-731 | CHEAT menu (711-716 side missions) | various |
| 901-903, 910-980 | flight VIEW menu | ignored on strategy screens |

## Time controls on the CSD

* **Dial** (widget id 100, `0x4cca65`, centre (562,304), rect (507,250)-(617,358)): six holes at
  (523,322) (536,336) (554,344) (574,344) (592,335) (605,322); picking hole *n* calls
  `GameClock(0x6bb700)::SetSpeed(n)` = `0x411f74`, n = 0..5 (the setter rejects n >= 6).
* **Keys** '1'..'6' (OnCommand ids 49..54) call the same `0x411f74(n-49)` and move the dial
  (`0x4cd0fa`).
* Dialog buttons "OK (5 SECS)" and "Return to base (5 secs)" also call `0x411f74`.
* `0x4120b7` / `0x41207f` pause and resume the clock around modal dialogs (launch mission,
  launch probe).
* Speed-to-rate values live in `0x411f74` (uses `GetTickCount`); the per-speed multipliers were
  not extracted.
