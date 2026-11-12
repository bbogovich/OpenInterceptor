# Immediate-mode UI

The strategy screen (app 0.16.0) is built from **immediate-mode** widgets: there are no button
objects. Each frame the screen *asks* for a button — where, what label — and the same call draws
it and answers whether it was clicked. This page shows the pattern in `src/app/widgets.rs` and
`src/app/strategy.rs`, and why it fits Rust so well.

## Retained mode, and what goes wrong in Rust

In a **retained-mode** toolkit (most desktop GUI libraries, the browser's DOM) you create widget
objects once, keep them in a tree, and give each a *callback* to run when it is clicked:

```text
let buy = Button::new("Buy 1");
buy.on_click(|| campaign.buy(item, 1));   // the closure must reach the campaign later
panel.add(buy);
```

Two things make this awkward in Rust:

- **The callback outlives the frame.** The closure is stored inside the button, so whatever it
  touches (`campaign`) must stay borrowed for as long as the button exists. Rust does not allow a
  long-lived `&mut campaign` inside a widget while the screen also uses the campaign; you end up
  with `Rc<RefCell<Campaign>>` everywhere, and borrow checks move from compile time to run time.
- **Two copies of the truth.** The button's label, its enabled flag and the list's rows are
  copies of the campaign's state. When the campaign changes (money arrives, a craft leaves), every
  widget that mirrors it must be updated, or the screen shows stale data.

## Immediate mode: the widget is a function call

`widgets::button` is an ordinary function. It borrows what it needs **only for the call**:

```rust
/// A push button: drawn in the skin, highlighted under the mouse, dimmed when disabled.
/// Returns whether it was clicked this frame.
pub fn button(ui: &Ui, input: &Input, r: Rect4, label: &str, enabled: bool) -> bool {
    let hover = enabled && contains(r, input.mouse);
    // … draw the rectangle and the label …
    enabled && input.clicked_in(r)
}
```

The caller decides what a click means, right there, with its own `&mut self`:

```rust
// draw_top, every frame
if widgets::button(ui, input, [bx, y, bh, bh], "<", nb > 1) {
    self.base = (self.base + nb - 1) % nb;   // no callback: just code after an `if`
}
```

Nothing is stored, so nothing outlives the frame, and the borrow checker sees short, ordinary
borrows. The label and the enabled flag are computed from the campaign **this frame**, so they
cannot be stale.

## Returning what happened instead of doing it

A list row may need to change the campaign, but `draw_panel` is busy borrowing the screen to draw.
So the drawing functions **return a value** describing the action, and the caller performs it
after the drawing borrow has ended:

```rust
// StrategyScreen::frame
if let Some(a) = self.draw_panel(ui, def, panel, &live, k) {   // borrows self while drawing
    self.act(def, a);                                           // then mutates self
}
```

`Act` is a small `Copy` enum (`Act::Buy(Product, u32)`, `Act::FitBest(usize)`, …): it carries
indices, not references, so it is free to hand from the drawing code to the code that changes
the campaign. The same idea runs through the engine: index handles instead of references
([Ownership and borrowing](ownership.md)).

## The state that must persist is plain data

Some things do outlive a frame: which row of a list is selected and how far it is scrolled. They
live in a plain struct the screen owns, and the arithmetic is pure functions on it:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ListView {
    pub first: usize,    // the first row shown
    pub selected: usize, // the selected row
}

impl ListView {
    /// Moves the selection by `by` rows and scrolls just enough to keep it in view.
    pub fn step(&mut self, by: isize, n: usize, visible: usize) { … }
}
```

Because `ListView`, `wrap` and `grid` take numbers and return numbers, they are tested without a
window or a font — `wrap` takes the measuring function as a parameter, and the test passes a fake
one (`&dyn Fn(&str) -> f32`, a *trait object*: any closure with that signature):

```rust
fn m(s: &str) -> f32 { s.chars().count() as f32 * 10.0 }   // every character 10 px

#[test]
fn text_wraps_between_words_and_cuts_only_giant_words() {
    assert_eq!(wrap("one two three", 80.0, &m), vec!["one two", "three"]);
}
```

## Blocking input under a prompt

A modal prompt must stop the screen beneath from reacting. With retained widgets you would
disable each one. Here the screen reads the input **once** into a `Copy` struct and simply hands
the widgets beneath an empty one:

```rust
let input = Input::read();
let live = if prompting { Input::default() } else { input };   // no clicks, no wheel
let next = self.draw_top(ctx, def, top, &live, k);              // draws, but cannot be clicked
```

`#[derive(Default)]` gives `Input::default()` with the mouse at the origin and every flag
`false` — a one-line way to say "nothing happened".

## The trade-off

Immediate mode redraws and recomputes everything every frame. For a screen of a few hundred rows
and buttons that is far below a frame's budget; for very large lists you would compute only the
visible rows, as `draw_panel` already does (`skip(first).take(visible)`). Layouts that depend on
measuring many widgets before placing them are harder than in a retained toolkit; here the pack's
[screen layout](../../modding/screens.md) supplies the rectangles, so no layout pass is needed.

## Where to look

- `src/app/widgets.rs` — `Input`, `button`, `button_row`, `grid`, `ListView`, `wrap`, `modal`.
- `src/app/strategy.rs` — `frame`, `draw_top`, `draw_info`, `draw_panel`, `prompts`, `Act`, `Row`.
- [Architecture › Presentation › Widgets and screen layouts](../../implementation/architecture/presentation.md#widgets-and-screen-layouts-appwidgets-contentscreens).
