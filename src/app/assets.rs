//! # Asset browser
//!
//! A testbed screen for modders: every texture, image (cockpits, HUD, menus, sprites)
//! and sound on the CD, shown or played exactly as the game would load it. When a pack
//! overrides an asset, the browser shows the replacement and names the pack — the
//! quickest way to check that a mod's files are in the right place.
//!
//! Models have their own screen, the [model viewer](super::viewer).

use super::{AppContext, Next};
use crate::content::vfs::AssetKind;
use crate::content::Content;
use macroquad::prelude::*;

/// The browser's tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Textures,
    Images,
    Sounds,
}

impl Tab {
    const ALL: [Tab; 3] = [Tab::Textures, Tab::Images, Tab::Sounds];

    pub fn next(self) -> Tab {
        Self::ALL[(Self::ALL.iter().position(|t| *t == self).unwrap_or(0) + 1) % Self::ALL.len()]
    }
}

/// One browsable asset.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub label: String,
    /// What to load: a texture name, or an `original:` reference.
    pub reference: String,
    /// `Some(pack id)` if a pack overrides this asset.
    pub overridden_by: Option<String>,
}

/// Lists the assets for a tab, with override information.
pub fn entries(content: &Content, tab: Tab) -> Vec<Entry> {
    let Some(o) = content.original.as_ref() else { return Vec::new() };
    let pack_name = |hit: Option<(usize, String)>| hit.map(|(p, _)| content.packs[p].id.clone());
    match tab {
        Tab::Textures => o
            .resource_names("XTXR")
            .into_iter()
            .map(|n| Entry {
                overridden_by: pack_name(content.find_override(AssetKind::Texture, None, Some(&n))),
                label: n.clone(),
                reference: n,
            })
            .collect(),
        Tab::Images => o
            .resources_of("TARG")
            .into_iter()
            .map(|(dll, id, _)| Entry {
                overridden_by: pack_name(content.find_override(AssetKind::Image, Some(&dll), Some(&id))),
                label: format!("{dll} #{id}"),
                reference: format!("original:{dll}#{id}"),
            })
            .collect(),
        Tab::Sounds => o
            .files_with_extension("wav")
            .into_iter()
            .filter_map(|p| p.file_name().map(|f| f.to_string_lossy().into_owned()))
            .map(|f| Entry {
                overridden_by: pack_name(content.find_override(AssetKind::Sound, Some(&f), None)),
                label: f.clone(),
                reference: format!("original:{f}"),
            })
            .collect(),
    }
}

/// The asset browser screen.
pub struct AssetBrowser {
    tab: Tab,
    list: Vec<Entry>,
    selected: usize,
    /// The texture for the selected entry and the entry index it belongs to.
    shown: Option<(usize, Texture2D, u32, u32)>,
    error: Option<String>,
    sound: Option<macroquad::audio::Sound>,
}

impl AssetBrowser {
    pub fn new(ctx: &AppContext) -> AssetBrowser {
        AssetBrowser {
            tab: Tab::Textures,
            list: entries(&ctx.content, Tab::Textures),
            selected: 0,
            shown: None,
            error: None,
            sound: None,
        }
    }

    fn load_selected(&mut self, ctx: &AppContext) {
        self.error = None;
        self.shown = None;
        let Some(e) = self.list.get(self.selected) else { return };
        let image = match self.tab {
            Tab::Textures => ctx.content.load_texture(&e.reference, 0),
            Tab::Images => {
                ctx.content.load_image(&e.reference, 0, e.reference.to_ascii_lowercase().contains("cockpit"))
            }
            Tab::Sounds => return,
        };
        match image {
            Ok(img) => {
                let t = Texture2D::from_rgba8(img.width as u16, img.height as u16, &img.rgba);
                t.set_filter(FilterMode::Nearest);
                self.shown = Some((self.selected, t, img.width, img.height));
            }
            Err(e) => self.error = Some(e),
        }
    }

