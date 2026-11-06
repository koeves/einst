//! MIDI clock sync for the standalone app, as v1's standalone shell does it
//! ([re:9b529e], [re:91ac69], option `StandaloneSyncToMIDIClock`
//! [re:15879f]): with sync on and a tempo between 30 and 300 BPM the clock
//! drives the transport ([re:be7613]) instead of the internal one.
//!
//! - Start (0xFA) / Continue (0xFB) arm the transport; the next clock
//!   (0xF8) starts it (from 0 after Start) and fixes the tick time.
//! - Each clock while running counts a tick (position = ticks / 24
//!   quarters) and measures its interval (at most 300 ms; 2 ms or less
//!   counts as 72 ms): the tempo 60000 / (24 · interval), when between 30
//!   and 300, moves 5 % of the way into a smoothed tempo whose rounding
//!   replaces the reported tempo when they differ by more than 2.
//! - Stop (0xFC) stops and goes back to 0; Song Position (0xF2) moves to
//!   its sixteenth.
//! - Between clocks the position runs on at the tempo, block by block.

/// The clock tracker (audio thread).
#[derive(Clone, Debug, Default)]
pub struct Clock {
    /// 0 running, 1 continue armed, 2 start armed (v1 `+0x118`).
    state: u8,
    /// Bit 0 changed this block, bit 1 playing (v1's VST transport flags).
    flags: u8,
    ticks: u32,
    last_ms: u32,
    smoothed: f64,
    /// The tempo reported to the engine.
    pub tempo: f64,
    /// The position in quarters.
    pub ppq: f64,
}

impl Clock {
    /// A realtime byte at `ms` (milliseconds on any running clock).
    pub fn realtime(&mut self, b: u8, ms: u32) {
        match b {
            0xf8 if self.state == 0 => {
                if self.flags & 2 != 0 {
                    self.ticks += 1;
                    self.ppq = self.ticks as f64 / 24.0;
                    let dt = ms.wrapping_sub(self.last_ms).min(300);
                    let per = if dt > 2 { dt as f64 * 24.0 } else { 72.0 };
                    let bpm = 60000.0 / per;
                    if bpm > 30.0 && bpm < 300.0 {
                        self.smoothed += (bpm - self.smoothed) * 0.05;
                        let r = (self.smoothed + 0.5).floor();
                        if (self.tempo - r).abs() > 2.0 {
                            self.tempo = r;
                        }
                    }
                    self.last_ms = ms;
                }
            }
            0xf8 => {
                if self.state == 2 {
                    self.ppq = 0.0;
                }
                self.state = 0;
                self.flags |= 3;
                self.last_ms = ms;
                if !(30.0..=300.0).contains(&self.tempo) {
                    self.tempo = 120.0;
                    self.smoothed = 120.0;
                }
            }
            0xfa => self.state = 2,
            0xfb => self.state = 1,
            0xfc => {
                self.flags = (self.flags & !3) + 1;
                self.ppq = 0.0;
                self.ticks = 0;
            }
            _ => {}
        }
    }

    /// Song Position Pointer (sixteenths).
    pub fn song_position(&mut self, sixteenths: u16) {
        self.ppq = sixteenths as f64 * 0.25;
        self.ticks = sixteenths as u32 * 6;
    }

    /// The clock drives the transport (a tempo in 30..300 BPM).
    pub fn active(&self) -> bool {
        self.tempo > 30.0 && self.tempo < 300.0
    }

    pub fn playing(&self) -> bool {
        self.flags & 2 != 0
    }

    /// After a block of `frames` at `sr`: the position runs on.
    pub fn advance(&mut self, frames: usize, sr: f32) {
        self.flags &= !1;
        if sr > 0.0 {
            self.ppq += frames as f64 / sr as f64 * self.tempo / 60.0;
        }
    }

    /// Feed one MIDI message received at `ms`.
    pub fn message(&mut self, bytes: &[u8], ms: u32) {
        match bytes {
            [0xf2, lsb, msb, ..] => self.song_position((*lsb as u16 & 0x7f) | ((*msb as u16 & 0x7f) << 7)),
            [b, ..] if *b >= 0xf8 => self.realtime(*b, ms),
            _ => {}
        }
    }
}

/// The standalone's clock input: a MIDI input connection feeding a queue the
/// audio thread drains.
#[cfg(feature = "standalone")]
pub struct ClockInput {
    _conn: midir::MidiInputConnection<()>,
    rx: std::sync::mpsc::Receiver<(u64, [u8; 3], usize)>,
    pub clock: Clock,
}

#[cfg(feature = "standalone")]
impl ClockInput {
    /// With `StandaloneSyncToMIDIClock` on: listen on the chosen MIDI input
    /// (the first one when none is chosen).
    pub fn open() -> Option<ClockInput> {
        let o = einst_editor::options::Options::load();
        if !o.bool("StandaloneSyncToMIDIClock", false) {
            return None;
        }
        let want = einst_editor::devices::Settings::load(&o).midi_in;
        let mut input = midir::MidiInput::new("Einst clock").ok()?;
        input.ignore(midir::Ignore::None);
        let ports = input.ports();
        let port = ports.iter().find(|p| want.is_empty() || input.port_name(p).is_ok_and(|n| n == want))?.clone();
        let (tx, rx) = std::sync::mpsc::sync_channel(1024);
        let conn = input
            .connect(
                &port,
                "einst-clock",
                move |stamp, bytes, _| {
                    if bytes.first().is_some_and(|b| *b >= 0xf8 || *b == 0xf2) {
                        let mut m = [0u8; 3];
                        let n = bytes.len().min(3);
                        m[..n].copy_from_slice(&bytes[..n]);
                        let _ = tx.try_send((stamp, m, n));
                    }
                },
                (),
            )
            .ok()?;
        einst_editor::CLOCK_SYNC.store(true, std::sync::atomic::Ordering::Relaxed);
        Some(ClockInput { _conn: conn, rx, clock: Clock::default() })
    }

    /// Feed the queued messages (audio thread; no blocking).
    pub fn drain(&mut self) {
        while let Ok((stamp, m, n)) = self.rx.try_recv() {
            if m[0] == 0xf8 {
                einst_editor::CLOCK_LAST_MS.store(einst_editor::now_ms(), std::sync::atomic::Ordering::Relaxed);
            }
            self.clock.message(&m[..n], (stamp / 1000) as u32);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_follows_v1() {
        let mut c = Clock::default();
        // Clocks before Start are ignored.
        c.realtime(0xf8, 0);
        assert!(!c.playing());
        c.realtime(0xfa, 0);
        c.realtime(0xf8, 10);
        assert!(c.playing());
        assert_eq!((c.tempo, c.ppq), (120.0, 0.0));
        // 125 BPM: 20 ms per tick; the smoothed tempo creeps up and the
        // reported one moves only by more than 2 (so it settles at 123).
        for k in 1..=200 {
            c.realtime(0xf8, 10 + 20 * k);
        }
        assert_eq!(c.ppq, 200.0 / 24.0);
        assert_eq!(c.tempo, 123.0);
        c.realtime(0xfc, 4010);
        assert!(!c.playing());
        assert_eq!(c.ppq, 0.0);
        c.message(&[0xf2, 0x10, 0x00], 4020);
        assert_eq!(c.ppq, 4.0);
    }
}
