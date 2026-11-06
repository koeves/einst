//! Einst: Dreist's port of the Ring Mod (`dreist_dsp::fx::nodes::ring_mod`
//! with the oscillator of `nodes::distortion`, copied from dreist-commons)
//! with v1's differences (`Voice::ProcessBuffer` ev_2c2fb94d,
//! `Voice::Reset` ev_cd0b5b9a, v1's oscillator init ev_6bae725f, v1's
//! step-table builder ev_6a9c1926; static disassembly of the v1 engine at
//! 0x54caa0 (voice constructor), 0x445318 (seed) and 0x4ed464 (builder)):
//!
//! - the band-limited step tables are live: v1 builds them once at start
//!   ([`bl_tables`]), so every step event adds `lerp(step)·amp` (saw, pulse,
//!   parabola) or the integrated `lerp(ramp)` correction (triangle) while it
//!   ages, with the fractional table position kept per event;
//! - the oscillator init draws a random start phase from the oscillator's
//!   own generator (`phase = (bits(s>>9 | 0x40000000) − 2)/2`), and sets
//!   sign, slope, the saw accumulator (`phase − ½`) and the pulse/triangle
//!   accumulator (`±½`) from it, before the five settling ticks;
//! - the white/pink sample-and-hold uses that same generator (seeded once in
//!   the voice constructor, re-seeded by init for the noise modes; the pink
//!   rows survive inits), sums `(w + sum) − old`, and holds the saw
//!   accumulator, so a fresh noise mode holds `phase − ½` until its first
//!   trigger;
//! - the frequency `2^Pitch` is v1's [`fast_exp2`](crate::fast_exp2),
//!   multiplied as `(1/sr · tune) · 2^Pitch`; the mode rounds as
//!   `(int)(m + ½)`;
//! - output sums in v1's order per mode (e.g. `(dry + wet·s)·x` for sine,
//!   triangle and noise).
//!
//! Mode 6 (pink) calls init with mode 11 where Dreist passes 10; v1's init
//! treats 10 and 11 alike (re-seed only), so this changes nothing.
//!
//! [re:691de1] (Ring Mod, kind 24; `Voice::Reset` [re:444071],
//! `Voice::ProcessBuffer` [re:d5dbeb]): soft-clipped input times a carrier at
//! `2^Pitch · tune` Hz (clamp 0.5 cycles/sample): sine ([re:19db55]),
//! triangle, saw, pulse, parabola, white or pink sample-and-hold, wet/dry.

use crate::fast_exp2;
use crate::math::{clip01, sat};
use dreist_dsp::analog::sin_tab;
use dreist_dsp::fx::nodes::blocks::Rng;
use dreist_dsp::fx::{Ctx, Node, Port, ports, ports_from};
use std::sync::OnceLock;

/// Length of each band-limited table (16× oversampled, 9 samples wide).
pub const BL_LEN: usize = 144;

/// The tables of v1's step-table builder that the ring modulator reads
/// (plus the windowed sinc they integrate).
pub struct BlTables {
    /// Kaiser-windowed sinc, normalised to sum 1.
    pub sinc: [f32; BL_LEN],
    /// Running sum of `sinc` (0 → 1): the band-limited unit step.
    pub step: [f32; BL_LEN],
    /// Running sum of `step`, ×1/16: the band-limited ramp.
    pub ramp: [f32; BL_LEN],
}

/// `I0`-style series `Σ (zⁿ/n!)²` (n ≥ 2 terms until one is ≤ 1e−6), added
/// to `first` (the n = 0, 1 terms), in v1's f32 order.
fn bessel_tail(mut sum: f32, z: f32) -> f32 {
    let (mut p, mut fact, mut n) = (z, 1.0f32, 2);
    loop {
        p *= z;
        fact *= n as f32;
        let t = (p / fact) * (p / fact);
        sum += t;
        n += 1;
        if t <= 1e-6 {
            return sum;
        }
    }
}

