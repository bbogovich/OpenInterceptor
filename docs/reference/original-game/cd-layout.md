# CD layout

What is on the CD and what the engine reads. Paths are relative to the `Game/` folder of a CD
copy. Sizes are approximate.

| Path | Holds | Read by the engine |
|---|---|---|
| `Resources/xobjects1.dll`, `xobjects2.dll` | All 3D models (`XMSH`), their textures (`XTXR`) and effect sprites (`XSPT`), as Win32 resources | Yes (Stage 1) |
| `Resources/CockpitA.dll`, `CockpitB.dll`, `CockpitC.dll` | Cockpit art per hull (`TARG` = TGA): the 640×480 frame and instrument parts | Yes (Stage 1) |
| `Resources/CockpitMisc.dll` | Shared cockpit/HUD parts (`TARG`) | Yes (Stage 1–2) |
| `Resources/SpritesDLL.dll` | ~1,700 small TGA sprites: icons, HUD elements | Stage 2–5 |
| `Resources/WrappersDLL.dll` | Full-screen menu and strategy screens (`TARG`) | Stage 5 |
| `ProgramFiles/Interceptor.exe` | The original program. Read only for its string tables (object names) | Strings only |
| `ProgramFiles/xcgen.dll` | The original 3D engine ("XCGEN") | Never run |
| `ProgramFiles/GameSettings.ini` | Balance data: weapons, equipment, hulls, tech tree, modules, stations, scores | Yes (Stage 1) |
| `ProgramFiles/Think.ini` | Alien strategic AI | Stage 4 |
| `ProgramFiles/soundfx.ini` | Sound-effect table | Stage 2 |
| `ProgramFiles/MPSettings.INI`, `NetSettings.ini` | Multiplayer arenas and network settings | Deferred |
| `ProgramFiles/3DObjects/*.tga` | Lens flares, reticles, sun and star sprites | Yes (Stage 1) |
| `ProgramFiles/Animations/mfdanim.mps` | MFD talking heads: an MPSRC pack of 113 Smacker clips ([MPSRC packs](../../specification/original-formats/mpsrc.md)) | Read since main `319a31b`; shown on the comms panel (app 0.13.1) once mapped to speakers |
| `ProgramFiles/SoundFX/{SpaceFlight,Chatter,Gadgets,Wrappers}/` | Sound effects and radio chatter, MS-ADPCM WAV | Stage 1–3 |
| `ProgramFiles/Loops/*.wav` | Music loops (cruise, battle, …) as PCM WAV | Stage 5 |
| `ProgramFiles/Text/*.txt` | UI strings, encyclopedia, names, news, credits | Stage 3–5 |
| `Animations/*.smk` | Cut-scenes, Smacker video | Stage 5 |
| `../CD Audio/*.aiff` | Red Book music tracks, extracted by the player as AIFF-C | Stage 5 |
| `SETUP.EXE`, `*.cab`, `DirectX/`, `IE4_0Setup/`, `EReg/` | Installer, DirectX 5, Internet Explorer 4, registration | Never |

The CD's `ProgramFiles/` folder is already an installed layout, so the engine reads the CD copy
directly; no installer step is needed.
