# Deviations

Where OpenInterceptor knowingly differs from the original game or from the shared design, and
why. A deviation on the record is a decision; an undocumented one looks like a bug.

| ID | Deviation | Why | Stage |
|---|---|---|---|
| DEV-001 | **Match speed** has a default key, `M`. | The action exists in the original's bindable list, but its release default key is not documented anywhere found. | 1 |
| DEV-002 | **Navigation points** can be placed in scenarios, with a nav-select key and HUD marker. | The original has no in-mission nav points (travel happens on the strategy map). Mods need them for custom missions; the Interceptor pack does not use them. | 1 |
| DEV-003 | **Gamepad** support with an XInput-style layout. | The original supported DirectInput joysticks and rudders only. A modern pad layout is added; the joystick-plus-rudder layout and Alt+J swap stay available. | 1 |
| DEV-004 | **Mouse controls are rebindable.** | The original fixed them. | 1 |
| DEV-005 | **Rendering is hardware 3D with per-vertex lighting** instead of the 16 shade palettes. | The shade palettes exist for the 1998 software renderer. Palette 0 (full brightness) is used as the texture's colours; light is applied by the GPU. | 1 |
| DEV-006 | **Wide-screen**: the 640×480 cockpit art is scaled to the window height and centred; the 3D view fills the whole window. | Modern displays are not 4:3. | 1 |
| DEV-007 | **Multiplayer** is deferred and may never be implemented. | Not needed for the single-player game or for modding. | 7 |
