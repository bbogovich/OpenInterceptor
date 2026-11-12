//! # Playing a video (Stage 6.3)
//!
//! A [`VideoPlayer`] shows a Smacker file ([`crate::formats::smacker`]) centred on the
//! screen, letterboxed to fit: frames are decoded as the clock reaches them (so a slow
//! frame is skipped rather than slowing the film), and the first audio track is decoded
//! up front and played as one sound, so picture and sound start together. Enter, Esc or a
//! click ends it.

use crate::content::Content;
use crate::formats::smacker::Smacker;
use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};
use macroquad::prelude::*;

/// A video being played.
pub struct VideoPlayer {
    data: Vec<u8>,
    width: u32,
    height: u32,
    frame_time: f32,
    frames: u32,
    /// Frames decoded so far (the decoder is rebuilt from `data` on each frame step, as
    /// it borrows the bytes; it keeps its own state between calls through `state`).
    shown: u32,
    state: Option<(Vec<u8>, [[u8; 3]; 256])>,
    texture: Texture2D,
    sound: Option<Sound>,
    started: f64,
}

impl VideoPlayer {
    /// Opens a video asset. Its whole soundtrack is decoded now. A pack path not found in
    /// `pack` is looked for in the other packs, latest first (a story event's film lives
    /// in whichever pack defines the event).
    pub async fn open(content: &Content, reference: &str, pack: usize) -> Result<VideoPlayer, String> {
        let data = content.read_asset(reference, pack).or_else(|first| {
            (0..content.packs.len()).rev().find_map(|p| content.read_asset(reference, p).ok()).ok_or(first)
        })?;
        let mut s = Smacker::new(&data).map_err(|e| e.to_string())?;
        let (width, height, frames, frame_time) = (s.width, s.out_height(), s.frames, s.frame_time);
        let track = s.tracks[0];
        let mut samples = Vec::new();
        while let Some(audio) = s.next_frame().map_err(|e| e.to_string())? {
            for (t, a) in audio {
                if t == 0 {
                    samples.extend(a);
                }
            }
        }
        let sound = match track {
            Some(t) if !samples.is_empty() => {
                let pcm =
                    crate::formats::wav::Pcm { sample_rate: t.rate, channels: if t.stereo { 2 } else { 1 }, samples };
                load_sound_from_bytes(&pcm.to_wav_bytes()).await.ok()
            }
            _ => None,
        };
        let texture = Texture2D::from_rgba8(width as u16, height as u16, &vec![0; (width * height * 4) as usize]);
        texture.set_filter(FilterMode::Linear);
        Ok(VideoPlayer { data, width, height, frame_time, frames, shown: 0, state: None, texture, sound, started: 0.0 })
    }

    /// One frame: advances to the frame the clock has reached, draws it, and returns
    /// `true` when the video is over (ended, or skipped).
    /// One frame; `volume` is the soundtrack's (the effects volume). `true` when over.
    pub fn frame(&mut self, volume: f32) -> bool {
        if self.shown == 0 && self.started == 0.0 {
            self.started = get_time();
            if let Some(s) = &self.sound {
                play_sound(s, PlaySoundParams { looped: false, volume: 0.9 * volume });
            }
        }
        let skip = is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::Escape)
            || is_mouse_button_pressed(MouseButton::Left);
        let due = (((get_time() - self.started) as f32 / self.frame_time) as u32 + 1).min(self.frames);
        if due > self.shown {
            // Decode up to the due frame. (A decoder borrows the bytes, so it is rebuilt
            // and fast-forwarded only the first time; after that the last frame's pixels
            // and palette carry the state.)
            if let Ok(mut s) = Smacker::new(&self.data) {
                if let Some((px, pal)) = &self.state {
                    s.pixels.clone_from(px);
                    s.palette = *pal;
                }
                s.skip_to(self.shown);
                while self.shown < due {
                    if s.next_frame().ok().flatten().is_none() {
                        break;
                    }
                    self.shown += 1;
                }
                self.texture.update(&Image { bytes: s.rgba(), width: self.width as u16, height: self.height as u16 });
                self.state = Some((s.pixels.clone(), s.palette));
            }
        }
        clear_background(BLACK);
        let (sw, sh) = (screen_width(), screen_height());
        let k = (sw / self.width as f32).min(sh / self.height as f32);
        let (w, h) = (self.width as f32 * k, self.height as f32 * k);
        draw_texture_ex(
            &self.texture,
            (sw - w) / 2.0,
            (sh - h) / 2.0,
            WHITE,
            DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() },
        );
        let over = skip
            || (self.shown >= self.frames && get_time() - self.started > (self.frames as f32 * self.frame_time) as f64);
        if over {
            if let Some(s) = &self.sound {
                stop_sound(s);
            }
        }
        over
    }
}
