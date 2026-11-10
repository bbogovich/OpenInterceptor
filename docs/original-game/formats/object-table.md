# Object and ship table

The original program names the mesh for each object class in string tables inside
`Interceptor.exe` (near the `SceneManager.cpp` assertions and in `EquipShip.cpp`). The engine
does not read the executable at run time: the Interceptor pack's rules carry this mapping as
plain data. Mapping confidence is given per row.

## Ship classes

The table lists 21 meshes in this order: `xfre98-5`, `al-freig6`, `aliensh3`, `alienb4`,
`alienc7`, `shade`, `altug04`, `DDC4`, `inttypea`, `firestar6`, `avenger16`, `p-typea`, `p-typeb`,
`firestar6`, `p-fre98-5`, `cortypea`, `c-typeb`, `c-avenger16`, `cpat`, `alienb4`, `nova2`.

| Mesh | Class | Confidence |
|---|---|---|
| `inttypea` | X-1A Lightning II (`Ship_IntType1`) | derived (also in the Equip Ship table) |
| `firestar6` | S-2 Firestar (`Ship_IntType2`); also the pirate Firestar | derived |
| `avenger16` | M-3 Super Avenger (`Ship_IntType3`) | derived |
| `xfre98-5` | X-COM freighter (Star Ranger) | derived |
| `al-freig6` | Spirit (alien freighter) | derived |
| `aliensh3`, `alienb4`, `alienc7` | Wraith, Spectre, Phantasm (order to verify) | derived |
| `shade` | Shade | derived |
| `altug04` | Harbinger (alien tug) | derived |
| `DDC4` | unknown alien (possibly Doomsday-related) | unknown |
| `drone01` | Drone | derived (name) |
| `p-typea`, `p-typeb`, `p-fre98-5` | Pirate Lightning II, pirate Firestar/Avenger variant, rogue freighter | derived |
| `cortypea`, `c-typeb`, `c-avenger16` | Corporate hulls | derived |
| `cpat` | Patton-class research cruiser | derived |
| `nova2` | Nova bomb / Nova science object | derived |
| `dm1a` … `dm8c` | Multiplayer colour variants (8 colours × hulls A/B/C) of the player hulls | derived |
| `x-ship*`, `xcorp*`, `xpirat*`, `a-ship*`, `1ash*` … | Debris chunks | derived |

## Stations and large objects

| Mesh | Object | Confidence |
|---|---|---|
| `xbase7` | X-COM base hub (modules attach) | derived |
| `al-base01` | Alien base hub | derived |
| `xout6` | Corporate outpost | derived |
| `xopp` | Ore Processing Plant | derived |
| `al-opp7` | Alien OPP / mining platform | derived |
| `alorbout` | Alien outpost | derived |
| `pbase3` | Pirate base | derived |
| `arms4` | Arms dealer base | derived |
| `scicenter02`, `sciradar02` | Science station and its radar | derived |
| `nova2` | Nova science station | derived |
| `landing02` | Landing pad | derived (name) |
| `xmmod02`, `xmmodhub`, `a-mmod01`, … | Base modules | derived |
| `missmine`, `albeam3`, `a_expmin` | Missile, beam and explosive mines | derived (string order) |
| `sensprob`, `ecmprob`, `commjam4/9`, `al-senpr5`, `al_injam`, `x-hunt1` | Probes | derived |
| `roid12` … `roid100`, `chunk*` | Asteroids, rocks | derived |
| `*neb*`, `planetn`, `nova` | Nebulae, planets, star backdrop | derived |
