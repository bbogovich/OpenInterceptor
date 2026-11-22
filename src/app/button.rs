//! # The original's sprite buttons (RQ-UI-02, RQ-UI-07, RQ-UI-08)
//!
//! How a button of the original strategy screens looks and when it fires, as pure
//! functions (Interceptor.exe, button base vtable 0x65f420; verified):
//!
//! * **States are consecutive sprites.** A button is a base sprite and a state count S:
//!   up is `base + 0`, pressed `base + S − 1`. With **S = 1 nothing is drawn when up** —
//!   the screen's background art already shows the button; its one sprite is the pressed
//!   image. There is **no hover image** ([`look`]).
//! * **It fires on release.** Pressing (inside) arms it and shows the pressed image while
//!   the cursor stays inside; releasing — anywhere — fires it, unless the button fires on
//!   press ([`Press`]). Held buttons may repeat (`app::listbox::Repeat`).
//! * **Toggles and radio groups** are S = 2 buttons that stay *latched*: a latched button
//!   shows its pressed image; choosing one in a group unlatches the others, and the first
//!   of a group starts latched ([`latch`]).
//! * **Placement.** The widget rect is the sprite plus 1 px all round, centred on a point
//!   (SpriteButtonC 0x417c6e) or with its image's top-left at a point (SpriteButtonTL
//!   0x417d7c); state images are drawn centred on the rect ([`sprite_rect`]).

/// What a button shows this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Look {
    /// Nothing (S = 1 and up: the art shows it).
    Nothing,
    /// State image `k` (0 = up, S − 1 = pressed).
    State(u8),
    /// The disabled image.
    Disabled,
}

/// The image a button shows: disabled → its disabled image if it has one, else state 0
/// (nothing when S = 1); enabled → the pressed state when pressed with the cursor inside
/// or latched, else state 0 — and nothing at all for state 0 when S = 1 (0x41773f).
pub fn look(enabled: bool, pressed_inside: bool, latched: bool, states: u8, has_disabled: bool) -> Look {
    let states = states.max(1);
    if !enabled {
        return if has_disabled {
            Look::Disabled
        } else if states == 1 {
            Look::Nothing
        } else {
            Look::State(0)
        };
    }
    if pressed_inside || latched {
        Look::State(states - 1)
    } else if states == 1 {
        Look::Nothing
    } else {
        Look::State(0)
    }
}

/// A button's press: armed by a press inside it, fired by the release (anywhere) or —
/// for a fire-on-press button — by the press itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Press {
    /// Pressed and not yet released.
    pub held: bool,
}

impl Press {
    /// One frame: `inside` the cursor is over the button, `went_down` / `went_up` the
    /// mouse button's edges this frame. Returns whether the button fires.
    pub fn update(&mut self, inside: bool, went_down: bool, went_up: bool, fire_on_press: bool) -> bool {
        if went_down && inside {
            self.held = true;
            if fire_on_press {
                return true;
            }
        }
        if went_up && self.held {
            self.held = false;
            return !fire_on_press;
        }
        false
    }
}

pub use crate::content::screens::Place;

/// The widget rect of a `w` × `h` sprite button: the image plus 1 px on every side.
pub fn sprite_rect(place: Place, w: f32, h: f32) -> [f32; 4] {
    match place {
        // The image is drawn at centre − size/2 (integer halves, as the exe).
        Place::Centre([x, y]) => [x - (w / 2.0).floor() - 1.0, y - (h / 2.0).floor() - 1.0, w + 2.0, h + 2.0],
        Place::TopLeft([x, y]) => [x - 1.0, y - 1.0, w + 2.0, h + 2.0],
    }
}

/// The member of a radio group that is latched: the one chosen, else the one marked
/// `selected`, else the first (the exe latches the first at creation).
pub fn latch<'a>(chosen: Option<&'a str>, members: &[(&'a str, bool)]) -> Option<&'a str> {
    chosen
        .filter(|c| members.iter().any(|(m, _)| m == c))
        .or_else(|| members.iter().find(|(_, sel)| *sel).map(|(m, _)| *m))
        .or_else(|| members.first().map(|(m, _)| *m))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_image_follows_the_exes_state_rules() {
        // S = 1: nothing when up, the one sprite when pressed inside.
        assert_eq!(look(true, false, false, 1, false), Look::Nothing);
        assert_eq!(look(true, true, false, 1, false), Look::State(0));
        // S = 2: up, pressed, latched.
        assert_eq!(look(true, false, false, 2, false), Look::State(0));
        assert_eq!(look(true, true, false, 2, false), Look::State(1));
        assert_eq!(look(true, false, true, 2, false), Look::State(1), "a latched toggle shows pressed");
        // Disabled: its image, else up (nothing for S = 1).
        assert_eq!(look(false, true, false, 2, true), Look::Disabled);
        assert_eq!(look(false, false, false, 2, false), Look::State(0));
        assert_eq!(look(false, false, false, 1, false), Look::Nothing);
    }

    #[test]
    fn a_button_fires_on_release_anywhere_unless_it_fires_on_press() {
        let mut p = Press::default();
        assert!(!p.update(true, true, false, false), "pressing arms it");
        assert!(p.held);
        assert!(p.update(false, false, true, false), "released outside: still fires");
        assert!(!p.held);
        assert!(!p.update(false, true, false, false), "a press outside does nothing");
        assert!(!p.update(true, false, true, false), "nor a release that was not armed");
        let mut f = Press::default();
        assert!(f.update(true, true, false, true), "fire on press");
        assert!(!f.update(true, false, true, true), "and not again on release");
    }

    #[test]
    fn sprite_buttons_are_placed_from_a_centre_or_a_corner() {
        // The CSD's Bases button: 45 x 32 centred on (516, 194).
        assert_eq!(sprite_rect(Place::Centre([516.0, 194.0]), 45.0, 32.0), [493.0, 177.0, 47.0, 34.0]);
        // Bases' side buttons: 49 x 39 with the top-left at (504, 361).
        assert_eq!(sprite_rect(Place::TopLeft([504.0, 361.0]), 49.0, 39.0), [503.0, 360.0, 51.0, 41.0]);
    }

    #[test]
    fn radio_groups_latch_the_chosen_the_selected_or_the_first() {
        let m = [("a", false), ("b", true), ("c", false)];
        assert_eq!(latch(None, &m), Some("b"));
        assert_eq!(latch(Some("c"), &m), Some("c"));
        assert_eq!(latch(Some("gone"), &m), Some("b"), "a stale choice falls back");
        assert_eq!(latch(None, &[("x", false), ("y", false)]), Some("x"));
        assert_eq!(latch(None, &[]), None);
    }
}
