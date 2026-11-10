# Deviations

Where OpenInterceptor knowingly differs from the original game, or uses an engine value where
the original's is unknown, and why. A deviation on the record is a decision; an undocumented
one looks like a bug.

| ID | Deviation | Why | Stage |
|---|---|---|---|
| DEV-001 | **Default keys for actions the original left undocumented**: `M` match speed, `W` cycle missile, `Kp4/6/8/2` VCT slides, `Q`/`E` roll, `LeftControl` also fires beams, `C` mouse flight. | These actions exist in the original's bindable list but their release defaults are not documented anywhere found; Q/E and LeftControl help keyboards without a keypad. | 1 |
| DEV-002 | **Navigation points** in scenarios (`[nav.<name>]`), cycled with `N`, shown on the MFD Nav page and as an on-screen marker. | The original has no in-mission nav points (travel happens on the strategy map). Mods need them; the test arenas use them to find the alien and pirate groups. | 1 |
| DEV-003 | **Gamepad** support with an XInput-style layout. | The original supported DirectInput joysticks and rudders only. | 1 |
| DEV-004 | **Mouse controls are rebindable** (beams/missile via the bindings; steering range via `[mouse] range`) and mouse flight is a toggle (`C`). | The original fixed them; a toggle avoids the mouse fighting the menus. Steering direction follows the readme (cursor down = nose down). | 1 |
| DEV-005 | **Hardware 3D with per-vertex lighting** instead of the 16 shade palettes. | The shade palettes served the 1998 software renderer; palette 0 (full brightness) gives the texture colours. | 1 |
| DEV-006 | **Wide-screen**: a cockpit's design space (640×480 for the originals) is scaled as large as fits and centred; the 3D view fills the window. Cockpits may declare their own `Design` (e.g. 1920×1080) and may be 3D models. | Modern displays are not 4:3; modders replace cockpits with HD, widescreen or 3D ones. | 1 |
| DEV-007 | **Multiplayer** deferred, possibly never implemented. | Not needed for single player or modding. | 7 |
| DEV-008 | **Beam `Range` read as kilometres** (`Range = 4` → 4 000 m); `RangeMetres` overrides. | The original's 1–6 values have no documented unit; kilometres matches the documented sensor ranges (3–5 km). | 1 |
| DEV-009 | **Missile `TurnRate` and `LockTime` are engine values** set in the Interceptor pack. | The original stores no turn rate per missile; lock time depends on the targeting system (Stage 4 equipment). | 1 |
| DEV-010 | **Proximity fuse fires on a near miss**: inside `ProximityRadius` *and* moving away from the target. A hit inside the target's collision shape deals `ImpactDamage`. | Firing as soon as the radius was entered meant impacts never happened. | 1 |
| DEV-011 | **Hull `Shields`, `Armor`, `Energy`, `EnergyRecharge`, `Acceleration` are engine values per hull** in the Interceptor pack. | In the original they come from fitted equipment (ShieldType*, ArmorType*, EngineType*), which arrives with the equipment system. | 1 → 4 |
| DEV-012 | **Alien, freighter and capital flight values are estimates** (MaxSpeed, turn rates, acceleration). | The original file has flight values only for the three X-COM hulls; the others were coded per class. | 1 → 2 |
| DEV-013 | **Missile lock cone is 10°** (`[flight] LockAngleDegrees`). | The original's `SensorsSetting LockAngle` units are not decoded. | 1 |
| DEV-014 | **Cockpit ↔ hull mapping** A = Lightning II, B = Firestar, C = Super Avenger. | Derived (one cockpit per hull), not confirmed. | 1 |
| DEV-015 | **One-shot inputs are carried across frames** that run no simulation tick (`Controls::carry_unconsumed`). | On high-refresh displays some frames run no tick; a key press would otherwise be lost. Not an original behaviour, an engine correctness fix. | 1 |
| DEV-016 | **Esc opens a pause menu** (resume, restart, back to launcher) instead of resetting the MFD. | The testbed needs a way back to the launcher; MFD pages cycle with `<`/`>`. | 1 |
| DEV-017 | **Keyed cockpit pixels become transparent black**, not transparent cyan. | Filtering would otherwise blend cyan into the edges of the art. Visual result matches the original. | 1 |
| DEV-018 | **"Ignore my target" and "retreat" wingman orders both mean "form on me"**; "evasive" is not bound. | Distinct behaviours arrive with the Stage 2 AI. | 1 → 2 |
| DEV-019 | **Newtonian flight model by default**: throttle is thrust, ships coast at zero throttle, `Z` reverses thrust, a held **brake** (`B`) retro-thrusts to a stop, `MaxSpeed` caps speed, turning does not steer the velocity. `[flight] Model = assisted` restores the arcade model (throttle = target speed, automatic slowing, drift damped). | The original's flight model is not documented and felt arcade-like; the project owner chose physics where deceleration only comes from thrust. Keeping both as data lets each campaign choose. | 1 |
| DEV-020 | **Flight computer (speed hold)**: match speed (`M`) and every AI pilot fly a held forward speed through `hold_speed`, which thrusts and brakes within the hull's own acceleration and also cancels sideways drift. | Under newtonian physics AI pilots need a way to manage speed that obeys the same limits as the player; cancelling drift keeps AI dogfights readable. | 1 → 2 |
| DEV-021 | **Arena edge**: ships reaching `[flight] ArenaRadius` (default 60 km) are halted against it ("ARENA BOUNDARY"). | Under newtonian physics a coasting ship would otherwise drift away for ever; outside testing this should rarely be reached. | 1 |
| DEV-022 | **Per-asset overrides and `--extract`**: any original model, texture, image or sound is replaced by a pack file at its override path; the extractor writes every original asset at those paths in OBJ/PNG/WAV. | The modding objective: HD and 3D replacements without touching or redistributing the originals (compare XWVM, [Modding](../modding/index.md#comparison-with-xwvm)). | 1 |
