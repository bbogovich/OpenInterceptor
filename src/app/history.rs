//! # The screen history (RQ-UI-15)
//!
//! How the original strategy screens remember where Back goes (Interceptor.exe
//! 0x4b5bfe-0x4b5e5b; verified):
//!
//! * **push(n)** (`open(n)`): the **home** screen (the CSD) clears the history. Any other
//!   screen, if it is not already the current one, is switched to; the current screen is
//!   remembered first **unless it is already in the history or the history is full**
//!   (29 screens).
//! * **menu(n)** (`menu(n)`): like push, but in a run of menu-bar switches only the
//!   first remembers the screen it left; later ones replace.
//! * **replace(n)** (`replace(n)`): switch without remembering.
//! * **back()**: return to the last remembered screen.
//!
//! Dialog screens (`dialog: true`) are not part of the history: they stack over the
//! current screen, and switching screens closes them.
//!
//! Plain owned data (`Vec<String>`), no references into the ruleset, so it can be cloned
//! and tested on its own.

/// The original's history size.
pub const DEFAULT_LIMIT: usize = 29;

/// Where the strategy screens are: the current screen, the ones Back returns to, and
/// the dialogs open over the current screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct History {
    /// The home screen: opening it clears the history.
    pub home: String,
    /// The screen shown (under any dialogs).
    pub current: String,
    /// Remembered screens, oldest first.
    pub trail: Vec<String>,
    /// Dialogs over the current screen, bottom first.
    pub dialogs: Vec<String>,
    /// The most screens remembered.
    pub limit: usize,
    /// The last switch was a menu switch (a run of them remembers only once).
    in_menu_run: bool,
}

impl History {
    /// Starts on the home screen.
    pub fn new(home: &str, limit: usize) -> History {
        History {
            home: home.to_string(),
            current: home.to_string(),
            trail: Vec::new(),
            dialogs: Vec::new(),
            limit,
            in_menu_run: false,
        }
    }

    /// The screens to draw, bottom first: the current one and its dialogs.
    pub fn shown(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.current.as_str()).chain(self.dialogs.iter().map(String::as_str))
    }

    /// The top screen or dialog (the one that takes input).
    pub fn top(&self) -> &str {
        self.dialogs.last().unwrap_or(&self.current)
    }

    /// `open(n)`: the original's push.
    pub fn push(&mut self, n: &str) {
        self.in_menu_run = false;
        self.switch(n, true);
    }

    /// `menu(n)`: only the first of a run of menu switches remembers.
    pub fn menu(&mut self, n: &str) {
        let remember = !self.in_menu_run;
        self.switch(n, remember);
        self.in_menu_run = true;
    }

    /// `replace(n)`: switch without remembering.
    pub fn replace(&mut self, n: &str) {
        self.in_menu_run = false;
        self.switch(n, false);
    }

    /// `back`: close the top dialog, else return to the last remembered screen. Returns
    /// whether anything changed.
    pub fn back(&mut self) -> bool {
        self.in_menu_run = false;
        if self.dialogs.pop().is_some() {
            return true;
        }
        match self.trail.pop() {
            Some(p) => {
                self.current = p;
                true
            }
            None => false,
        }
    }

    /// Opens a dialog over the current screen (once).
    pub fn open_dialog(&mut self, n: &str) {
        if self.dialogs.last().map(String::as_str) != Some(n) {
            self.dialogs.push(n.to_string());
        }
    }

    fn switch(&mut self, n: &str, remember: bool) {
        self.dialogs.clear();
        if n == self.home {
            self.trail.clear();
            self.current = n.to_string();
            return;
        }
        if n == self.current {
            return;
        }
        let known = self.trail.contains(&self.current);
        if remember && !known && self.trail.len() < self.limit {
            self.trail.push(self.current.clone());
        }
        self.current = n.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_screens_remembers_them_and_home_clears() {
        let mut h = History::new("csd", 29);
        h.push("bases");
        h.push("purchase");
        assert_eq!((h.current.as_str(), h.trail.clone()), ("purchase", vec!["csd".to_string(), "bases".into()]));
        assert!(h.back());
        assert_eq!(h.current, "bases");
        h.push("csd");
        assert!(h.trail.is_empty() && h.current == "csd", "the home screen clears the history");
        assert!(!h.back(), "nothing to go back to");
        h.push("csd");
        assert_eq!(h.current, "csd");
    }

    #[test]
    fn a_screen_already_remembered_or_a_full_history_is_not_pushed_again() {
        let mut h = History::new("csd", 29);
        h.push("bases");
        h.push("research");
        h.push("bases");
        // research is remembered (not yet in the trail); then bases is current, already in it.
        assert_eq!(h.trail, vec!["csd".to_string(), "bases".into(), "research".into()]);
        h.push("purchase");
        assert_eq!(h.trail.len(), 3, "bases was already remembered");
        let mut small = History::new("csd", 2);
        for s in ["a", "b", "c", "d"] {
            small.push(s);
        }
        assert_eq!(small.trail, vec!["csd".to_string(), "a".into()], "full at 2");
        h.push("purchase");
        assert_eq!(h.current, "purchase", "opening the current screen changes nothing");
    }

    #[test]
    fn menu_runs_remember_once_and_replace_never() {
        let mut h = History::new("csd", 29);
        h.push("bases");
        h.menu("research");
        h.menu("purchase");
        h.menu("ufopedia");
        assert_eq!(h.trail, vec!["csd".to_string(), "bases".into()], "only the first menu switch remembered");
        h.replace("reports");
        assert_eq!((h.current.as_str(), h.trail.len()), ("reports", 2));
        h.back();
        assert_eq!(h.current, "bases");
    }

    #[test]
    fn dialogs_stack_over_the_screen_and_close_on_switch_or_back() {
        let mut h = History::new("csd", 29);
        h.open_dialog("orders");
        h.open_dialog("orders");
        assert_eq!(h.dialogs.len(), 1);
        assert_eq!(h.top(), "orders");
        assert_eq!(h.shown().collect::<Vec<_>>(), vec!["csd", "orders"]);
        assert!(h.back());
        assert_eq!(h.top(), "csd");
        h.open_dialog("orders");
        h.push("bases");
        assert!(h.dialogs.is_empty(), "switching screens closes dialogs");
    }
}
