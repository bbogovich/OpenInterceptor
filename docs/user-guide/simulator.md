# Simulator

The simulator flies any mission of the loaded campaign the way you set it up: in another place,
with hazards added, against a smaller or larger enemy force. It is for practice and for trying
things out; nothing you do in it counts towards a campaign. (App 0.6.3; requirement
[R-MSN-010](../specification/requirements/missions.md#r-msn-010-simulator-missions).)

## Opening it

In the launcher, choose **Simulator (set up a mission and fly it)** — the entry after
*Controls*. The difficulty level is the launcher's: press **D** there first to change it.

## Setting up

| Row | Choices | What it does |
|---|---|---|
| **Mission** | every mission of the campaign | The battle to fly. Its briefing is shown under the rows. |
| **Ship** | the ships you can fly | Your craft. A mission that fixes your craft shows "(the mission's own)". |
| **Environment** | the mission's own (or open space), then every environment of the campaign | Where the battle happens: open space, orbit, a pocket, a planet surface … |
| **Asteroids** | off / on | Adds an asteroid field ahead of you. |
| **Minefield** | off / on | Adds a minefield ahead of you. |
| **Black hole** | off / on | Adds a black hole some 9 km ahead: its pull grows as you near it, and its core destroys what crosses it. |
| **Ion storm** | off / on | Cuts your radar range to 40 % and slows missile locks. |
| **Enemy force** | ×0.5, ×1, ×1.5, ×2 | Scales every group of enemies and every enemy reinforcement (never below one craft). Your wingmen and what you defend are unchanged. |
| **Fly** | — | Starts the battle. |

Keys: **Up/Down** choose a row, **Left/Right** change it, **Enter** flies, **Esc** returns to the
launcher.

- A campaign that has no asteroids or no mines shows "(none in this campaign)" on that row.
- Some ships cannot fly in an atmosphere. If you choose one with a planet-surface environment,
  Fly shows "The … cannot enter an atmosphere: choose another ship or environment." and nothing
  starts.
- With the mission's own environment, no hazards and ×1, the battle is exactly the mission as
  the launcher would start it.

## After the battle

The debrief appears as in any mission. **Enter** (pad **A**) flies the **same setup** again —
the same mission, ship, environment, hazards and force; **Esc** or **L** returns to the launcher.
**R** in the pause menu also restarts the same setup.

## Ideas

- Fly `defend_station` on `moon_surface` to feel how the same fight changes in thin air and low
  gravity.
- Add a black hole to `intercept_scouts` and use its pull to catch the scouts.
- Try a hard mission at ×0.5 to learn it, then at ×2 once you know it.