    /// One frame. `async` because loading a sound for playback is (macroquad's API);
    /// the app loop simply `.await`s it.
    pub async fn frame(&mut self, ctx: &mut AppContext) -> Next {
        let raw = ctx.devices.read();
        let pressed = |k: &str| raw.pressed.contains(k);
        let n = self.list.len().max(1);
        if pressed("escape") {
            return Next::Launcher;
        }
        if pressed("tab") {
            self.tab = self.tab.next();
            self.list = entries(&ctx.content, self.tab);
            self.selected = 0;
            self.shown = None;
        }
        let step = if raw.shift { 10 } else { 1 };
        if pressed("down") || pressed("n") {
            self.selected = (self.selected + step) % n;
        }
        if pressed("up") || pressed("b") {
            self.selected = (self.selected + n * 10 - step) % n;
        }
        if self.tab != Tab::Sounds && self.shown.as_ref().map(|s| s.0) != Some(self.selected) && self.error.is_none() {
            self.load_selected(ctx);
        }
        if self.tab == Tab::Sounds && (pressed("enter") || pressed("space")) {
            if let Some(e) = self.list.get(self.selected) {
                match ctx.content.load_sound(&e.reference, 0) {
                    Ok(pcm) => {
                        let bytes = pcm.to_wav_bytes();
                        if let Ok(s) = macroquad::audio::load_sound_from_bytes(&bytes).await {
                            macroquad::audio::play_sound_once(&s);
                            self.sound = Some(s);
                        }
                    }
                    Err(e) => self.error = Some(e),
                }
            }
        }
        if pressed("down") || pressed("up") || pressed("n") || pressed("b") {
            self.error = None;
        }

        clear_background(Color::new(0.06, 0.06, 0.08, 1.0));
        let title = format!(
            "Asset browser - {:?} ({} of {})   Tab: next kind   Up/Down (Shift x10)   Enter: play sound   Esc: back",
            self.tab,
            self.selected + 1,
            self.list.len()
        );
        draw_text(&title, 20.0, 30.0, 20.0, WHITE);
        if self.list.is_empty() {
            draw_text(
                "No original data loaded (run with --data, or set it in settings.yaml).",
                20.0,
                70.0,
                22.0,
                ORANGE,
            );
            return Next::Stay;
        }
        // The list, centred on the selection.
        let first = self.selected.saturating_sub(12);
        for (row, i) in (first..(first + 26).min(self.list.len())).enumerate() {
            let e = &self.list[i];
            let y = 70.0 + row as f32 * 22.0;
            let colour = if i == self.selected { YELLOW } else { LIGHTGRAY };
            let label = match &e.overridden_by {
                Some(p) => format!("{}   [overridden by {p}]", e.label),
                None => e.label.clone(),
            };
            draw_text(&label, 20.0, y, 18.0, colour);
        }
        // The preview, fitted to the right half of the screen.
        let area = Rect::new(screen_width() * 0.42, 60.0, screen_width() * 0.55, screen_height() - 100.0);
        draw_rectangle_lines(area.x, area.y, area.w, area.h, 1.0, DARKGRAY);
        if let Some((_, tex, w, h)) = &self.shown {
            let scale = (area.w / *w as f32).min(area.h / *h as f32).min(8.0);
            let size = vec2(*w as f32 * scale, *h as f32 * scale);
            let at = vec2(area.x + (area.w - size.x) * 0.5, area.y + (area.h - size.y) * 0.5);
            // A checkerboard behind shows transparency.
            for cy in 0..(size.y / 16.0).ceil() as i32 {
                for cx in 0..(size.x / 16.0).ceil() as i32 {
                    let c = if (cx + cy) % 2 == 0 {
                        Color::new(0.25, 0.25, 0.25, 1.0)
                    } else {
                        Color::new(0.35, 0.35, 0.35, 1.0)
                    };
                    draw_rectangle(
                        at.x + cx as f32 * 16.0,
                        at.y + cy as f32 * 16.0,
                        16.0_f32.min(size.x - cx as f32 * 16.0),
                        16.0_f32.min(size.y - cy as f32 * 16.0),
                        c,
                    );
                }
            }
            draw_texture_ex(tex, at.x, at.y, WHITE, DrawTextureParams { dest_size: Some(size), ..Default::default() });
            draw_text(&format!("{w} x {h}"), area.x, area.y + area.h + 24.0, 18.0, GRAY);
        }
        if let Some(e) = &self.error {
            draw_text(e, area.x, area.y + area.h + 24.0, 18.0, ORANGE);
        }
        Next::Stay
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tabs_cycle() {
        assert_eq!(Tab::Textures.next(), Tab::Images);
        assert_eq!(Tab::Sounds.next(), Tab::Textures);
    }

    #[test]
    fn no_original_data_means_empty_lists() {
        let packs = crate::content::checkout_dirs();
        let c =
            Content::load(&crate::content::ContentConfig { pack_dirs: packs, no_original: true, ..Default::default() })
                .unwrap();
        for t in Tab::ALL {
            assert!(entries(&c, t).is_empty());
        }
    }
}
