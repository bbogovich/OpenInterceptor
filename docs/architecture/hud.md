# Cockpit and HUD

`src/hud/`: the cockpit frame and every instrument. Plan: Stage 1, item 1.9.

## Architecture Overview

```mermaid
flowchart LR
    R["Rules: cockpit:a/b/c"] --> CF["cockpit frame<br/>TGA, colour-keyed"]
    W["World: player ship, target"] --> RA["radar scopes"]
    W --> MF["target MFD<br/>name, distance, hull, model"]
    W --> GA["gauges<br/>speed, throttle, energy, shields, hull"]
    W --> RE["reticle, lead marker, lock"]
    W --> MS["message line"]
    CF --> D["draw over 3D view"]
    RA --> D
    MF --> D
    GA --> D
    RE --> D
    MS --> D
```

The 640×480 cockpit art is scaled to the window height and centred; instrument positions are
given in 640×480 coordinates in the `cockpit:` rules, so a modder can move them. In the
full-forward and chase views a minimal HUD is drawn instead.

## Key Symbols

| Symbol | What it is |
|---|---|
| `Hud` | Loaded cockpit textures and instrument layout for the current ship. |
| `CockpitLayout` | Rectangles for radar, MFD, gauges and message line in 640×480 space. |
| `Messages` | A short queue of timed text lines (target destroyed, weapons linked…). |

## API Reference

- `Hud::new(&Rules, &Assets, cockpit_id)`; `Hud::draw(&mut self, &World, view)`.

## Debugging

`--hud-layout` draws the instrument rectangles over the cockpit to check positions.
