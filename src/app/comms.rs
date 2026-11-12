//! # The cockpit comms panel (Stage 6.6)
//!
//! When a voice speaks on the radio — a wingman answering an order, mission chatter —
//! a small panel on the left of the screen shows who is talking for as long as the clip
//! lasts: the voice set's `face` (a still picture, or a Smacker clip that loops while the
//! voice speaks) and its `name`. A voice without a face gets an animated signal graphic,
//! so a campaign with no portrait art still shows that someone is on the air.
//!
//! Faces are loaded the first time their voice speaks, not with the battle: most voice
//! sets never speak in a given battle, and a talking-head clip is a few megabytes once
//! decoded. A face that fails to load is reported once and drawn as the signal graphic.

use crate::content::rules::Ruleset;
use crate::content::Content;
use crate::formats::smacker::Smacker;
use macroquad::prelude::*;

/// A loaded face.
enum Face {
    /// Every frame of a clip, and the seconds each is shown.
    Clip(Vec<Texture2D>, f32),
    Still(Texture2D),
    /// Tried and failed (or none given): the signal graphic.
    Signal,
}

/// The panel's state.
#[derive(Default)]
pub struct Comms {
    /// Per voice set; `None` until it first speaks.
    faces: Vec<Option<Face>>,
    /// Who is speaking: voice set, until (battle seconds), and when it started (wall clock).
    speaking: Option<(Option<usize>, f64, f64)>,
}

/// Seconds the panel stays up after a clip whose length is unknown.
const DEFAULT_SECONDS: f32 = 3.0;

/// How long the panel stays up for a clip of `length` seconds: the clip plus a short
/// tail, so the face does not vanish on the last syllable — and never less than 2.5 s,
/// long enough to read the name after a short chirp.
pub fn showing_for(length: Option<f32>) -> f32 {
    length.map_or(DEFAULT_SECONDS, |l| (l.clamp(0.5, 30.0) + 0.6).max(2.5))
}

/// The signal graphic's bar heights (0..1) at time `t`: a pulse moving across `n` bars,
/// so it reads as speech without any art. Pure, so it is tested.
pub fn signal_bars(n: usize, t: f32) -> Vec<f32> {
    (0..n)
        .map(|i| {
            let x = i as f32 * 0.9 + t * 9.0;
            (0.25 + 0.75 * (0.5 + 0.5 * x.sin()) * (0.6 + 0.4 * (x * 0.37 + 1.3).sin().abs())).clamp(0.05, 1.0)
        })
        .collect()
}

impl Comms {
    /// A voice clip has started at battle time `now`, said by `speaker` (a voice set) or
    /// by an unnamed voice on the radio.
    pub fn speak(&mut self, content: &Content, rules: &Ruleset, speaker: Option<usize>, length: Option<f32>, now: f64) {
        let speaker = speaker.filter(|v| *v < rules.voices.len());
        if let Some(v) = speaker {
            if self.faces.len() < rules.voices.len() {
                self.faces.resize_with(rules.voices.len(), || None);
            }
            if self.faces[v].is_none() {
                self.faces[v] = Some(load_face(content, &rules.voices[v]));
            }
        }
        self.speaking = Some((speaker, now + f64::from(showing_for(length)), get_time()));
    }

    /// Draws the panel if someone is speaking at battle time `now`.
    pub fn draw(&self, rules: &Ruleset, now: f64) {
        let Some((v, until, started)) = self.speaking else { return };
        if now >= until {
            return;
        }
        let sh = screen_height();
        let h = (sh * 0.2).clamp(90.0, 220.0);
        let w = h * 4.0 / 3.0;
        let (x, y) = (12.0, sh * 0.09); // top left, under the status line, clear of the panel instruments
        let ink = Color::new(0.55, 1.0, 0.6, 0.9);
        draw_rectangle(x - 3.0, y - 3.0, w + 6.0, h + 26.0, Color::new(0.0, 0.0, 0.0, 0.65));
        draw_rectangle_lines(x - 3.0, y - 3.0, w + 6.0, h + 26.0, 1.5, ink);
        let since = (get_time() - started) as f32;
        let texture = match v.and_then(|v| self.faces.get(v)).and_then(Option::as_ref) {
            Some(Face::Clip(frames, ft)) if !frames.is_empty() => {
                Some(&frames[(since / ft.max(0.01)) as usize % frames.len()])
            }
            Some(Face::Still(t)) => Some(t),
            _ => None,
        };
        match texture {
            Some(t) => {
                draw_texture_ex(t, x, y, WHITE, DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() })
            }
            None => {
                let bars = signal_bars(12, since);
                let bw = w / bars.len() as f32;
                for (i, b) in bars.iter().enumerate() {
                    let bh = h * 0.8 * b;
                    draw_rectangle(x + i as f32 * bw + 2.0, y + (h - bh) * 0.5, bw - 4.0, bh, ink);
                }
            }
        }
        let name = match v.map(|v| &rules.voices[v]) {
            Some(voice) => voice.name.as_deref().unwrap_or(&voice.id).to_uppercase(),
            None => "COMMS".to_string(),
        };
        draw_text(&name, x + 2.0, y + h + 17.0, 18.0, ink);
    }

    /// Clears the panel (a new battle).
    pub fn clear(&mut self) {
        self.speaking = None;
    }
}

/// Loads a voice set's face: a Smacker clip (`.smk`, possibly inside an MPSRC pack),
/// a picture, or the signal graphic.
fn load_face(content: &Content, voice: &crate::content::rules::VoiceSet) -> Face {
    let Some(reference) = &voice.face else { return Face::Signal };
    let made = if reference.to_ascii_lowercase().ends_with(".smk") {
        content.read_asset(reference, voice.pack).and_then(|bytes| clip_frames(&bytes))
    } else {
        content.load_image(reference, voice.pack, false).map(|img| {
            let t = Texture2D::from_rgba8(img.width as u16, img.height as u16, &img.rgba);
            t.set_filter(FilterMode::Linear);
            Face::Still(t)
        })
    };
    made.unwrap_or_else(|e| {
        eprintln!("voices.{}: face {reference}: {e}", voice.id);
        Face::Signal
    })
}

/// Every frame of a Smacker clip as a texture.
fn clip_frames(bytes: &[u8]) -> Result<Face, String> {
    let mut s = Smacker::new(bytes).map_err(|e| e.to_string())?;
    let (w, h) = (s.width as u16, s.out_height() as u16);
    let mut frames = Vec::with_capacity(s.frames as usize);
    while s.next_frame().map_err(|e| e.to_string())?.is_some() {
        let t = Texture2D::from_rgba8(w, h, &s.rgba());
        t.set_filter(FilterMode::Linear);
        frames.push(t);
    }
    Ok(Face::Clip(frames, s.frame_time))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panel_lasts_the_clip_and_the_signal_moves() {
        assert_eq!(showing_for(None), DEFAULT_SECONDS);
        assert!((showing_for(Some(2.0)) - 2.6).abs() < 1e-6);
        assert_eq!(showing_for(Some(0.3)), 2.5, "a chirp still shows long enough to read");
        assert!(showing_for(Some(500.0)) <= 31.0, "a runaway clip does not pin the panel");
        let (a, b) = (signal_bars(12, 0.0), signal_bars(12, 0.2));
        assert_eq!(a.len(), 12);
        assert!(a.iter().all(|h| (0.05..=1.0).contains(h)));
        assert_ne!(a, b, "the bars move");
    }
}