/// v1's step-table builder (ev_6a9c1926), run once. The builder also
/// computes parabola tables and "split" tables the ring modulator never
/// reads; they are left out. Its `ramp[120] − 3.5625` is a register value
/// used only for the split tables (0x4ed464 loads, never stores it), so
/// `ramp` stays the plain running sum.
pub fn bl_tables() -> &'static BlTables {
    static T: OnceLock<BlTables> = OnceLock::new();
    T.get_or_init(|| {
        let mut t = BlTables { sinc: [0.0; BL_LEN], step: [0.0; BL_LEN], ramp: [0.0; BL_LEN] };
        let mut acc = 0.0f32;
        for k in 0..BL_LEN {
            let x = (k as i32 - 64) as f32 * 0.05882353;
            let mut w = 0.0f32;
            if x.abs() < 4.0 {
                let q = x * 0.25;
                let z = (1.0 - q * q).sqrt() * 8.0;
                let mut num = z * z + 1.0;
                if 1e-6 < z * z {
                    num = bessel_tail(num, z);
                }
                // Denominator: the same series at z = 8 (1 + 64 + …).
                let mut den = 65.0f32;
                let (mut p, mut fact, mut n) = (8.0f32, 1.0f32, 2);
                loop {
                    p *= 8.0;
                    fact *= n as f32;
                    let t = (p / fact) * (p / fact);
                    den += t;
                    n += 1;
                    if t <= 1e-6 {
                        break;
                    }
                }
                w = num / den;
            }
            let a = x * 1.5707964;
            let s = if 1e-20 <= a.abs() { a.sin() / a } else { 1.0 };
            acc += w * s;
            t.sinc[k] = w * s;
            t.step[k] = acc;
        }
        let inv = 1.0 / t.step[BL_LEN - 1];
        for k in 0..BL_LEN {
            t.sinc[k] *= inv;
            t.step[k] *= inv;
        }
        let mut r = t.step[0];
        t.ramp[0] = r;
        for k in 1..BL_LEN {
            r += t.step[k];
            t.ramp[k] = r;
        }
        for v in t.ramp.iter_mut() {
            *v *= 0.0625;
        }
        t
    })
}

/// `frac·tab[idx+1] + (1 − frac)·tab[idx]`. v1 reads whatever memory lies
/// outside the table for an index out of range (only reachable with a
/// non-positive frequency); here such reads give 0.
#[inline]
fn lerp(tab: &[f32; BL_LEN], idx: i32, frac: f32) -> f32 {
    let at = |i: i32| usize::try_from(i).ok().and_then(|i| tab.get(i)).copied().unwrap_or(0.0);
    frac * at(idx.wrapping_add(1)) + (1.0 - frac) * at(idx)
}

/// Clamp to ±1: `(|x + 1| − |x − 1|)/2`.
#[inline]
fn clamp1(x: f32) -> f32 {
    ((x + 1.0).abs() - (x - 1.0).abs()) * 0.5
}

/// A step event: table position (`idx` in 1/16 samples plus `frac`), its
/// amplitude, and for the triangle the last ramp value and its integral.
#[derive(Debug, Clone, Copy, Default)]
struct Slot {
    idx: i32,
    frac: f32,
    amp: f32,
    ramp: f32,
    integ: f32,
}

/// Voss–McCartney pink rows of the oscillator's generator (count starts
/// at 1, sum and rows at 0 in the voice constructor; init leaves them).
#[derive(Debug, Clone, Copy)]
struct Pink {
    count: u32,
    sum: f32,
    rows: [f32; 5],
}

/// v1's oscillator, reduced to what the ring modulator runs. v1 allots 128
/// event slots; at most a handful are ever live (each lives ≤ 8 ticks), so
/// 32 are kept, and a spawn beyond them is dropped.
#[derive(Debug, Clone, Copy)]
struct Osc {
    slots: [Slot; 32],
    live: usize,
    ring: [f32; 4],
    ring_pw: [f32; 4],
    ring_idx: usize,
    acc_saw: f32,
    acc: f32,
    slope: f32,
    prev: f32,
    phase_saw: f32,
    phase: f32,
    sign: f32,
    hp: f32,
    pw: f32,
    rng: Rng,
    pink: Pink,
}

impl Osc {
    /// Voice constructor: event slots cleared, generator seeded once from
    /// the shared seed (0x54ce44), pink rows cleared with count 1.
    fn new() -> Osc {
        Osc {
            slots: [Slot::default(); 32],
            live: 0,
            ring: [0.0; 4],
            ring_pw: [0.5; 4],
            ring_idx: 0,
            acc_saw: 0.0,
            acc: 0.0,
            slope: 0.0,
            prev: 0.0,
            phase_saw: 0.0,
            phase: 0.0,
            sign: 0.0,
            hp: 0.0,
            pw: 0.5,
            rng: Rng::vrand(),
            pink: Pink { count: 1, sum: 0.0, rows: [0.0; 5] },
        }
    }

