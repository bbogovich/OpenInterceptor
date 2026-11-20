# X-COM: Interceptor: public sources online (survey 2026-11-19)

This survey looks for published analysis of X-COM: Interceptor (1998) that could inform OpenInterceptor:
combat mechanics, stats, strategy rules and screen behaviour. **Bottom line:** no technical analysis exists
online. Nobody has published a reverse-engineering effort, a file-format write-up, an INI key reference, a
damage-formula study or an editor. What exists is one designer-written strategy guide, a few player guides
and walkthroughs, some wiki stubs, and compatibility fixes. The project's own work (`notes/re/`, the
docs site, the vault) is already the most detailed technical record of the game that we know of.

Each claim below is marked: **read** = page fetched and read on 2026-11-19; **seen** = known only from a
search result or a title; **vault** = recorded in an earlier session. All content is summarised in my own
words, with no copied text.

## 1. Sources worth using

| Source | What it gives | Use for | Status |
|---|---|---|---|
| **Sybex, *X-COM Interceptor Official Strategies & Secrets* (David Ellis, 1998)**, OCR text on archive.org, item `x-com-interceptor-official-strategies-secrets-1998-pc` (`*_djvu.txt`) | The most authoritative public source. Written with the designers. Covers Frontier generation, the start state, the income formula, the outpost evaluation, the alien Doomsday economy, alien-message timing, side missions, and win/lose conditions. Its squadron tables are partly illegible | Strategy-layer rules (Track B logic, #43, #39) and cross-checking `GameSettings.ini` meanings | vault |
| **Steam guide "X-COM Interceptor: the 20th anniversary strategy guide, 2018"** by Zandbergen, <https://steamcommunity.com/sharedfiles/filedetails/?id=1414648373> | A guide for the highest difficulties with play observations (see §3). It shows a Steam "removed" banner, so it may vanish | Combat behaviour to check against reference play | read |
| **Steam thread "How I fixed the frame rate issue (and boosted resolution)"** by EgoMaster, 2020, <https://steamcommunity.com/app/7730/discussions/0/4241773359639886515/> | A complete recipe for running the original on modern Windows: the StrategyCore "winxp-fix", then dgVoodoo2 (DirectX tab: 3D-accelerated adapter, 2 GB, resolution 3× → 1920×1440 from the fixed 640×480 mode, forced V-sync, app-controlled fullscreen; General tab: stretched with aspect kept, mouse capture). Known issues: cursor line artefacts, crashes after about 10 min for one user | **Reference play (plan §2)**: the most concrete route to running the original for measurements | read |
| **GameFAQs walkthrough by Brian Stewart, v1.00, Dec 2000** (mirror: <https://www.cyberly.org/game-cheats/pc-walkthroughs/x-com-interceptor.html>) | An informal walkthrough: start state, base and ship setup, the four side missions, the final mission, combat tips, money, friendly and enemy ships. Almost no numbers (see §3) | Qualitative behaviour; the screen and mission flow | read |
| **Demo readme** (GAMBLERCD18 `demos/xcominte/readme.doc` on discmaster.textfiles.com) and a 1998 cover-disc readme (<https://discmaster.textfiles.com/file/24033/1998-07_Disc_4.1.iso/xcom/readme.doc/readme.pdf>) | The CSD screens are listed in the demo readme; the cover-disc readme is a release readme | Screen inventory cross-check | vault / seen |
| **Official patch v1.2** (GamePressure mirror, <https://www.gamepressure.com/download/x-com-interceptor-v12-patch/zd13072>, about 800 KB, "technical issues and bugs") | The last official update | **Version check**: see §4 | seen |

## 2. Sources of little or no use

- **UFOpaedia** Interceptor section (`Info_(Interceptor)`, Research/`Technologies`, cheats/exploits).
  It's thin: an info page, a research list that dumps the in-game UFOpaedia text, Hypernews flavour text and
  cheats. Direct fetches return HTTP 403. The vault notes that web.archive.org `id_` snapshots work, but
  this session's permission settings blocked that route, so nothing new was read.
- **TV Tropes** (<https://tvtropes.org/pmwiki/pmwiki.php/VideoGame/XComInterceptor>). Fan claims: a
  Permeator puts half its damage into shields and half through to the hull; armour protects only once
  shields fail. These are unverified. They match the pack's `permeator: damage: bypass_shields` only partly
  (that bypasses fully).
- **SpaceGameJunkie written let's play**, entries 1–4 (e.g. <https://www.spacegamejunkie.com/?p=182022>).
  Narrative impressions of early play, not mechanics. Seen.
- **YouTube** "Let's Play X-COM Interceptor Experienced", about 25 parts, 2010–11 (listed on RAWG; channel not
  identified). A potential **visual reference for screen behaviour and timing** if the original can't be run.
  Not verified.
- **Wikipedia, MobyGames, GOG and Steam store pages, GameSpot, PCGamingWiki (a mirror only)**. Release
  facts only: MicroProse Chapel Hill, June 1998, Steam app 7730, GOG DRM-free, "not supported on
  Windows 8".
- **Steam forum**, 76 threads. Mostly compatibility, joystick-binding loss and the probe bug. No mechanics
  or modding discussion.
- **StrategyCore** hosts the "winxp-fix" (<http://www.strategycore.co.uk/files/x-com-interceptor-winxp-fix/>),
  but the site returns 403 to automated fetches. What the fix changes is undocumented anywhere I could read.
- **Nexus Mods and other "XCOM interceptor" hits** are about XCOM: Enemy Unknown (2012) aircraft, a
  different game.
- Not found: any GitHub or other repository, format documentation, save-game editor, trainer, or
  `GameSettings.ini` reference for this game.

## 3. Facts gathered (to check against RE or reference play, never to copy)

From the **Stewart walkthrough** (read, player-written):
- The game starts with 3 interceptors and 5 pilots.
- Armour and shields can't be repaired once destroyed and need spare parts in base stock.
- Cargo space limits salvage, and disabling a ship yields more loot than destroying it.
- Corporations pay protection money and never return once they withdraw.
- Aliens appear to know where radar coverage ends.
- Hardpoints: Lightning 2 beam + 2 missile; Super Avenger 4 + 4 + 1 Doppelganger.
- The Shade carries accumulator shields, tracking laser, VCT, ECM and a cloak. It attacks from behind, and when
  damaged it leaves a vapour trail while cloaked.
- Wraith: 2 plasma + 2 elerium torpedoes. Drone: no armour.
- The cloak draws on weapon power.
- In the final mission the black hole destroys craft that come close, and the Doomsday weapon can't be destroyed.

From the **2018 Steam guide** (read, player-written):
- Tractor beam range is about 0.25 km, and it drains energy.
- Fusion missiles can hit the launching ship.
- Hostile fleets turn 90° on final approach.
- Wraiths can be captured.
- Shields can be boosted from weapon energy.
- Accuracy and flying are the pilot stats that matter, and commanders don't improve.
- Combat runs too fast on modern PCs, which suggests the original's simulation is frame-tied. This affects
  any timed reference measurement: measure under a frame cap.

From the **Steam "Beam weapons" thread** (read, players):
- In the Steam build, players report that beam and shield energy don't recharge at any recharge setting.
  One player worked around it by editing the config file (energy cost 0, higher shield value).
- Either this is a version or compatibility bug, or the recharge works differently from how it's read.
  Treat it as a warning for reference play: it could corrupt measurements of `BeamRechargeRate`.

From the **probe-bug threads** (read):
- Alien link-disruptor probes, pirate bases and outposts can become untargetable after about a month.
- The simulator then shows a huge distance value (likely a corrupted position).
- The community fixes are the unofficial patch and the winxp-fix. Their internals are undocumented.

## 4. Version check: CD release vs. patch 1.2 (verified 2026-11-20)

The user downloaded the official 1.2 patch (`Interceptor12.exe`, 871,424 bytes, dated 16 Oct 1999, from
StrategyCore). It is kept in the git-ignored `resources/xcom-cd/intv12/`. It was **read as data only and
never run**: the InstallShield wrapper holds a Microsoft CAB at file offset 92,928, carved out and listed
with macOS `tar` (libarchive). The extracted files went to a scratch folder outside the repo.

| | CD release (`resources/xcom-cd/Game/ProgramFiles/Interceptor.exe`) | Patch 1.2 (in the CAB) |
|---|---|---|
| Size | 3,008,512 bytes | 3,008,000 bytes |
| Build stamp string | compiled 28 May 1998 18:13:27 | compiled 29 Jun 1998 17:12:05 |
| PE timestamp | 1998-05-29 | 1998-07-02 |
| Entry point | 0x650840 | 0x6509a0 |

- **The CAB holds only `Interceptor.exe` and `IGZPatch1.exe`** (16 KB, a small MSVC helper). There's **no
  `GameSettings.ini` or other data file**, so every INI stat is the same in 1.2 and on the CD. The
  readme lists only joystick-sensitivity fixes.
- **The exe is a full rebuild**: `.text` grew by 0x160 bytes, so code and data addresses shift. The meaningful
  string differences are almost all multiplayer/network logging (lobby, locator, client wait-loop messages).
  A few others: `Refusing to create ship -`, `Failed to send GameSettingsStart!`, and a changed
  `Sim %d: Shields killed on base %d` debug line. Gameplay changes can't be ruled out from strings alone.
- **The sprite table is byte-identical** (1527 × 20 bytes) but moved from VA 0x69bc18 (CD) to
  **0x69b598** (1.2), i.e. −0x680. Every VA in `notes/re/wrappers/` refers to the **CD build**.

Consequences:
- **Combat stats (Track A) aren't affected by the patch** as far as data goes. Mechanics RE should
  name which exe it reads, and preferably check findings in both builds.
- **The engine reads nothing from `Interceptor.exe` at run time.** Mesh names and screen positions are
  facts copied into the pack, so they work with either build. Keep it that way. If a future reader takes data
  from the exe (e.g. the planned sprite-table decoder, plan B1.1), it must **find the table by content
  signature, not a fixed address**, so it works with the CD build, patch 1.2 and Steam/GOG copies alike.
- Record the exe's identity (size, build stamp) at the top of each RE note.

## 5. Recommendations

1. Treat the Sybex guide as the primary written source for strategy rules. Everything else is anecdotal.
2. For reference play, use EgoMaster's dgVoodoo2 recipe on a Windows machine or VM, with the user's own
   (ideally v1.2) install. Measure at a fixed frame rate. Keep screenshots and recordings outside the repo.
3. Add a version check to plan §2: identify the user's Steam/GOG build and diff it against the CD copy.
4. Nothing online replaces exe RE for the damage, shield, recharge and lock mechanics (plan A1–A5).
