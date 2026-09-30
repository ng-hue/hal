//! Voice mode — **currently a stub.**
//!
//! Right now this only fakes a "breathing" level while voice mode is on, so
//! the orb already feels alive. The real pipeline will replace
//! `simulated_level` without touching the UI, because the orb only reads
//! `AppState::voice_level`:
//!
//! 1. `cpal`       — capture the mic, compute RMS loudness per buffer
//! 2. `whisper-rs` — speech → text, then `actions::send_message`
//! 3. TTS (Piper)  — text → speech, played with `rodio`; drive the level from
//!                   the *output* audio while hal is speaking

use std::time::Duration;

use dioxus::prelude::*;

use crate::state::{AppState, Mode};

const TICK: Duration = Duration::from_millis(33); // ~30 fps is plenty for a glow

pub fn use_voice_engine(state: AppState) {
    use_future(move || async move {
        let mut state = state;
        let mut t: f32 = 0.0;
        loop {
            tokio::time::sleep(TICK).await;
            t += TICK.as_secs_f32();

            let target = match *state.mode.peek() {
                Mode::Voice => simulated_level(t),
                Mode::Manual => 0.0,
            };

            // Ease toward the target so changes feel physical, not jumpy.
            let current = *state.voice_level.peek();
            let next = current + (target - current) * 0.12;
            if (next - current).abs() > 0.002 {
                state.voice_level.set(next);
            }
        }
    });
}

/// Three out-of-phase sine waves = an irregular, organic-looking pulse.
fn simulated_level(t: f32) -> f32 {
    let wobble = (t * 1.3).sin() * 0.5 + (t * 3.7).sin() * 0.3 + (t * 7.9).sin() * 0.2;
    (0.4 + 0.3 * wobble).clamp(0.0, 1.0)
}