    /// v1's oscillator init `init(mode, f, true)` (ev_6bae725f): random start
    /// phase, state reset, then five settling ticks for saw (0), pulse (1),
    /// parabola (2) and triangle (3); sine (4) only resets; 10 and 11 reset
    /// and re-seed the generator. The init's pending frequency offset is 0
    /// here, so the ticks see `f` (clamped to ±1; the triangle `2f`).
    fn init(&mut self, mode: u32, f: f32) {
        // (bits(s>>9 | 0x40000000) − 2)/2 == white_raw() + ½ exactly.
        let r = self.rng.white_raw() + 0.5;
        self.phase_saw = r;
        self.phase = r;
        let sign = if 0.5 <= r { 1.0 } else { -1.0 };
        self.sign = sign;
        self.slope = sign;
        self.acc_saw = r - 0.5;
        self.acc = sign * 0.5;
        self.ring = [0.0; 4];
        self.ring_pw = [0.5; 4];
        self.ring_idx = 0;
        self.prev = 0.0;
        self.hp = 0.0;
        self.slots = [Slot::default(); 32];
        self.live = 0;
        self.pw = 0.5;
        match mode {
            0 => (0..5).for_each(|_| {
                self.tick_saw(clamp1(f));
            }),
            1 => (0..5).for_each(|_| {
                self.tick_sqr(clamp1(f), true);
            }),
            2 => (0..5).for_each(|_| {
                self.tick_prb(((f.abs() + 1.0) - (f - 1.0).abs()) * 0.5, true);
            }),
            3 => (0..5).for_each(|_| {
                self.tick_tri(clamp1(f + f));
            }),
            10 | 11 => self.rng.reseed(),
            _ => {}
        }
    }

    /// Spawn a step event at `x16` (position × 16).
    fn spawn(&mut self, x16: f32, amp: f32) {
        if self.live < self.slots.len() {
            let i = x16.floor() as i32;
            self.slots[self.live] = Slot { idx: i, frac: x16 - i as f32, amp, ramp: 0.0, integ: 0.0 };
            self.live += 1;
        }
    }

    /// Age step events: those past 126 add their amplitude to `acc` and
    /// leave; the rest add `lerp(step)·amp` to the returned sum and move on
    /// by 16.
    fn age_steps(slots: &mut [Slot; 32], live: &mut usize, acc: &mut f32) -> f32 {
        let step = &bl_tables().step;
        let mut sum = 0.0f32;
        let mut n = 0;
        for i in 0..*live {
            let mut s = slots[i];
            if s.idx > 126 {
                *acc += s.amp;
            } else {
                sum += lerp(step, s.idx, s.frac) * s.amp;
                s.idx = s.idx.wrapping_add(16);
                slots[n] = s;
                n += 1;
            }
        }
        *live = n;
        sum
    }

    /// Saw: delayed increments integrate into `acc_saw`, wraps spawn −1 steps.
    fn tick_saw(&mut self, f: f32) -> f32 {
        let i = self.ring_idx;
        self.acc_saw += self.ring[i];
        self.ring[i] = f;
        self.ring_idx = (i + 1) & 3;
        let p = self.phase_saw + f;
        self.phase_saw = p;
        if p < 1.0 {
            if p < 0.0 {
                self.spawn((p / f) * 16.0, 1.0);
                self.phase_saw += 1.0;
            }
        } else {
            self.phase_saw = p - 1.0;
            self.spawn(((p - 1.0) / f) * 16.0, -1.0);
        }
        let sum = Osc::age_steps(&mut self.slots, &mut self.live, &mut self.acc_saw);
        sum + self.acc_saw
    }

    /// Pulse at width ½ (the init's ticks place events at `p·16·(1/f)`,
    /// the buffer's at `(p/f)·16`).
    fn tick_sqr(&mut self, f: f32, settle: bool) -> f32 {
        let i = self.ring_idx;
        let old = self.ring_pw[i];
        self.ring_pw[i] = 0.5;
        self.ring_idx = (i + 1) & 3;
        let inv = 1.0 / f;
        let pos = |p: f32| if settle { p * 16.0 * inv } else { (p / f) * 16.0 };
        let mut p = self.phase + f;
        self.phase = p;
        let mut th = if self.sign < 0.0 {
            self.pw = 0.5;
            0.5
        } else {
            1.0 - self.pw
        };
        while th <= p {
            p -= th;
            self.phase = p;
            self.sign = -self.sign;
            self.spawn(pos(p), self.sign);
            if self.sign < 0.0 {
                self.pw = 0.5;
                th = 0.5;
            } else {
                th = 1.0 - self.pw;
            }
        }
        while p < 0.0 {
            self.sign = -self.sign;
            self.spawn(pos(p), self.sign);
            p = (self.phase + 1.0) - th;
            self.phase = p;
            if self.sign < 0.0 {
                self.pw = 0.5;
                th = 0.5;
            } else {
                th = 1.0 - self.pw;
            }
        }
        let sum = Osc::age_steps(&mut self.slots, &mut self.live, &mut self.acc);
        self.acc + -0.5 + old + sum
    }

