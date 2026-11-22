# Sprite-button states per control (generated)

Generated 2026-11-22 by `tools/uikit.py` + the widget scan (CD build). Facts only: table indices and
SpritesDLL resource ids. Rule (verified, 0x41773f / 0x417e79 / 0x417f81): a button with S states draws
sprite[index + 0] when up and sprite[index + S - 1] when pressed-and-inside or latched. **S = 1 draws nothing when
up** (the background art shows the button); the one sprite is its pressed/lit image. A disabled button
(window byte +0xf = 0) draws its disabled sprite if one was set (0x417e58), else state 0. There is no
hover image. Sound = interface sound index played on press (0x41765a; table in ui-framework.md §2.6).
Repeat = auto-repeat delay,interval in 60 Hz ticks while held (0x46b990). All scroll arrows: sprite 60/63,
fire on press, repeat 20,5.

| File | Widget | id | centre/pos | index | S | up (state 0) | down (state S-1) | disabled | sound | repeat | latched at start |
|---|---|---|---|---|---|---|---|---|---|---|---|
| CSD | SpriteButtonC | 600 | (460, 429) | 383 | 2 | spr:19200 | spr:19201 |  | 18 sndBeep29b |  |  |
| CSD | SpriteButtonC | 601 | (460, 385) | 385 | 2 | spr:19202 | spr:19203 |  | 18 sndBeep29b |  |  |
| CSD | SpriteButtonC | 504 | (516, 194) | 372 | 1 | (none: art) | spr:18000 |  | 17 sndBeep22 |  |  |
| CSD | SpriteButtonC | 502 | (516, 226) | 378 | 1 | (none: art) | spr:18006 |  | 17 sndBeep22 |  |  |
| CSD | SpriteButtonC | 500 | (609, 194) | 382 | 1 | (none: art) | spr:18010 |  | 17 sndBeep22 |  |  |
| CSD | SpriteButtonC | 503 | (609, 226) | 374 | 1 | (none: art) | spr:18002 |  | 17 sndBeep22 |  |  |
| CSD | SpriteButtonC | 515 | (563, 194) | 380 | 1 | (none: art) | spr:18008 |  | 17 sndBeep22 |  |  |
| CSD | SpriteButtonC | 516 | (563, 226) | 376 | 1 | (none: art) | spr:18004 |  | 17 sndBeep22 |  |  |
| CSD | SpriteButtonC | 700 | (605, 206) | 1033 | 1 | (none: art) | spr:29420 |  | 0 Click (default) |  |  |
| CSD | SpriteButtonC | 800 | (544, 69) | 1018 | 1 | (none: art) | spr:29400 |  | 0 Click (default) |  |  |
| CSD | SpriteButtonC | 801 | (544, 102) | 1019 | 1 | (none: art) | spr:29401 |  | 0 Click (default) |  |  |
| CSD | SpriteButtonC | 802 | (544, 135) | 1020 | 1 | (none: art) | spr:29402 |  | 0 Click (default) |  |  |
| CSD | SpriteButtonC | 803 | (603, 69) | 1021 | 1 | (none: art) | spr:29403 |  | 0 Click (default) |  |  |
| CSD | SpriteButtonC | 805 | (603, 102) | 1022 | 1 | (none: art) | spr:29404 |  | 0 Click (default) |  |  |
| CSD | SpriteButtonC | 806 | (603, 135) | 1023 | 1 | (none: art) | spr:29405 |  | 0 Click (default) |  |  |
| CSD | SpriteButtonC | 505 | (544, 57) | 1024 | 1 | (none: art) | spr:29406 |  | 0 Click (default) |  |  |
| CSD | SpriteButtonC | 506 | (544, 90) | 1025 | 1 | (none: art) | spr:29407 |  | 0 Click (default) |  |  |
| CSD | SpriteButtonC | 507 | (544, 123) | 1026 | 1 | (none: art) | spr:29408 |  | 0 Click (default) |  |  |
| CSD | SpriteButtonC | 508 | (544, 156) | 1027 | 1 | (none: art) | spr:29409 |  | 0 Click (default) |  |  |
| CSD | SpriteButtonC | 509 | (544, 189) | 1028 | 1 | (none: art) | spr:29410 |  | 0 Click (default) |  |  |
| CSD | SpriteButtonC | 513 | (603, 57) | 1029 | 1 | (none: art) | spr:29411 |  | 0 Click (default) |  |  |
| CSD | SpriteButtonC | 510 | (603, 90) | 1030 | 1 | (none: art) | spr:29412 |  | 0 Click (default) |  |  |
| CSD | SpriteButtonC | 511 | (603, 123) | 1031 | 1 | (none: art) | spr:29413 |  | 0 Click (default) |  |  |
| CSD | SpriteButtonC | 512 | (603, 156) | 1032 | 1 | (none: art) | spr:29414 |  | 0 Click (default) |  |  |
| ResearchScreen | SpriteButtonC | 101 | (53, 336) | 1077 | 2 | spr:22026 | spr:22027 |  | 0 Click (default) |  |  |
| ResearchScreen | SpriteButtonC | 100 | (253, 336) | 1075 | 2 | spr:22024 | spr:22025 |  | 0 Click (default) |  |  |
| Animation | SpriteButtonC | 402 | computed | 60 | 2 | spr:27500 | spr:27501 | spr:27502 | 11 Beep8 | 20,5 ticks |  |
| Animation | SpriteButtonC | 401 | computed | 63 | 2 | spr:27503 | spr:27504 | spr:27505 | 11 Beep8 | 20,5 ticks |  |
| Animation | SpriteButtonC | 112 | computed | 60 | 2 | spr:27500 | spr:27501 | spr:27502 | 11 Beep8 | 20,5 ticks |  |
| Animation | SpriteButtonC | 113 | computed | 63 | 2 | spr:27503 | spr:27504 | spr:27505 | 11 Beep8 | 20,5 ticks |  |
| Animation | SpriteButtonC | 112 | computed | 60 | 2 | spr:27500 | spr:27501 | spr:27502 | 11 Beep8 | 20,5 ticks |  |
| Animation | SpriteButtonC | 113 | computed | 63 | 2 | spr:27503 | spr:27504 | spr:27505 | 11 Beep8 | 20,5 ticks |  |
| Bases | SpriteButtonC | 400 | (484, 247) | 1381 | 2 | spr:28144 | spr:28145 |  | 0 Click (default) |  |  |
| Bases | SpriteButtonC | 401 | (580, 247) | 1379 | 2 | spr:28142 | spr:28143 |  | 0 Click (default) |  |  |
| Bases | SpriteButtonC | 402 | (387, 247) | 1377 | 2 | spr:28140 | spr:28141 |  | 0 Click (default) |  |  |
| Wrappers | SpriteButtonC | 102 | (620, 282) | 69 | 2 | spr:27509 | spr:27510 |  | 0 Click (default) |  |  |
| Wrappers | SpriteButtonC | 104 | (617, 365) | 66 | 2 | spr:27506 | spr:27507 |  | 0 Click (default) |  |  |
| StatusReports | SpriteButtonC | 2000 | computed | 60 | 2 | spr:27500 | spr:27501 | spr:27502 | 11 Beep8 | 20,5 ticks |  |
| StatusReports | SpriteButtonC | 2001 | computed | 63 | 2 | spr:27503 | spr:27504 | spr:27505 | 11 Beep8 | 20,5 ticks |  |
| CStartScreen | SpriteButtonC | 101 | (566, 338) | 322 | 2 | spr:2520 | spr:2521 |  | 0 Click (default) |  |  |
| CStartScreen | SpriteButtonC | 102 | (446, 338) | 326 | 2 | spr:2524 | spr:2525 |  | 0 Click (default) |  |  |
| CStartScreen | SpriteButtonC | 103 | (506, 338) | 324 | 2 | spr:2522 | spr:2523 |  | 0 Click (default) |  |  |
| MiscWrappers | SpriteButtonC | 201 | computed | 60 | 2 | spr:27500 | spr:27501 | spr:27502 | 11 Beep8 | 20,5 ticks |  |
| MiscWrappers | SpriteButtonC | 202 | computed | 63 | 2 | spr:27503 | spr:27504 | spr:27505 | 11 Beep8 | 20,5 ticks |  |
| MiscWrappers | SpriteButtonC | 201 | computed | 60 | 2 | spr:27500 | spr:27501 | spr:27502 | 11 Beep8 | 20,5 ticks |  |
| MiscWrappers | SpriteButtonC | 202 | computed | 63 | 2 | spr:27503 | spr:27504 | spr:27505 | 11 Beep8 | 20,5 ticks |  |
| MiscWrappers | SpriteButtonC | 201 | computed | 60 | 2 | spr:27500 | spr:27501 | spr:27502 | 11 Beep8 | 20,5 ticks |  |
| MiscWrappers | SpriteButtonC | 202 | computed | 63 | 2 | spr:27503 | spr:27504 | spr:27505 | 11 Beep8 | 20,5 ticks |  |
| MiscWrappers | SpriteButtonC | 201 | computed | 60 | 2 | spr:27500 | spr:27501 |  | 0 Click (default) | 20,5 ticks |  |
| MiscWrappers | SpriteButtonC | 202 | computed | 63 | 2 | spr:27503 | spr:27504 |  | 0 Click (default) | 20,5 ticks |  |
| MiscWrappers | SpriteButtonC | 201 | computed | 60 | 2 | spr:27500 | spr:27501 | spr:27502 | 11 Beep8 | 20,5 ticks |  |
| MiscWrappers | SpriteButtonC | 202 | computed | 63 | 2 | spr:27503 | spr:27504 | spr:27505 | 11 Beep8 | 20,5 ticks |  |
| MiscWrappers | SpriteButtonC | 201 | computed | 60 | 2 | spr:27500 | spr:27501 | spr:27502 | 11 Beep8 | 20,5 ticks |  |
| MiscWrappers | SpriteButtonC | 202 | computed | 63 | 2 | spr:27503 | spr:27504 | spr:27505 | 11 Beep8 | 20,5 ticks |  |
| MiscWrappers | SpriteButtonC | 201 | computed | 60 | 2 | spr:27500 | spr:27501 | spr:27502 | 11 Beep8 | 20,5 ticks |  |
| MiscWrappers | SpriteButtonC | 202 | computed | 63 | 2 | spr:27503 | spr:27504 | spr:27505 | 11 Beep8 | 20,5 ticks |  |
| MiscWrappers | SpriteButtonC | 201 | computed | 60 | 2 | spr:27500 | spr:27501 | spr:27502 | 11 Beep8 | 20,5 ticks |  |
| MiscWrappers | SpriteButtonC | 202 | computed | 63 | 2 | spr:27503 | spr:27504 | spr:27505 | 11 Beep8 | 20,5 ticks |  |
| MiscWrappers | SpriteButtonC | 201 | computed | 60 | 2 | spr:27500 | spr:27501 | spr:27502 | 11 Beep8 | 20,5 ticks |  |
| MiscWrappers | SpriteButtonC | 202 | computed | 63 | 2 | spr:27503 | spr:27504 | spr:27505 | 11 Beep8 | 20,5 ticks |  |
| BaseStats | SpriteButtonC | 401 | computed | 60 | 2 | spr:27500 | spr:27501 | spr:27502 | 11 Beep8 | 20,5 ticks |  |
| BaseStats | SpriteButtonC | 400 | computed | 63 | 2 | spr:27503 | spr:27504 | spr:27505 | 11 Beep8 | 20,5 ticks |  |
| MPSTest | SpriteButtonC | 100 | (170, 352) | 1504 | 1 | (none: art) | spr:29004 |  | 0 Click (default) |  |  |
| MPSTest | SpriteButtonC | 101 | (108, 351) | 1502 | 2 | spr:29002 | spr:29003 |  | 0 Click (default) |  |  |
| MPSTest | SpriteButtonC | 102 | (301, 351) | 1500 | 2 | spr:29000 | spr:29001 |  | 0 Click (default) |  |  |
| MPSTest | SpriteButtonC | 103 | (240, 352) | 1510 | 1 | (none: art) | spr:29010 |  | 0 Click (default) |  |  |
| MPSTest | SpriteButtonC | 104 | (582, 82) | 1506 | 1 | (none: art) | spr:29006 |  | 0 Click (default) |  |  |
| MPSTest | SpriteButtonC | 106 | (582, 184) | 1509 | 1 | (none: art) | spr:29009 |  | 0 Click (default) |  |  |
| MPSTest | SpriteButtonC | 601 | (625, 269) | 60 | 2 | spr:27500 | spr:27501 | spr:27502 | 11 Beep8 | 20,5 ticks |  |
| MPSTest | SpriteButtonC | 602 | (625, 355) | 63 | 2 | spr:27503 | spr:27504 | spr:27505 | 11 Beep8 | 20,5 ticks |  |
| MPSTest | SpriteButtonC | 611 | (569, 426) | 60 | 2 | spr:27500 | spr:27501 | spr:27502 | 11 Beep8 | 20,5 ticks |  |
| MPSTest | SpriteButtonC | 612 | (569, 452) | 63 | 2 | spr:27503 | spr:27504 | spr:27505 | 11 Beep8 | 20,5 ticks |  |
| DropDown | SpriteButtonC | 101 | computed | 60 | 2 | spr:27500 | spr:27501 | spr:27502 | 11 Beep8 | 20,5 ticks |  |
| DropDown | SpriteButtonC | 102 | computed | 63 | 2 | spr:27503 | spr:27504 | spr:27505 | 11 Beep8 | 20,5 ticks |  |
| ProdScreen | SpriteButtonC | 400 | computed | 60 | 2 | spr:27500 | spr:27501 | spr:27502 | 11 Beep8 | 20,5 ticks |  |
| ProdScreen | SpriteButtonC | 401 | computed | 63 | 2 | spr:27503 | spr:27504 | spr:27505 | 11 Beep8 | 20,5 ticks |  |
| Databank | SpriteButtonC | 102 | (616, 64) | 396 | 2 | spr:5103 | spr:5104 | spr:5104 | 0 Click (default) |  |  |
| Databank | SpriteButtonC | 103 | (336, 64) | 398 | 2 | spr:5105 | spr:5106 | spr:5106 | 0 Click (default) |  |  |
| Databank | SpriteButtonC | 108 | (614, 123) | 400 | 2 | spr:5108 | spr:5109 | spr:5109 | 0 Click (default) |  |  |
| Databank | SpriteButtonC | 109 | (614, 160) | 402 | 2 | spr:5110 | spr:5111 | spr:5111 | 0 Click (default) |  |  |
| EquipShip | SpriteButtonC | 401 | (572, 409) | 1259 | 1 | (none: art) | spr:22956 |  | 0 Click (default) |  |  |
| EquipShip | SpriteButtonC | 402 | (572, 451) | 1260 | 1 | (none: art) | spr:22958 |  | 0 Click (default) |  |  |
| EquipShip | SpriteButtonC | 300 | (200, 356) | 1159 | 2 | spr:22506 | spr:22507 |  | 19 sndBeep3025 |  |  |
| EquipShip | SpriteButtonC | 302 | (479, 57) | 1153 | 2 | spr:22500 | spr:22501 |  | 19 sndBeep3025 |  | yes |
| EquipShip | SpriteButtonC | 304 | (533, 57) | 1155 | 2 | spr:22502 | spr:22503 |  | 19 sndBeep3025 |  |  |
| EquipShip | SpriteButtonC | 306 | (587, 57) | 1157 | 2 | spr:22504 | spr:22505 |  | 19 sndBeep3025 |  |  |
| EquipShip | SpriteButtonC | 302 | (489, 57) | 1153 | 2 | spr:22500 | spr:22501 |  | 0 Click (default) |  | yes |
| EquipShip | SpriteButtonC | 304 | (577, 57) | 1155 | 2 | spr:22502 | spr:22503 |  | 0 Click (default) |  |  |
| EquipShip | SpriteButtonC | 350 | (404, 222) | 1238 | 1 | (none: art) | spr:22903 |  | 0 Click (default) |  |  |
| EquipShip | SpriteButtonC | 501 | (482, 347) | 1263 | 2 | spr:22980 | spr:22981 |  | 0 Click (default) |  |  |
| EquipShip | SpriteButtonC | 502 | (593, 347) | 1265 | 2 | spr:22982 | spr:22983 |  | 0 Click (default) |  |  |
| EquipShip | SpriteButtonC | 503 | (519, 347) | 1267 | 2 | spr:22984 | spr:22985 |  | 0 Click (default) |  |  |
| EquipShip | SpriteButtonC | 505 | (556, 347) | 1269 | 2 | spr:22986 | spr:22987 |  | 0 Click (default) |  |  |
| EquipShip | SpriteButtonC | 500 | (445, 347) | 66 | 2 | spr:27506 | spr:27507 | spr:27508 | 11 Beep8 | 20,5 ticks |  |
| EquipShip | SpriteButtonC | 504 | (630, 347) | 69 | 2 | spr:27509 | spr:27510 | spr:27511 | 11 Beep8 | 20,5 ticks |  |
| TrainingSetup | SpriteButtonC | 70 | (445, 421) | 1289 | 1 | (none: art) | spr:24012 |  | 19 sndBeep3025 |  |  |
| TrainingSetup | SpriteButtonC | 113 | (47, 47) | 1277 | 2 | spr:24004 | spr:24005 |  | 17 sndBeep22 |  |  |
| TrainingSetup | SpriteButtonC | 111 | (94, 47) | 1275 | 2 | spr:24002 | spr:24003 |  | 17 sndBeep22 |  |  |
| TrainingSetup | SpriteButtonC | 140 | (141, 47) | 1283 | 2 | spr:24015 | spr:24016 |  | 17 sndBeep22 |  |  |
| TrainingSetup | SpriteButtonC | 112 | (188, 47) | 1279 | 2 | spr:24006 | spr:24007 |  | 17 sndBeep22 |  |  |
| TrainingSetup | SpriteButtonC | 141 | (235, 47) | 1281 | 2 | spr:24013 | spr:24014 |  | 17 sndBeep22 |  |  |
| TeamSel | SpriteButtonC | 900 | (219, 144) | 69 | 2 | spr:27509 | spr:27510 |  | 0 Click (default) |  |  |
| TeamSel | SpriteButtonC | 901 | (219, 227) | 66 | 2 | spr:27506 | spr:27507 |  | 0 Click (default) |  |  |
| TeamSel | SpriteButtonC | 902 | (35, 349) | 1517 | 1 | (none: art) | spr:29030 |  | 0 Click (default) |  |  |
| TeamSel | SpriteButtonC | 903 | (92, 349) | 1515 | 1 | (none: art) | spr:29028 |  | 0 Click (default) |  |  |
| TeamSel | SpriteButtonC | 905 | (160, 353) | 1520 | 1 | (none: art) | spr:29033 |  | 0 Click (default) |  | yes |
| TeamSel | SpriteButtonC | 906 | (566, 431) | 1519 | 1 | (none: art) | spr:29032 |  | 0 Click (default) |  |  |
| csd2 | SpriteButtonC | 1000 | computed | 60 | 2 | spr:27500 | spr:27501 | spr:27502 | 11 Beep8 | 20,5 ticks |  |
| csd2 | SpriteButtonC | 1001 | computed | 63 | 2 | spr:27503 | spr:27504 | spr:27505 | 11 Beep8 | 20,5 ticks |  |
| csd2 | SpriteButtonC | 1000 | computed | 60 | 2 | spr:27500 | spr:27501 | spr:27502 | 11 Beep8 | 20,5 ticks |  |
| csd2 | SpriteButtonC | 1001 | computed | 63 | 2 | spr:27503 | spr:27504 | spr:27505 | 11 Beep8 | 20,5 ticks |  |
| ResearchScreen | SpriteButtonTL | 102 | (130, 320) | 1069 | 2 | spr:22020 | spr:22021 |  | 15 Beep13 |  |  |
| ResearchScreen | SpriteButtonTL | 204 | computed | 60 | 2 | spr:27500 | spr:27501 | spr:27502 | 11 Beep8 | 20,5 ticks |  |
| ResearchScreen | SpriteButtonTL | 205 | computed | 63 | 2 | spr:27503 | spr:27504 | spr:27505 | 11 Beep8 | 20,5 ticks |  |
| Bases | SpriteButtonTL | 200 | (504, 287) | 1341 | 2 | spr:28100 | spr:28101 |  | 17 sndBeep22 |  |  |
| Bases | SpriteButtonTL | 202 | (504, 324) | 1343 | 2 | spr:28102 | spr:28103 |  | 17 sndBeep22 |  |  |
| Bases | SpriteButtonTL | 204 | (504, 361) | 1345 | 2 | spr:28104 | spr:28105 |  | 17 sndBeep22 |  |  |
| Bases | SpriteButtonTL | 206 | (504, 398) | 1347 | 2 | spr:28106 | spr:28107 |  | 17 sndBeep22 |  |  |
| Bases | SpriteButtonTL | 208 | (504, 435) | 1349 | 2 | spr:28108 | spr:28109 |  | 17 sndBeep22 |  |  |
| MainConfig | SpriteButtonTL | 113 | (558, 173) | 404 | 2 | spr:6911 | spr:6912 | spr:6913 | 11 Beep8 | 20,5 ticks |  |
| MainConfig | SpriteButtonTL | 114 | (558, 215) | 407 | 2 | spr:6914 | spr:6915 | spr:6913 | 11 Beep8 | 20,5 ticks |  |
| EquipShip | SpriteButtonTL | 400 | (597, 391) | 1254 | 2 | spr:22950 | spr:22951 |  | 0 Click (default) |  |  |
| EquipShip | SpriteButtonTL | 204 | (518, 413) | 60 | 2 | spr:27500 | spr:27501 | spr:27502 | 11 Beep8 | 20,5 ticks |  |
| EquipShip | SpriteButtonTL | 205 | (518, 434) | 63 | 2 | spr:27503 | spr:27504 | spr:27505 | 11 Beep8 | 20,5 ticks |  |
| Debriefing | SpriteButtonTL | 102 | (548, 42) | 410 | 2 | spr:6917 | spr:6918 |  | 0 Click (default) |  |  |
| Debriefing | SpriteButtonTL | 103 | (548, 67) | 413 | 2 | spr:6920 | spr:6921 | spr:6922 | 11 Beep8 |  |  |
| EquipPerson | SpriteButtonTL | 106 | (587, 326) | 1061 | 2 | spr:20022 | spr:20023 |  | 0 Click (default) |  |  |
| EquipPerson | SpriteButtonTL | 300 | (81, 226) | 1053 | 2 | spr:20014 | spr:20015 |  | 17 sndBeep22 |  | yes |
| EquipPerson | SpriteButtonTL | 301 | (132, 226) | 1055 | 2 | spr:20016 | spr:20017 |  | 17 sndBeep22 |  |  |
| EquipPerson | SpriteButtonTL | 302 | (183, 226) | 1057 | 2 | spr:20018 | spr:20019 |  | 17 sndBeep22 |  |  |
| EquipPerson | SpriteButtonTL | 303 | (234, 226) | 1059 | 2 | spr:20020 | spr:20021 |  | 17 sndBeep22 |  |  |
| EquipPerson | SpriteButtonTL | 400 | (514, 142) | 1049 | 2 | spr:20010 | spr:20011 |  | 19 sndBeep3025 |  | yes |
| EquipPerson | SpriteButtonTL | 401 | (514, 204) | 1051 | 2 | spr:20012 | spr:20013 |  | 19 sndBeep3025 |  |  |
| EquipPerson | SpriteButtonTL | 700 | (562, 290) | 404 | 2 | spr:6911 | spr:6912 | spr:6913 | 11 Beep8 | 20,5 ticks |  |
| EquipPerson | SpriteButtonTL | 701 | (562, 314) | 407 | 2 | spr:6914 | spr:6915 | spr:6913 | 11 Beep8 | 20,5 ticks |  |
| EquipPerson | SpriteButtonTL | 180 | (587, 279) | 1047 | 2 | spr:20002 | spr:20003 |  | 0 Click (default) |  |  |
| EquipPerson | SpriteButtonTL | 181 | (587, 373) | 1045 | 2 | spr:20000 | spr:20001 |  | 0 Click (default) |  |  |
| EquipPerson | SpriteButtonTL | 700 | (562, 290) | 404 | 2 | spr:6911 | spr:6912 | spr:6913 | 11 Beep8 | 20,5 ticks |  |
| EquipPerson | SpriteButtonTL | 701 | (562, 314) | 407 | 2 | spr:6914 | spr:6915 | spr:6913 | 11 Beep8 | 20,5 ticks |  |
| EquipPerson | SpriteButtonTL | 700 | (195, 290) | 404 | 2 | spr:6911 | spr:6912 | spr:6913 | 11 Beep8 | 20,5 ticks |  |
| EquipPerson | SpriteButtonTL | 701 | (195, 314) | 407 | 2 | spr:6914 | spr:6915 | spr:6913 | 11 Beep8 | 20,5 ticks |  |