    /// Parabola: the band-limited saw integrated, smoothed by the delayed
    /// increment (×8). The init's ticks have no negative-wrap branch and
    /// weight by the increment itself rather than its magnitude.
    fn tick_prb(&mut self, f: f32, settle: bool) -> f32 {
        let i = self.ring_idx;
        let old = self.ring[i];
        self.acc_saw = old + self.acc_saw;
        self.ring[i] = f;
        self.ring_idx = (i + 1) & 3;
        let p = self.phase_saw + f;
        self.phase_saw = p;
        if 1.0 <= p {
            self.phase_saw = p - 1.0;
            self.spawn(((p - 1.0) / f) * 16.0, -1.0);
        } else if p < 0.0 && !settle {
            self.spawn((p / f) * 16.0, 1.0);
            self.phase_saw += 1.0;
        }
        let sum = Osc::age_steps(&mut self.slots, &mut self.live, &mut self.acc_saw);
        let w = if settle { old } else { old.abs() };
        self.acc = ((sum + self.acc_saw) * 8.0 - self.acc) * w + self.acc;
        self.acc
    }

    /// Triangle at increment `f2`: slope-integrating accumulator, events
    /// carry the integrated band-limited ramp correction, DC-tracking output.
    fn tick_tri(&mut self, f2: f32) -> f32 {
        let i = self.ring_idx;
        let old = self.ring[i];
        self.ring[i] = f2;
        self.ring_idx = (i + 1) & 3;
        self.acc += old * self.slope;
        let p = f2 + self.phase;
        self.phase = p;
        if p < 1.0 {
            if p < 0.0 {
                self.sign = -self.sign;
                self.spawn((p / f2) * 16.0, self.sign + self.sign);
                self.phase += 1.0;
            }
        } else {
            self.phase = p - 1.0;
            self.sign = -self.sign;
            self.spawn(((p - 1.0) / f2) * 16.0, self.sign + self.sign);
        }
        let ramp = &bl_tables().ramp;
        let mut sum = 0.0f32;
        let mut n = 0;
        for k in 0..self.live {
            let mut s = self.slots[k];
            s.integ = (self.prev - old) * s.ramp + s.integ;
            s.ramp = lerp(ramp, s.idx, s.frac) * s.amp;
            if s.idx < 111 {
                sum = sum + s.ramp * old + s.integ;
                s.idx = s.idx.wrapping_add(16);
                self.slots[n] = s;
                n += 1;
            } else {
                self.acc = s.integ + self.acc + s.ramp * old;
                self.slope = -self.slope;
            }
        }
        self.live = n;
        self.prev = old;
        let y = (self.acc - self.hp) + sum;
        self.hp += old * 0.1 * y;
        y
    }

    /// One white value in [−½, ½).
    fn white(&mut self) -> f32 {
        self.rng.white_raw()
    }

    /// v1's pink: row by the count's lowest set bit (0..3, else 4), up to
    /// ten draws for a running sum `(w + sum) − old` within ±2 (kept if none
    /// is), then `(w + sum)·0.2`.
    fn pink(&mut self) -> f32 {
        let c = self.pink.count;
        let k = if c & 1 != 0 {
            0
        } else if c & 2 != 0 {
            1
        } else if c & 4 != 0 {
            2
        } else if c & 8 != 0 {
            3
        } else {
            4
        };
        let old = self.pink.rows[k];
        for _ in 0..10 {
            let w = self.rng.white_raw();
            self.pink.rows[k] = w;
            let t = (w + self.pink.sum) - old;
            if (-2.0..=2.0).contains(&t) {
                self.pink.sum = t;
                break;
            }
        }
        self.pink.count = c.wrapping_add(1);
        (self.rng.white_raw() + self.pink.sum) * 0.2
    }
}

pub struct RingMod {
    ports: Vec<Port>,
    osc: Osc,
    mode: i32,
    fresh: bool,
    f: f32,
    drive: f32,
}

impl RingMod {
    pub fn new() -> RingMod {
        let mut ports = ports_from(ports::RING_MOD);
        // Int-form ctor: Mode 0..6 (Sin, Tri, Saw, Sqr, Prb, White, Pink).
        ports[1] = Port::new(0.0, 0.0, 6.0);
        RingMod { ports, osc: Osc::new(), mode: -1, fresh: true, f: 0.0, drive: 1.0 }
    }
}

impl Default for RingMod {
    fn default() -> Self {
        RingMod::new()
    }
}

impl Node for RingMod {
    fn ports(&mut self) -> &mut Vec<Port> {
        &mut self.ports
    }

    fn process(&mut self, ctx: &Ctx, inp: [&[f32]; 2], out: [&mut [f32]; 2]) {
        let p = |i: usize| self.ports[i].value;
        let d = p(0) * p(0) * p(0);
        let wet = clip01(p(3));
        let dry = 1.0 - wet;
        let f = (1.0 / ctx.sample_rate) * ctx.tune * fast_exp2(p(2));
        let ft = ((f.abs() + 0.5) - (f - 0.5).abs()) * 0.5;
        let m = ((((p(1).abs() + 6.0) - (p(1) - 6.0).abs()) * 0.5) + 0.5) as i32;
        let prev = self.mode;
        self.mode = m;
        if prev != m || self.fresh {
            match m {
                0 => self.osc.init(4, ft),
                1 => self.osc.init(3, ft),
                2 => self.osc.init(0, ft),
                3 => self.osc.init(1, ft),
                4 => self.osc.init(2, ft),
                5 => self.osc.init(10, ft),
                6 => self.osc.init(11, ft),
                _ => {}
            }
        }
        self.fresh = false;
        let n = inp[0].len();
        if n == 0 {
            return;
        }
        let inv_n = 1.0 / n as f32;
        let df = (ft - self.f) * inv_n;
        let dd = inv_n * ((d.abs() + d) * 0.5 - self.drive);
        let tab = sin_tab();
        let [ol, or] = out;
        for i in 0..n {
            self.f += df;
            self.drive += dd;
            let l = sat(self.drive * inp[0][i]);
            let r = sat(self.drive * inp[1][i]);
            let f = self.f;
            let o = &mut self.osc;
            let (a, b) = match m {
                0 => {
                    let mut ph = o.phase_saw + f;
                    if 1.0 <= ph {
                        ph -= 1.0;
                    }
                    o.phase_saw = ph;
                    let x = ph * 2048.0;
                    let k = x.floor() as i32;
                    let j = (1024 + k) as usize;
                    let s = ((tab[j + 1] - tab[j]) * (x - k as f32) + tab[j]) * wet;
                    ((dry + s) * l, (s + dry) * r)
                }
                1 => {
                    let y = o.tick_tri(f + f);
                    ((dry + wet * y) * l, (wet * y + dry) * r)
                }
                2 => {
                    let y = o.tick_saw(f);
                    (wet * l * y + l * dry, wet * r * y + r * dry)
                }
                3 => {
                    let y = o.tick_sqr(f, false);
                    (wet * l * y + l * dry, wet * r * y + r * dry)
                }
                4 => {
                    let y = o.tick_prb(f, false);
                    (wet * l * y + l * dry, wet * r * y + r * dry)
                }
                _ => {
                    let v = f + f + o.phase_saw;
                    o.phase_saw = v;
                    if 1.0 <= v {
                        o.phase_saw = v - v.floor();
                        o.acc_saw = if m == 5 { o.white() } else { o.pink() };
                    }
                    let h = o.acc_saw;
                    ((dry + wet * h) * l, (wet * h + dry) * r)
                }
            };
            ol[i] = a;
            or[i] = b;
        }
    }

    fn reset(&mut self) {
        self.f = 0.0;
        self.drive = 1.0;
        self.mode = -1;
        self.fresh = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_table_ends_at_one() {
        let t = bl_tables();
        assert!((t.step[BL_LEN - 1] - 1.0).abs() < 1e-6);
        assert!(t.step[0].abs() < 1e-3);
        // The ramp keeps its plain running sum at 120 (no −3.5625).
        assert!(t.ramp[120] > 3.0);
    }
}
