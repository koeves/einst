//! v1's slice sound classifier: the drum type the slicer gives each slice.
//!
//! [re:49a32c] (ev_7807cb08) builds one classifier per audio file
//! ([re:b214ab] ev_b49bd4fd, [re:5fcf8f] ev_e8d9dcd4 with the file's maximum
//! slice length, channel count and `(int)` sample rate), hands it each slice
//! in turn ([re:5c6e2d] ev_a4ab9fb6, at most `2·sr` frames of the slice, see
//! [`slice_frames`]) and maps the result with [`slice_type`].
//!
//! Per slice ([re:5c6e2d]):
//!
//! 1. [re:6a0cb2] (ev_4dd4b4f2): the signal is the left channel, or
//!    `(l + r)·0.5` for two channels, peak normalised (abs max, ×1/max,
//!    [re:903198] ev_8cba0c8e); only its first 88 200 samples feed the
//!    envelope (the constructor's `(int)(44100·2.0)`, which
//!    [re:5fcf8f] does not redo for the real sample rate).
//! 2. [re:cbdeef] (ev_f6f96b28): a squared-signal envelope follower
//!    (attack `e^(−1000/(3·sr))`, release `e^(−1000/(30·sr))`); its peak in the
//!    first 100 ms; attack start / end = first envelope sample ≥ 0.1 / 0.95 of
//!    the peak ([re:d37370] ev_b48637fa); the end = first sample after the
//!    attack end below −50 dB of the peak ([re:457ef4] ev_f8644a15,
//!    [re:bd4ad4] ev_88147e06), or the slice length. [re:964468]
//!    (ev_cc39b3a9) zeroes the signal from there on.
//! 3. [re:c6e8b3] (ev_2e119526): a 2048-point STFT, hop 512, symmetric
//!    Hann window ([re:09b9a0] ev_0b0472b4), `ceil((2048 + end)/512) − 1`
//!    frames, the first window ending at sample 511; each frame's 1025
//!    magnitudes ([re:27687e] ev_54c600da) and their maximum.
//! 4. [re:7578b3] (ev_f29a7229), in this order: spectral centroid, spread
//!    variance, skew, flatness (each a per-frame value averaged with the
//!    frame maxima as weights, [re:abfec8] ev_3d6e0f59), 40 relative mel band
//!    energies, `log10((attack end − attack start)/sr)` and the mean zero
//!    crossing count per 2048-sample block; then the rule list
//!    ([re:be0457], `RULES`) names a sound and [re:5876c1] (ev_cfdf1766)
//!    numbers it ([`UNKNOWN`] .. [`TOM`]).
//!
//! The classifier keeps state between the slices of a file and v1 depends on
//! it (see [`Classifier`]); [`classify`] runs one slice on a fresh one.
//!
//! Deviations: v1's FFT is a library single-precision real FFT; here a
//! radix-2 FFT in f64 rounded to f32, so magnitudes can differ in the last
//! bits. `log10f` / `powf` / `expf` / `logf` come from the host's libm. The
//! f32-vs-f64 evaluation of the envelope coefficients and the mel point
//! conversions follows the decompiled types (not checked instruction by
//! instruction). Everything else keeps v1's f32 operation order (sums left to
//! right, `powf` where v1 calls `powf`), so features agree to rounding, and
//! the rule list (its own f64 / f32 comparisons, exact constants) is exact.

/// [re:5876c1]'s sound numbers ([`Classifier::process`]'s result).
pub const UNKNOWN: u8 = 0;
pub const KICK: u8 = 1;
pub const KICK_CYMBAL: u8 = 2;
pub const KICK_HAT: u8 = 3;
pub const SNARE: u8 = 4;
pub const SNARE_HAT: u8 = 5;
pub const HIHAT: u8 = 6;
pub const CLAP: u8 = 7;
pub const CRASH: u8 = 8;
pub const CYMBAL: u8 = 9;
pub const PERC: u8 = 10;
pub const TOM: u8 = 11;

/// The names [re:5876c1] compares against, by sound number.
pub const SOUND_NAMES: [&str; 12] = ["Unknown", "Kick", "KickCymbal", "KickHat", "Snare", "SnareHat", "Hihat", "Clap", "Crash", "Cymbal", "Perc", "Tom"];

/// Mel bands ([re:80bc0a]: 40 filters, 10 Hz .. 15 kHz).
pub const BANDS: usize = 40;
const FFT: usize = 2048;
const HOP: usize = 512;
const BINS: usize = FFT / 2 + 1;
/// The constructor's signal limit `(int)(44100 · 2.0)` ([re:b214ab]).
const MAX_SIGNAL: usize = 88200;
/// The constructor's dB value; thresholds are `10^(−50/20)` ([re:bd4ad4]).
const DB: f32 = 50.0;

/// v1's mapping of the classifier result to the slice type ([re:49a32c]):
/// 0 none, 1 Kick, 2 Snare, 3 Hihat, 4 Percussion.
pub fn slice_type(raw: u8) -> u8 {
    match raw {
        KICK | KICK_CYMBAL | KICK_HAT => 1,
        SNARE | SNARE_HAT | CLAP => 2,
        HIHAT | CRASH | CYMBAL => 3,
        PERC | TOM => 4,
        _ => 0,
    }
}

/// The frames of a slice v1 classifies: `min(len, 2·(int)sr)` ([re:49a32c]).
pub fn slice_frames(len: usize, sample_rate: f32) -> usize {
    len.min((sample_rate as i32).max(0) as usize * 2)
}

/// One slice on a fresh classifier, as v1 classifies a file's first slice.
/// `left` / `right` hold the frames to classify (see [`slice_frames`]);
/// `max_len` is the file's longest slice, which v1 passes to its
/// initialisation but never uses. Returns the raw sound number (0..=11).
pub fn classify(left: &[f32], right: Option<&[f32]>, sample_rate: f32, max_len: usize) -> u8 {
    Classifier::new(max_len, if right.is_some() { 2 } else { 1 }, sample_rate).process(left, right)
}

/// The features [re:7578b3] feeds the rules, kept for inspection.
#[derive(Clone, Debug, PartialEq)]
pub struct Features {
    pub zero_crossings: f32,
    pub log_attack: f32,
    pub centroid: f32,
    pub spread_variance: f32,
    pub skew: f32,
    pub flatness: f32,
    pub bands: [f32; BANDS],
    pub attack_start: f32,
    pub attack_end: f32,
    pub end: f32,
    pub frames: usize,
}

impl Default for Features {
    fn default() -> Self {
        Features { zero_crossings: 0.0, log_attack: 0.0, centroid: 0.0, spread_variance: 0.0, skew: 0.0, flatness: 0.0, bands: [0.0; BANDS], attack_start: 0.0, attack_end: 0.0, end: 0.0, frames: 0 }
    }
}

/// v1's float vector where it matters: growing reallocates zeroed,
/// shrinking keeps the contents ([re:029f02] ev_acb3baba).
#[derive(Clone, Debug, Default)]
struct Vector {
    buf: Vec<f32>,
    len: usize,
}

impl Vector {
    fn set_len(&mut self, n: usize) {
        if n > self.len {
            self.buf = vec![0.0; n];
        }
        self.len = n;
    }

    fn get(&self) -> &[f32] {
        &self.buf[..self.len]
    }

    fn get_mut(&mut self) -> &mut [f32] {
        &mut self.buf[..self.len]
    }
}

/// v1 vector sum: left to right (also [re:d9d1c0]'s and
/// [re:991506]'s loops).
fn sum(x: &[f32]) -> f32 {
    let mut s = 0.0f32;
    for (i, &v) in x.iter().enumerate() {
        s = if i == 0 { v } else { s + v };
    }
    s
}

/// v1 vector max: the running `if v <= m { m } else { v }`, 0 if empty.
fn max(x: &[f32]) -> f32 {
    let Some(&first) = x.first() else { return 0.0 };
    x.iter().fold(first, |m, &v| if v <= m { m } else { v })
}

/// v1 vector min.
fn min(x: &[f32]) -> f32 {
    let Some(&first) = x.first() else { return 0.0 };
    x.iter().fold(first, |m, &v| if m <= v { m } else { v })
}

/// [re:abfec8]: `Σ x·w / Σ w`, 0 if `x` is empty or `Σ w` is 0.
fn weighted_mean(x: &[f32], w: &[f32]) -> f32 {
    if x.is_empty() {
        return 0.0;
    }
    let ws = sum(w);
    if ws == 0.0 {
        return 0.0;
    }
    let p: Vec<f32> = x.iter().zip(w).map(|(&a, &b)| a * b).collect();
    sum(&p) / ws
}

/// [re:d37370] with `false`: the first index with `v <= x[i]`, −1 if none.
fn arg_value(x: &[f32], v: f32) -> i32 {
    x.iter().position(|&e| v <= e).map_or(-1, |i| i as i32)
}

/// [re:457ef4]: the first index from `start` with `x[i] < v`, −1 if none.
fn arg_min_value(x: &[f32], v: f32, start: u32) -> i32 {
    x.iter().enumerate().skip(start as usize).find(|&(_, &e)| e < v).map_or(-1, |(i, _)| i as i32)
}

/// [re:38e0ec]: 0 for ±0, 1 for ≥ 0, else −1 (NaN too).
fn sign_of(x: f32) -> i32 {
    if x == 0.0 {
        0
    } else if 0.0 <= x {
        1
    } else {
        -1
    }
}

/// [re:bd4ad4]: `10^(x/−20)`.
fn from_db(x: f32) -> f32 {
    10f32.powf(x / -20.0)
}

/// The mel filter bank of [re:80bc0a] / [re:8f1413] (ev_9455558b,
/// ev_1f7f3373): triangles between neighbouring mel points, `2/(upper −
/// lower)` high.
#[derive(Clone, Debug)]
struct MelFilters {
    lower: Vec<f32>,
    upper: Vec<f32>,
    weights: Vec<Vec<f32>>,
    /// `((bins − 1) + (bins − 1)) · (1/sr)`: bins per Hz.
    scale: f32,
}

impl MelFilters {
    fn new(filters: usize, min_hz: i32, max_hz: i32, bins: usize, sr: i32, freqs: &[f32]) -> MelFilters {
        let inv_sr = 1.0 / sr as f32;
        let mel: Vec<f32> = (0..(max_hz - min_hz).max(0) as u32).map(|i| ((i as f32 + min_hz as f32) * 0.0014285714 + 1.0).log10() * 2595.0).collect();
        let (top, bottom) = (max(&mel), min(&mel));
        let step = (top - bottom) / (filters + 1) as f32;
        let hz: Vec<f32> = (0..filters + 2).map(|i| (10f32.powf(((i as f32 + 0.0) * step + bottom) * 0.00038535646) - 1.0) * 700.0).collect();
        let lower = hz[..filters].to_vec();
        let center = &hz[1..filters + 1];
        let upper = hz[2..filters + 2].to_vec();
        let scale = ((bins as f32 - 1.0) + (bins as f32 - 1.0)) * inv_sr;
        let weights = (0..filters)
            .map(|i| {
                let mut w = vec![0.0f32; bins];
                let height = 2.0 / (upper[i] - lower[i]);
                let lo = (scale * lower[i]).ceil() as u32 as i32;
                let hi = (scale * upper[i]).ceil() as i32;
                let mid = (scale * center[i]).ceil() as u32 as i32;
                if hi < bins as i32 {
                    for j in lo.max(0)..mid {
                        let j = j as usize;
                        w[j] = ((freqs[j] - lower[i]) * height) / (center[i] - lower[i]);
                    }
                    for j in mid.max(0)..hi {
                        let j = j as usize;
                        w[j] = ((upper[i] - freqs[j]) * height) / (upper[i] - center[i]);
                    }
                }
                w
            })
            .collect();
        MelFilters { lower, upper, weights, scale }
    }

    /// [re:9dca71] (ev_3ec4b163): each filter's `Σ w·|X|²` over
    /// `ceil(scale·lower) .. min(ceil(scale·upper), bins − 1)`.
    fn filter(&self, mag: &[f32], out: &mut [f32; BANDS]) {
        let power: Vec<f32> = mag.iter().map(|&m| m * m).collect();
        for (i, w) in self.weights.iter().enumerate() {
            let lo = (self.scale * self.lower[i]).ceil() as u32 as i32;
            let mut hi = (self.scale * self.upper[i]).ceil() as i32;
            if w.len() as i32 <= hi {
                hi = w.len() as i32 - 1;
            }
            let mut s = 0.0f32;
            for j in lo.max(0)..hi {
                s = s + w[j as usize] * power[j as usize];
            }
            out[i] = s;
        }
    }
}

/// An in-place radix-2 FFT over `re` / `im` (f64).
fn fft(re: &mut [f64], im: &mut [f64]) {
    let n = re.len();
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let mut len = 2;
    while len <= n {
        let ang = -2.0 * std::f64::consts::PI / len as f64;
        for k in 0..len / 2 {
            let (s, c) = (ang * k as f64).sin_cos();
            let mut i = k;
            while i < n {
                let a = i + len / 2;
                let (tr, ti) = (re[a] * c - im[a] * s, re[a] * s + im[a] * c);
                re[a] = re[i] - tr;
                im[a] = im[i] - ti;
                re[i] += tr;
                im[i] += ti;
                i += len;
            }
        }
        len <<= 1;
    }
}

/// v1's per-file classifier ([re:b214ab] / [re:5fcf8f]).
///
/// v1 reuses one across a file's slices and two values survive between
/// them: [re:3a4d53] (ev_237df10a) and [re:83761a] (ev_be9b751b) only
/// recompute the per-frame spectral spreads while their mean is still 0 —
/// in practice for the first slice. Later slices take the variance and the
/// skew's divisor from the previous slice's spreads where the frame count
/// did not grow, and from zeros (skew ±inf / NaN) where it did
/// ([re:029f02]). [`Classifier::process`] reproduces that when called slice
/// after slice.
#[derive(Clone, Debug)]
pub struct Classifier {
    sr: i32,
    channels: usize,
    /// Bin frequencies `i · sr/2048`.
    freqs: Vec<f32>,
    window: Vec<f32>,
    mel: MelFilters,
    /// Per-frame magnitudes, `ceil((88200 + 2048)/512) − 1` of them.
    spectra: Vec<Vec<f32>>,
    frame_max: Vector,
    centroids: Vector,
    spreads: Vector,
    skews: Vector,
    centroid_mean: f32,
    spread_mean: f32,
    signal: Vec<f32>,
    /// `min(slice length, 88200)`: the samples the envelope follows.
    used: usize,
    attack_start: f32,
    attack_end: f32,
    end: f32,
    frames: usize,
    features: Features,
}

impl Classifier {
    /// [re:b214ab] then [re:5fcf8f]`(max_len, channels, (int)sample_rate)`;
    /// `max_len` is unused as in v1.
    pub fn new(max_len: usize, channels: usize, sample_rate: f32) -> Classifier {
        let _ = max_len;
        let sr = sample_rate as i32;
        // [re:09b9a0]: 0.5 − 0.5·cos(2πi/(n − 1)) in f64.
        let window = (0..FFT).map(|i| ((i as f64 * 6.283185307179586 / (FFT - 1) as f64).cos() * -0.5 + 0.5) as f32).collect();
        let total = ((MAX_SIGNAL + FFT) as u32 as f32 / HOP as f32).ceil() as i32 - 1;
        let step = sr as f32 / ((BINS as f32 - 1.0) + (BINS as f32 - 1.0));
        let freqs: Vec<f32> = (0..BINS).map(|i| (i as f32 + 0.0) * step).collect();
        let mel = MelFilters::new(BANDS, 10, 15000, BINS, sr, &freqs);
        Classifier {
            sr,
            channels,
            freqs,
            window,
            mel,
            spectra: vec![vec![0.0; BINS]; total.max(0) as usize],
            frame_max: Vector::default(),
            centroids: Vector::default(),
            spreads: Vector::default(),
            skews: Vector::default(),
            centroid_mean: 0.0,
            spread_mean: 0.0,
            signal: Vec::new(),
            used: 0,
            attack_start: 0.0,
            attack_end: 0.0,
            end: 0.0,
            frames: 0,
            features: Features::default(),
        }
    }

    /// The last slice's features.
    pub fn features(&self) -> &Features {
        &self.features
    }

    /// [re:5c6e2d]: classify `left.len()` frames (`right` for a two-channel
    /// classifier; missing, the left channel stands in). 0 for an empty slice
    /// or a channel count other than 1 or 2.
    pub fn process(&mut self, left: &[f32], right: Option<&[f32]>) -> u8 {
        let n = left.len();
        if n < 1 || !(1..=2).contains(&self.channels) {
            return UNKNOWN;
        }
        self.setup_signal(left, right);
        self.attack_start = 0.0;
        self.attack_end = 0.0;
        self.find_attack_times();
        let frames = (((FFT as f32 + self.end) / HOP as f32).ceil() - 1.0) as i32;
        if frames < 0 || frames as usize > self.spectra.len() {
            // v1 throws out_of_range in CalcSTFT here; the caller's slice
            // limit keeps it from happening.
            return UNKNOWN;
        }
        let frames = frames as usize;
        self.frames = frames;
        self.centroids.set_len(frames);
        self.spreads.set_len(frames);
        self.skews.set_len(frames);
        self.frame_max.set_len(frames);
        self.adjust_end(n);
        self.stft();
        self.classify()
    }

    /// [re:6a0cb2].
    fn setup_signal(&mut self, left: &[f32], right: Option<&[f32]>) {
        let n = left.len();
        self.signal.clear();
        if self.channels == 1 {
            self.signal.extend_from_slice(left);
        } else {
            self.signal.extend((0..n).map(|i| (left[i] + right.map_or(left[i], |r| r[i])) * 0.5));
        }
        self.used = n.min(MAX_SIGNAL);
        // [re:903198]: the running max of |x| from 0, then ×(1/max).
        let mut m = 0.0f32;
        for &v in &self.signal {
            let a = if 0.0 < v { v } else { -v };
            m = if a <= m { m } else { a };
        }
        if m != 0.0 {
            let k = 1.0 / m;
            for v in &mut self.signal {
                *v = *v * k;
            }
        }
    }

    /// [re:cbdeef].
    fn find_attack_times(&mut self) {
        let n = self.signal.len();
        let a = (-1000.0f32 / (self.sr as f32 * 3.0)).exp();
        let r = (-1000.0f32 / (self.sr as f32 * 30.0)).exp();
        let mut env = vec![0.0f32; n];
        let mut e = 0.0f32;
        for i in 0..self.used {
            let x = self.signal[i] * self.signal[i];
            e = if e < x { e * a + x * (1.0 - a) } else { e * r + x * (1.0 - r) };
            env[i] = e;
        }
        let mut head = self.sr as f64 * 0.1;
        if (self.used as f64) < head {
            head = self.used as f64;
        }
        let peak = max(&env[..(head as i32).max(0) as usize]);
        let mut attack_end = arg_value(&env, peak * 0.95) as f32;
        if attack_end < 1.0 {
            attack_end = 1.0;
        }
        self.attack_end = attack_end;
        self.attack_start = arg_value(&env, peak * 0.1) as f32;
        let end = arg_min_value(&env, from_db(DB) * peak, attack_end as i32 as u32);
        self.end = if end == -1 { n as f32 } else { end as f32 };
    }

    /// [re:964468].
    fn adjust_end(&mut self, n: usize) {
        if self.end < n as f32 && ((self.end as u32 as i32) < n as i32) {
            for v in &mut self.signal[self.end as usize..] {
                *v = 0.0;
            }
        }
    }

    /// [re:c6e8b3]. The frame buffer is zeroed once; samples before the
    /// signal's start keep the previous frame's (windowed) values, as in v1.
    fn stft(&mut self) {
        let mut buf = [0.0f32; FFT];
        let mut re = vec![0.0f64; FFT];
        let mut im = vec![0.0f64; FFT];
        for f in 0..self.frames {
            let start = ((f * HOP) as u32).wrapping_add(HOP as u32).wrapping_sub(FFT as u32);
            for (j, b) in buf.iter_mut().enumerate() {
                let idx = start.wrapping_add(j as u32) as i32;
                if idx >= 0 {
                    *b = if (idx as f32) < self.end { self.signal[idx as usize] } else { 0.0 };
                }
            }
            for (b, w) in buf.iter_mut().zip(&self.window) {
                *b = *b * *w;
            }
            for j in 0..FFT {
                re[j] = buf[j] as f64;
                im[j] = 0.0;
            }
            fft(&mut re, &mut im);
            let spec = &mut self.spectra[f];
            for k in 0..BINS {
                let (x, y) = (re[k] as f32, im[k] as f32);
                // [re:27687e]
                spec[k] = if x == 0.0 && y == 0.0 { 0.0 } else { (x * x + y * y).sqrt() };
            }
            self.frame_max.get_mut()[f] = max(spec);
        }
    }

    /// [re:7578b3].
    fn classify(&mut self) -> u8 {
        let centroid = self.spec_centroid();
        let spread_variance = self.spec_spread_variance();
        let skew = self.spec_skew();
        let flatness = self.spec_flatness();
        let bands = self.relative_energy_bands();
        let log_attack = ((self.attack_end - self.attack_start) / self.sr as f32).log10();
        let zero_crossings = self.mean_zero_crossings();
        self.features =
            Features { zero_crossings, log_attack, centroid, spread_variance, skew, flatness, bands, attack_start: self.attack_start, attack_end: self.attack_end, end: self.end, frames: self.frames };
        matches_rule(&self.features)
    }

    /// The frames before the first silent one (v1's per-frame loops stop
    /// there; the later frames keep their old values but still weigh in).
    fn live_frames(&self) -> usize {
        self.frame_max.get().iter().position(|&m| m == 0.0).unwrap_or(self.frames)
    }

    /// [re:7e078e] (ev_3d171f73) with [re:7e9f66] (ev_6896cebc).
    fn spec_centroid(&mut self) -> f32 {
        for i in 0..self.live_frames() {
            let mag = &self.spectra[i];
            let s = sum(mag);
            self.centroids.get_mut()[i] = if s != 0.0 {
                let wf: Vec<f32> = self.freqs.iter().zip(mag).map(|(&f, &m)| f * m).collect();
                sum(&wf) / s
            } else {
                0.0
            };
        }
        self.centroid_mean = weighted_mean(self.centroids.get(), self.frame_max.get());
        self.centroid_mean
    }

    /// [re:e09766] (ev_8325f873) with [re:6b6c58] (ev_196c8f12).
    fn spec_spread(&mut self) {
        if self.centroid_mean == 0.0 {
            self.spec_centroid();
        }
        for i in 0..self.live_frames() {
            let mag = &self.spectra[i];
            let c = self.centroids.get()[i];
            let s = sum(mag);
            self.spreads.get_mut()[i] = if s != 0.0 {
                let d: Vec<f32> = self.freqs.iter().zip(mag).map(|(&f, &m)| ((f - c) * (f - c)) * m).collect();
                (sum(&d) / s).powf(0.5)
            } else {
                0.0
            };
        }
        self.spread_mean = weighted_mean(self.spreads.get(), self.frame_max.get());
    }

    /// [re:3a4d53]: the population variance of the per-frame spreads
    /// ([re:991506] ev_0664a9db).
    fn spec_spread_variance(&mut self) -> f32 {
        if self.spread_mean == 0.0 {
            self.spec_spread();
        }
        let x = self.spreads.get();
        let n = x.len() as f32;
        let mean = sum(x) / n;
        let mut acc = 0.0f32;
        for &v in x {
            acc = (v - mean) * (v - mean) + acc;
        }
        acc / n
    }

    /// [re:83761a] with [re:8d26c7] (ev_319ef2e8).
    fn spec_skew(&mut self) -> f32 {
        if self.centroid_mean == 0.0 {
            self.spec_centroid();
        }
        if self.spread_mean == 0.0 {
            self.spec_spread();
        }
        for i in 0..self.live_frames() {
            let mag = &self.spectra[i];
            let c = self.centroids.get()[i];
            let spread = self.spreads.get()[i];
            let s = sum(mag);
            self.skews.get_mut()[i] = if s != 0.0 {
                let d: Vec<f32> = self.freqs.iter().zip(mag).map(|(&f, &m)| (f - c).powf(3.0) * m).collect();
                (sum(&d) / s) / spread.powf(3.0)
            } else {
                0.0
            };
        }
        weighted_mean(self.skews.get(), self.frame_max.get())
    }

    /// [re:4b2ab6] (ev_8cd95520) with [re:065f79] (ev_ebac0b6e): geometric
    /// over arithmetic mean ([re:d9d1c0] ev_0e444915).
    fn spec_flatness(&self) -> f32 {
        let mut flat = vec![0.0f32; self.frames];
        for (i, v) in flat.iter_mut().enumerate().take(self.live_frames()) {
            let x = &self.spectra[i];
            let n = x.len() as f32;
            let mean = sum(x) / n;
            let mut logs = 0.0f32;
            for &e in x {
                logs = e.ln() + logs;
            }
            let gmean = (logs / n).exp();
            *v = if mean != 0.0 { gmean / mean } else { 0.0 };
        }
        weighted_mean(&flat, self.frame_max.get())
    }

    /// [re:584d52] (ev_8b169f4f): each band's share of the energy summed
    /// over all frames.
    fn relative_energy_bands(&self) -> [f32; BANDS] {
        let mut acc = [0.0f32; BANDS];
        let mut band = [0.0f32; BANDS];
        let mut total = 0.0f32;
        for f in 0..self.frames {
            self.mel.filter(&self.spectra[f], &mut band);
            let s = sum(&band);
            for (a, &b) in acc.iter_mut().zip(&band) {
                *a = b + *a;
            }
            total = total + s;
        }
        let mut out = [0.0f32; BANDS];
        if 0.0 < total {
            for (o, &a) in out.iter_mut().zip(&acc) {
                *o = a / total;
            }
        }
        out
    }

    /// [re:4c9e03] (ev_e1fd3a41): sign changes per 2048-sample block up to
    /// the end, stopping at the first block at or below −50 dB mean power.
    fn mean_zero_crossings(&self) -> f32 {
        let end = self.end as i32;
        let floor = from_db(DB);
        let mut buf = [0.0f32; FFT];
        let mut counts = Vec::new();
        let mut start = 0i32;
        while start < end {
            for (j, b) in buf.iter_mut().enumerate() {
                let idx = start + j as i32;
                *b = if idx < end { self.signal[idx as usize] } else { 0.0 };
            }
            let sq: Vec<f32> = buf.iter().map(|&v| v * v).collect();
            if sum(&sq) / FFT as f32 <= floor {
                break;
            }
            let mut c = 0.0f32;
            for j in 0..FFT - 1 {
                c = ((sign_of(buf[j + 1]) as f32 - sign_of(buf[j]) as f32) * 0.5).abs() + c;
            }
            counts.push(c);
            start += FFT as i32;
        }
        let mut s = 0.0f32;
        for &c in &counts {
            s = s + c;
        }
        s / counts.len() as f32
    }
}

// The rule list ([re:be0457], ev_0ccbaae6): an ordered list of conjunctions
// over the features, compiled by v1's compiler into a decision graph of
// float and double comparisons. Each row is one comparison as v1's machine
// code makes it (`x` widened to f64 against the exact constant; f32
// comparisons are widened too, which keeps their result) and the two
// successors. Generated from the procedure's instructions with the
// constants read from its pools; a VM over the instructions agreed with the
// table on 20 000 random feature sets.

/// Feature indices: 0..40 the mel bands, then these.
const ZCR: u8 = 40;
const LOG_ATTACK: u8 = 41;
const CENTROID: u8 = 42;
const SPREAD_VAR: u8 = 43;
const SKEW: u8 = 44;
const FLATNESS: u8 = 45;

/// Comparison outcomes and the sets a row jumps on.
const LT: u8 = 1;
const EQ: u8 = 2;
const IS_GT: u8 = 4;
const UNORD: u8 = 8;
/// `x > k` (ja).
const GT: u8 = IS_GT;
/// `x <= k`, both ordered.
const LE: u8 = LT | EQ;
/// `!(x > k)` (jbe).
const NGT: u8 = LT | EQ | UNORD;
/// `!(x <= k)` (ja + jp).
const NLE: u8 = IS_GT | UNORD;
/// `x` is NaN (jp).
const NAN: u8 = UNORD;

/// A successor at or above `L` is a result: `L | sound`.
const L: u16 = 0x8000;
const KICK_L: u16 = KICK as u16;
const KICK_HAT_L: u16 = KICK_HAT as u16;
const SNARE_L: u16 = SNARE as u16;
const SNARE_HAT_L: u16 = SNARE_HAT as u16;
const HIHAT_L: u16 = HIHAT as u16;
const CLAP_L: u16 = CLAP as u16;
const CYMBAL_L: u16 = CYMBAL as u16;
const PERC_L: u16 = PERC as u16;
const TOM_L: u16 = TOM as u16;

/// One comparison: feature, constant, jump set, successor if the outcome is
/// in the set, successor otherwise.
#[derive(Clone, Copy, Debug)]
struct Rule {
    x: u8,
    k: f64,
    jump: u8,
    yes: u16,
    no: u16,
}

const fn r(x: u8, k: f64, jump: u8, yes: u16, no: u16) -> Rule {
    Rule { x, k, jump, yes, no }
}

/// [re:be0457] over the features; the sound number of the matching rule.
fn matches_rule(f: &Features) -> u8 {
    let value = |x: u8| -> f32 {
        match x {
            ZCR => f.zero_crossings,
            LOG_ATTACK => f.log_attack,
            CENTROID => f.centroid,
            SPREAD_VAR => f.spread_variance,
            SKEW => f.skew,
            FLATNESS => f.flatness,
            b => f.bands[b as usize],
        }
    };
    let mut i = 0usize;
    loop {
        let rule = RULES[i];
        let v = value(rule.x) as f64;
        let outcome = if v.is_nan() {
            UNORD
        } else if v < rule.k {
            LT
        } else if v == rule.k {
            EQ
        } else {
            IS_GT
        };
        let next = if rule.jump & outcome != 0 { rule.yes } else { rule.no };
        if next >= L {
            return (next - L) as u8;
        }
        i = next as usize;
    }
}

const B0: u8 = 0;
const B1: u8 = 1;
const B2: u8 = 2;
const B3: u8 = 3;
const B4: u8 = 4;
const B8: u8 = 8;
const B10: u8 = 10;
const B13: u8 = 13;
const B27: u8 = 27;
const B29: u8 = 29;
const B33: u8 = 33;
const B35: u8 = 35;
const B39: u8 = 39;

/// [re:be0457]'s decision graph, entry at row 0.
#[rustfmt::skip]
const RULES: [Rule; 965] = [
    r(B0, 0.134251, NGT, 1, 2), // 0
    r(B33, 0.006321, NGT, 3, 4), // 1
    r(B33, 0.00022, NLE, 5, 6), // 2
    r(B35, 0.005472, NGT, 7, 8), // 3
    r(LOG_ATTACK, -1.571454, NGT, 3, 9), // 4
    r(B33, 0.006321, NGT, 3, 4), // 5
    r(CENTROID, 1505.942993, NLE, 5, 10), // 6
    r(B0, 0.134251, NGT, 11, 12), // 7
    r(LOG_ATTACK, -1.782307, NLE, 7, 13), // 8
    r(B4, 0.02774, NLE, 3, 14), // 9
    r(B1, 0.421437, NLE, 5, 15), // 10
    r(B0, 0.138877, NGT, 16, 17), // 11
    r(B33, 0.00022, NLE, 18, 19), // 12
    r(CENTROID, 7830.331543, NGT, 7, 20), // 13
    r(B29, 0.015609, NGT, 3, 21), // 14
    r(B13, 0.000611, NLE, 5, 22), // 15
    r(ZCR, 115.5, NGT, 23, 24), // 16
    r(B33, 0.00022, NLE, 16, 25), // 17
    r(B39, 0.000121, NLE, 11, 26), // 18
    r(CENTROID, 1619.942749, NAN, 27, 28), // 19
    r(B8, 0.039225, NLE, 7, 29), // 20
    r(ZCR, 745.5, NLE, 3, 30), // 21
    r(CENTROID, 618.451965, NLE, 5, 31), // 22
    r(ZCR, 108.75, NGT, 32, 33), // 23
    r(B35, 0.005472, NGT, 23, 34), // 24
    r(CENTROID, 2399.848633, NLE, 16, 35), // 25
    r(CENTROID, 2399.848633, NLE, 11, 36), // 26
    r(B1, 0.602448, NGT, 37, 38), // 27
    r(CENTROID, 1619.942749, NGT, 39, 27), // 28
    r(B29, 0.022563, NGT, 7, 40), // 29
    r(LOG_ATTACK, -1.488192, NGT, 3, L | CYMBAL_L), // 30
    r(FLATNESS, 0.037943, NLE, 5, 41), // 31
    r(ZCR, 165.333328, NLE, 42, 43), // 32
    r(B35, 0.005472, GT, 44, 45), // 33
    r(B13, 0.026448, NLE, 23, 46), // 34
    r(B13, 0.001917, NLE, 16, 47), // 35
    r(B13, 0.001917, NGT, 48, 49), // 36
    r(CENTROID, 1619.942749, NLE, 50, 51), // 37
    r(B33, 3.3e-05, NLE, 37, 52), // 38
    r(B10, 0.000809, NLE, 27, 53), // 39
    r(B3, 0.000315, NGT, 7, L | HIHAT_L), // 40
    r(B10, 0.000397, NLE, 5, 54), // 41
    r(B0, 0.138877, NGT, 55, 56), // 42
    r(B0, 0.081401, NLE, 57, 58), // 43
    r(B2, 0.015744, NGT, 59, 60), // 44
    r(B2, 0.050504, NGT, 32, 61), // 45
    r(B8, 0.096187, NLE, 23, 62), // 46
    r(CENTROID, 726.625122, NLE, 63, 64), // 47
    r(B13, 0.001917, NAN, 11, 65), // 48
    r(B0, 0.138877, NGT, 16, 17), // 49
    r(CENTROID, 1848.422852, NLE, 18, 66), // 50
    r(B10, 0.000809, NLE, 50, 67), // 51
    r(B33, 1.4e-05, NGT, 68, 69), // 52
    r(B1, 0.547838, NLE, 70, 71), // 53
    r(ZCR, 7.785714, NLE, 5, L | KICK_L), // 54
    r(ZCR, 165.333328, NLE, 72, 73), // 55
    r(B33, 0.00022, NLE, 55, 74), // 56
    r(B0, 0.083773, NLE, 42, 75), // 57
    r(SKEW, 2.637241, NGT, 57, 76), // 58
    r(B2, 0.050504, NGT, 32, 61), // 59
    r(B1, 0.005823, NGT, 59, 77), // 60
    r(B0, 0.064745, NLE, 32, 78), // 61
    r(B2, 0.01592, NLE, 23, 79), // 62
    r(B13, 0.001917, NLE, 16, 80), // 63
    r(B1, 0.429172, NLE, 63, 81), // 64
    r(B35, 4.8e-05, NGT, 11, 82), // 65
    r(B3, 0.098741, NLE, 18, 83), // 66
    r(CENTROID, 700.250549, NGT, 50, 84), // 67
    r(B33, 1.4e-05, NAN, 37, 85), // 68
    r(CENTROID, 1619.942749, NLE, 50, 51), // 69
    r(B1, 0.602448, NGT, 37, 38), // 70
    r(CENTROID, 726.625122, NLE, 70, 86), // 71
    r(ZCR, 74.666664, NLE, 87, 88), // 72
    r(B0, 0.083773, NLE, 72, 89), // 73
    r(CENTROID, 2399.848633, NLE, 55, 90), // 74
    r(SKEW, 2.637241, NGT, 42, 91), // 75
    r(B1, 0.195292, NLE, 57, 92), // 76
    r(B1, 0.161789, NLE, 59, 93), // 77
    r(B27, 0.035614, NLE, 32, 94), // 78
    r(SPREAD_VAR, 189276.953125, NGT, 23, 95), // 79
    r(CENTROID, 537.577698, NGT, 96, 97), // 80
    r(SKEW, 7.215701, NLE, 63, 98), // 81
    r(B39, 4.5e-05, NLE, 11, 99), // 82
    r(B4, 0.045118, NGT, 18, 100), // 83
    r(B1, 0.472061, NLE, 50, 101), // 84
    r(B27, 5.1e-05, NLE, 37, 102), // 85
    r(B1, 0.385699, NLE, 70, 103), // 86
    r(ZCR, 70.0, NLE, 104, 105), // 87
    r(B0, 0.163773, NLE, 87, 106), // 88
    r(ZCR, 37.833332, NLE, 107, 108), // 89
    r(B3, 0.083475, NLE, 55, 109), // 90
    r(B1, 0.195292, NLE, 42, 110), // 91
    r(B13, 0.060828, NLE, 57, 111), // 92
    r(B8, 0.005369, NGT, 59, 112), // 93
    r(B39, 0.002043, NLE, 32, 113), // 94
    r(B33, 0.012563, NGT, 23, 114), // 95
    r(CENTROID, 537.577698, NAN, 16, 115), // 96
    r(ZCR, 115.5, NGT, 23, 24), // 97
    r(LOG_ATTACK, -1.431718, NLE, 63, L | KICK_L), // 98
    r(B8, 0.000681, NLE, 11, L | KICK_HAT_L), // 99
    r(B2, 0.110901, NGT, 116, 117), // 100
    r(B8, 0.000889, NLE, 50, 118), // 101
    r(SPREAD_VAR, 791670.125, NGT, 37, L | TOM_L), // 102
    r(B8, 0.000363, NLE, 70, 119), // 103
    r(ZCR, 165.333328, NLE, 120, 121), // 104
    r(B0, 0.163773, NLE, 104, 122), // 105
    r(SKEW, 3.328605, NGT, 87, 123), // 106
    r(B2, 0.026483, NLE, 72, 124), // 107
    r(B1, 0.308665, NLE, 107, 125), // 108
    r(CENTROID, 557.863342, NLE, 126, 127), // 109
    r(B13, 0.049504, GT, 128, 129), // 110
    r(ZCR, 27.4, NGT, 57, 130), // 111
    r(B4, 0.031941, NGT, 59, L | HIHAT_L), // 112
    r(B13, 0.014997, NLE, 32, 131), // 113
    r(B3, 0.174349, NLE, 23, 132), // 114
    r(CENTROID, 462.034119, NLE, 16, 133), // 115
    r(B2, 0.110901, NAN, 18, L | PERC_L), // 116
    r(B39, 0.000121, NLE, 11, 26), // 117
    r(SKEW, 4.575411, NLE, 50, 134), // 118
    r(SKEW, 40.914612, NLE, 70, 135), // 119
    r(ZCR, 70.0, NLE, 136, 137), // 120
    r(B0, 0.083773, NGT, 120, 138), // 121
    r(SKEW, 3.343801, NGT, 104, 139), // 122
    r(ZCR, 30.0, NGT, 87, 140), // 123
    r(ZCR, 43.833332, NGT, 72, 141), // 124
    r(B27, 0.000182, NLE, 107, 142), // 125
    r(B3, 0.083475, NLE, 55, 143), // 126
    r(B27, 0.000137, NLE, 126, 144), // 127
    r(B0, 0.001749, NLE, 129, L | PERC_L), // 128
    r(B1, 0.195292, NLE, 42, 145), // 129
    r(B29, 0.000932, NLE, 57, 146), // 130
    r(B10, 0.033226, NLE, 32, 147), // 131
    r(B39, 0.170713, NLE, 23, 148), // 132
    r(B2, 0.017314, NGT, 16, 149), // 133
    r(FLATNESS, 0.081754, NLE, 50, L | KICK_L), // 134
    r(B27, 6e-05, NLE, 70, 150), // 135
    r(ZCR, 69.583336, NLE, 151, 152), // 136
    r(B0, 0.163773, NGT, 136, 153), // 137
    r(B1, 0.084933, NGT, 120, 154), // 138
    r(SKEW, 3.731799, NGT, 104, 155), // 139
    r(B2, 0.594842, NLE, 87, 156), // 140
    r(LOG_ATTACK, -1.770537, NGT, 157, 158), // 141
    r(B35, 4.6e-05, NLE, 107, 159), // 142
    r(B13, 0.000776, NGT, 160, 161), // 143
    r(B27, 4.2e-05, NLE, 126, 162), // 144
    r(B13, 0.049504, NLE, 42, 163), // 145
    r(B2, 0.619322, NLE, 57, 164), // 146
    r(B0, 0.01811, NLE, 32, 165), // 147
    r(B35, 0.010564, NGT, 23, L | HIHAT_L), // 148
    r(B27, 4.5e-05, NLE, 16, 166), // 149
    r(CENTROID, 618.451965, NLE, 70, 167), // 150
    r(ZCR, 58.900002, NLE, 168, 169), // 151
    r(B0, 0.163773, NGT, 151, 170), // 152
    r(B3, 0.095706, NLE, 136, 171), // 153
    r(CENTROID, 2561.913818, NGT, 120, 172), // 154
    r(B27, 6.8e-05, NLE, 104, 173), // 155
    r(B1, 0.001966, NGT, 87, 174), // 156
    r(LOG_ATTACK, -1.770537, NAN, 72, 175), // 157
    r(ZCR, 74.666664, NLE, 87, 88), // 158
    r(B1, 0.001927, NGT, 107, 176), // 159
    r(B13, 0.000776, NAN, 161, 177), // 160
    r(B3, 0.083475, NLE, 55, 178), // 161
    r(B8, 0.003929, NLE, 126, 179), // 162
    r(ZCR, 43.166668, NGT, 180, 42), // 163
    r(B33, 8.6e-05, NLE, 57, 181), // 164
    r(B8, 0.073756, NLE, 32, 182), // 165
    r(B0, 0.175721, NGT, 16, L | KICK_L), // 166
    r(B10, 0.00016, NLE, 70, L | KICK_L), // 167
    r(ZCR, 55.375, NLE, 183, 184), // 168
    r(B0, 0.163773, NLE, 168, 185), // 169
    r(B13, 0.001917, NLE, 186, 187), // 170
    r(B13, 0.001917, NLE, 136, 188), // 171
    r(B4, 0.023831, NLE, 120, 189), // 172
    r(ZCR, 16.5, NLE, 104, 190), // 173
    r(B33, 0.000217, NLE, 87, 191), // 174
    r(B13, 0.05102, NLE, 72, L | PERC_L), // 175
    r(ZCR, 23.75, NGT, 107, 192), // 176
    r(CENTROID, 702.108276, NGT, 161, 193), // 177
    r(B13, 0.000776, NLE, 55, 194), // 178
    r(ZCR, 11.6, NLE, 126, 195), // 179
    r(ZCR, 43.166668, NAN, 42, 196), // 180
    r(B10, 0.000864, NGT, 57, 197), // 181
    r(SPREAD_VAR, 1777264.125, NLE, 32, L | SNARE_L), // 182
    r(ZCR, 74.666664, NLE, 198, 199), // 183
    r(B0, 0.082974, NLE, 183, 200), // 184
    r(SKEW, 3.343801, NGT, 168, 201), // 185
    r(B3, 0.095706, NLE, 151, 202), // 186
    r(B2, 0.306534, NLE, 186, 203), // 187
    r(CENTROID, 2828.376465, NGT, 136, 204), // 188
    r(B27, 0.002496, NLE, 120, 205), // 189
    r(ZCR, 11.571428, NGT, 104, 206), // 190
    r(FLATNESS, 0.048579, NGT, 87, L | PERC_L), // 191
    r(SPREAD_VAR, 892078.625, NGT, 107, 207), // 192
    r(B1, 0.503159, NGT, 161, 208), // 193
    r(B1, 0.437565, NGT, 209, 210), // 194
    r(SKEW, 9.108047, NGT, 126, L | TOM_L), // 195
    r(CENTROID, 1955.848389, NLE, 42, 211), // 196
    r(SKEW, 3.144959, NGT, 57, 212), // 197
    r(ZCR, 77.333336, NLE, 213, 214), // 198
    r(B0, 0.079778, NLE, 198, 215), // 199
    r(SKEW, 3.413128, NGT, 183, 216), // 200
    r(B27, 6.8e-05, GT, L | TOM_L, 217), // 201
    r(B13, 0.001917, NLE, 151, 218), // 202
    r(B0, 0.605642, NGT, 186, 219), // 203
    r(B10, 0.000384, NLE, 136, 220), // 204
    r(B13, 7.3e-05, NGT, 120, 221), // 205
    r(B27, 4.6e-05, NLE, 104, 222), // 206
    r(B27, 0.000105, NLE, 107, L | PERC_L), // 207
    r(FLATNESS, 0.045172, NGT, 161, 223), // 208
    r(B1, 0.437565, NAN, 55, 224), // 209
    r(ZCR, 165.333328, NLE, 72, 73), // 210
    r(B27, 0.000104, NLE, 42, 225), // 211
    r(B1, 0.030434, NLE, 57, L | PERC_L), // 212
    r(ZCR, 77.071426, NLE, 226, 227), // 213
    r(B0, 0.06554, NLE, 228, 229), // 214
    r(SKEW, 3.542206, NGT, 198, 230), // 215
    r(B2, 0.159298, NGT, 231, 232), // 216
    r(ZCR, 55.375, NLE, 183, 184), // 217
    r(B39, 6.8e-05, NGT, 233, 234), // 218
    r(B39, 6.7e-05, NLE, 186, 235), // 219
    r(B13, 7.2e-05, NLE, 136, L | KICK_HAT_L), // 220
    r(B13, 0.004189, NLE, 120, 236), // 221
    r(B0, 0.075832, NLE, 104, 237), // 222
    r(ZCR, 11.6, NGT, 161, 238), // 223
    r(CENTROID, 702.108276, NGT, 55, 239), // 224
    r(ZCR, 19.4, NGT, 42, 240), // 225
    r(B29, 0.00962, NGT, 241, 242), // 226
    r(B0, 0.062938, NGT, 243, 244), // 227
    r(B0, 0.219038, NGT, 245, 246), // 228
    r(SKEW, 2.61608, NLE, 247, 248), // 229
    r(B27, 0.000127, NLE, 198, 249), // 230
    r(B2, 0.159298, NAN, 183, L | TOM_L), // 231
    r(ZCR, 74.666664, NLE, 198, 199), // 232
    r(B39, 6.8e-05, NAN, 151, 250), // 233
    r(ZCR, 58.900002, NLE, 168, 169), // 234
    r(CENTROID, 702.108276, NGT, 186, 251), // 235
    r(B1, 0.508562, NLE, 120, 252), // 236
    r(SPREAD_VAR, 2302903.25, NLE, 104, 253), // 237
    r(B33, 5e-05, NLE, 161, 254), // 238
    r(B8, 0.001128, NLE, 55, 255), // 239
    r(B2, 0.001536, NGT, 42, 256), // 240
    r(B1, 0.096196, NGT, 257, 258), // 241
    r(B13, 0.035709, NLE, 241, 259), // 242
    r(B0, 0.062938, NAN, 244, 260), // 243
    r(B1, 0.105807, GT, 261, 262), // 244
    r(B0, 0.06554, NLE, 213, 263), // 245
    r(B1, 0.0893, NGT, 245, 264), // 246
    r(SKEW, 3.542206, NGT, 265, 266), // 247
    r(B39, 2.5e-05, NLE, 247, 267), // 248
    r(FLATNESS, 0.07309, NLE, 198, 268), // 249
    r(B0, 0.605642, NGT, 151, 269), // 250
    r(B13, 0.000317, NGT, 186, 270), // 251
    r(B2, 0.030854, NGT, 120, 271), // 252
    r(SPREAD_VAR, 448172.09375, NGT, 104, L | TOM_L), // 253
    r(B10, 0.00072, NLE, 161, L | KICK_L), // 254
    r(B13, 0.000314, NLE, 55, 272), // 255
    r(B4, 0.590126, NLE, 42, 273), // 256
    r(B29, 0.00962, NGT, 274, 275), // 257
    r(ZCR, 76.5, NLE, 257, 276), // 258
    r(B2, 0.011187, NLE, 241, 277), // 259
    r(CENTROID, 3367.833008, NLE, 278, 279), // 260
    r(B3, 0.098095, NGT, 280, 281), // 261
    r(B3, 0.121931, NGT, 282, 283), // 262
    r(CENTROID, 3367.833008, NLE, 213, 284), // 263
    r(B13, 0.000754, NLE, 245, 285), // 264
    r(CENTROID, 3367.833008, NLE, 228, 286), // 265
    r(B27, 0.000127, NLE, 265, 287), // 266
    r(B2, 0.23287, NGT, 247, 288), // 267
    r(CENTROID, 982.250732, NLE, 198, 289), // 268
    r(CENTROID, 721.526428, NGT, 151, 290), // 269
    r(B33, 6.3e-05, NLE, 186, L | KICK_L), // 270
    r(B1, 0.254909, NGT, 120, L | KICK_HAT_L), // 271
    r(ZCR, 9.857142, NLE, 55, 291), // 272
    r(B3, 0.773032, NLE, 42, 292), // 273
    r(B13, 0.030255, NGT, 293, 294), // 274
    r(B13, 0.035709, NLE, 274, 295), // 275
    r(B13, 0.001917, NGT, 257, 296), // 276
    r(FLATNESS, 0.643014, NGT, 241, 297), // 277
    r(B8, 0.001233, NGT, 244, 298), // 278
    r(B27, 0.000739, NGT, 299, 300), // 279
    r(B3, 0.121931, NGT, 282, 283), // 280
    r(ZCR, 30.0, NLE, 280, 301), // 281
    r(B4, 0.049711, NGT, 302, 303), // 282
    r(SPREAD_VAR, 220615.671875, GT, 304, 305), // 283
    r(B8, 0.002913, NLE, 213, 306), // 284
    r(B35, 8.1e-05, GT, 307, 308), // 285
    r(B2, 0.625154, NGT, 228, 309), // 286
    r(B8, 0.000179, GT, 310, 311), // 287
    r(B29, 0.000739, NLE, 247, L | SNARE_L), // 288
    r(B27, 4.5e-05, NLE, 198, 312), // 289
    r(B2, 0.06734, NLE, 151, 313), // 290
    r(SPREAD_VAR, 2719282.0, NLE, 55, L | KICK_L), // 291
    r(B27, 6.8e-05, NLE, 42, 314), // 292
    r(B33, 0.005008, NGT, 315, 316), // 293
    r(B3, 0.046573, NLE, 293, 317), // 294
    r(B2, 0.011187, NLE, 274, 318), // 295
    r(B0, 0.670077, NLE, 257, 319), // 296
    r(B8, 0.089361, NLE, 241, L | HIHAT_L), // 297
    r(B0, 0.051209, NLE, 244, 320), // 298
    r(B27, 0.000739, NAN, 278, 321), // 299
    r(B8, 0.001233, NGT, 244, 298), // 300
    r(B35, 1.6e-05, NLE, 280, 322), // 301
    r(B0, 0.062938, NLE, 323, 324), // 302
    r(SPREAD_VAR, 287578.59375, NGT, 325, 302), // 303
    r(B8, 0.001287, NGT, 282, 326), // 304
    r(B4, 0.049711, NGT, 302, 303), // 305
    r(B27, 1.6e-05, GT, 327, 328), // 306
    r(B0, 0.669587, NGT, 308, L | KICK_HAT_L), // 307
    r(B13, 0.000754, NLE, 245, 329), // 308
    r(ZCR, 37.833332, GT, L | SNARE_L, 228), // 309
    r(ZCR, 21.75, NLE, 311, 330), // 310
    r(B27, 0.000127, NLE, 265, 331), // 311
    r(B10, 5.3e-05, NGT, 198, 332), // 312
    r(B33, 0.000129, NLE, 151, 333), // 313
    r(B29, 7.2e-05, NLE, 42, 334), // 314
    r(B29, 0.007821, NGT, 335, 336), // 315
    r(B2, 0.051828, NLE, 315, 337), // 316
    r(B10, 0.01866, NGT, 293, 338), // 317
    r(FLATNESS, 0.651344, NLE, 274, 339), // 318
    r(B8, 0.001399, NGT, 257, 340), // 319
    r(B27, 0.000739, NLE, 244, 341), // 320
    r(ZCR, 21.875, NLE, 278, 342), // 321
    r(B2, 0.207125, NLE, 280, L | TOM_L), // 322
    r(B13, 0.001917, NLE, 343, 344), // 323
    r(B13, 0.002143, NLE, 345, 346), // 324
    r(SPREAD_VAR, 287578.59375, NAN, 302, L | SNARE_L), // 325
    r(B4, 0.0995, NLE, 347, 348), // 326
    r(B13, 0.000529, NGT, 213, L | TOM_L), // 327
    r(ZCR, 77.071426, NLE, 226, 227), // 328
    r(CENTROID, 702.108276, GT, 349, 350), // 329
    r(B27, 2.6e-05, NLE, 311, 351), // 330
    r(B29, 3.9e-05, NGT, 352, 353), // 331
    r(B8, 0.000502, NGT, 198, L | PERC_L), // 332
    r(B29, 7.4e-05, NGT, 151, 354), // 333
    r(B0, 0.005599, NLE, 42, L | PERC_L), // 334
    r(B29, 0.008198, NGT, 355, 356), // 335
    r(B13, 0.024075, NAN, 357, 358), // 336
    r(B13, 0.035709, NLE, 315, 359), // 337
    r(B27, 0.023868, NLE, 293, 360), // 338
    r(SPREAD_VAR, 1858628.125, NLE, 274, 361), // 339
    r(SPREAD_VAR, 1086361.0, NLE, 257, L | TOM_L), // 340
    r(ZCR, 55.75, NLE, 244, 362), // 341
    r(ZCR, 16.375, NGT, 278, L | TOM_L), // 342
    r(B13, 0.001917, LE, 363, 226), // 343
    r(B2, 0.315789, NGT, 364, 365), // 344
    r(B13, 0.001917, NLE, 343, 344), // 345
    r(CENTROID, 2790.041504, NLE, L | SNARE_L, 366), // 346
    r(B4, 0.049711, NGT, 302, 303), // 347
    r(B3, 0.198192, NGT, 347, L | PERC_L), // 348
    r(B1, 0.566349, NLE, 245, 367), // 349
    r(B0, 0.06554, NLE, 213, 263), // 350
    r(LOG_ATTACK, -1.537568, NLE, 311, 368), // 351
    r(B29, 3.9e-05, NAN, 265, 369), // 352
    r(CENTROID, 3367.833008, NLE, 228, 286), // 353
    r(B10, 0.000341, NLE, 151, L | KICK_L), // 354
    r(B29, 0.007821, NGT, 370, 371), // 355
    r(B2, 0.007795, NLE, 355, 372), // 356
    r(B2, 0.007795, NLE, 335, 373), // 357
    r(B13, 0.024075, NGT, 374, 357), // 358
    r(B4, 0.203157, NGT, 315, 375), // 359
    r(B10, 0.230206, NLE, 293, 376), // 360
    r(B27, 0.008636, NGT, 274, 377), // 361
    r(B1, 0.00351, NGT, 378, 379), // 362
    r(B39, 6.8e-05, NLE, 380, 381), // 363
    r(B0, 0.606624, GT, 382, 383), // 364
    r(SPREAD_VAR, 245729.6875, NGT, 364, 384), // 365
    r(B33, 1.2e-05, GT, L | TOM_L, L | SNARE_L), // 366
    r(B10, 0.000307, NLE, 245, 385), // 367
    r(ZCR, 20.666666, NLE, 311, L | TOM_L), // 368
    r(B13, 0.000511, NLE, 265, L | PERC_L), // 369
    r(CENTROID, 2531.032715, NLE, 386, 387), // 370
    r(B2, 0.007795, NLE, 370, 388), // 371
    r(B13, 0.035709, NLE, 355, 389), // 372
    r(SPREAD_VAR, 2318872.5, NGT, 335, 390), // 373
    r(B2, 0.013128, NLE, 391, 392), // 374
    r(SKEW, 0.654409, NLE, 315, 393), // 375
    r(B4, 0.058424, NLE, 293, 394), // 376
    r(B35, 0.000869, NGT, 274, 395), // 377
    r(B1, 0.105807, GT, 261, 262), // 378
    r(B29, 0.000103, NLE, 378, L | PERC_L), // 379
    r(B39, 6.8e-05, NLE, 396, 397), // 380
    r(SKEW, 5.996243, NGT, 380, 398), // 381
    r(B39, 6.7e-05, NLE, 399, 400), // 382
    r(B0, 0.613617, NGT, 401, 402), // 383
    r(B27, 1.2e-05, GT, L | TOM_L, 403), // 384
    r(B13, 0.000273, NLE, 245, 404), // 385
    r(ZCR, 69.583336, NLE, 405, 406), // 386
    r(B0, 0.082411, NGT, 386, 407), // 387
    r(B13, 0.035709, NLE, 370, 408), // 388
    r(B35, 0.001867, NGT, 355, 409), // 389
    r(B35, 0.02048, NGT, 410, 411), // 390
    r(B2, 0.007795, NLE, 335, 373), // 391
    r(FLATNESS, 0.651344, NGT, 391, 412), // 392
    r(B13, 0.00115, NGT, 315, L | HIHAT_L), // 393
    r(B27, 0.000949, NGT, 293, 413), // 394
    r(LOG_ATTACK, -1.782904, NGT, 274, 414), // 395
    r(B2, 0.35154, NLE, 415, 416), // 396
    r(B1, 0.596174, NLE, 417, 418), // 397
    r(B8, 0.002027, NLE, 380, 419), // 398
    r(B0, 0.613617, NGT, 401, 402), // 399
    r(CENTROID, 721.526428, NGT, 399, 420), // 400
    r(B39, 6.8e-05, NLE, 380, 381), // 401
    r(B39, 6.7e-05, NLE, 401, 421), // 402
    r(B0, 0.606624, GT, 382, 383), // 403
    r(B8, 0.000129, NLE, 245, 422), // 404
    r(CENTROID, 2529.265625, NLE, 423, 424), // 405
    r(B0, 0.082411, NLE, 425, 426), // 406
    r(B10, 0.004778, NGT, 427, 428), // 407
    r(CENTROID, 9484.806641, NGT, 370, 429), // 408
    r(B8, 0.40238, NLE, 355, 430), // 409
    r(B35, 0.02048, NAN, 335, L | CLAP_L), // 410
    r(B29, 0.008198, NGT, 355, 356), // 411
    r(B1, 0.00351, NLE, 391, L | SNARE_HAT_L), // 412
    r(B27, 0.011882, NLE, 293, L | CLAP_L), // 413
    r(B2, 0.007336, NLE, 274, 431), // 414
    r(B2, 0.35154, GT, L | TOM_L, 432), // 415
    r(B39, 6.8e-05, NLE, 415, 433), // 416
    r(B39, 6.8e-05, NLE, 396, 434), // 417
    r(B13, 0.000792, NLE, 417, 435), // 418
    r(B1, 0.431853, NGT, 380, 436), // 419
    r(B2, 0.055539, NGT, 399, L | KICK_L), // 420
    r(CENTROID, 721.526428, NGT, 401, 437), // 421
    r(SPREAD_VAR, 633990.5625, NGT, 245, L | KICK_L), // 422
    r(B0, 0.139205, NLE, 438, 439), // 423
    r(B0, 0.081401, NGT, 423, 440), // 424
    r(B33, 0.000102, NLE, 405, 441), // 425
    r(B13, 0.004507, NGT, 442, 425), // 426
    r(B33, 0.000105, NGT, 443, 444), // 427
    r(B33, 4.7e-05, NLE, 427, L | PERC_L), // 428
    r(B27, 0.001083, NGT, 370, 445), // 429
    r(SPREAD_VAR, 2138514.25, NLE, 355, 446), // 430
    r(FLATNESS, 0.591009, NLE, 274, 447), // 431
    r(B39, 6.8e-05, GT, L | KICK_L, 448), // 432
    r(B0, 0.164939, NGT, 449, 450), // 433
    r(B1, 0.596174, NLE, 451, 452), // 434
    r(CENTROID, 710.210022, NGT, 417, 453), // 435
    r(B8, 0.001271, NLE, 380, 454), // 436
    r(B35, 5.8e-05, NLE, 401, 455), // 437
    r(B1, 0.116272, NGT, 456, 457), // 438
    r(B29, 0.009858, NLE, 438, 458), // 439
    r(B33, 0.000105, NLE, 423, 459), // 440
    r(B39, 2.6e-05, NLE, 405, 460), // 441
    r(B13, 0.004507, NAN, 425, L | PERC_L), // 442
    r(B33, 0.000105, NAN, 386, 461), // 443
    r(ZCR, 69.583336, NLE, 405, 406), // 444
    r(SPREAD_VAR, 1777264.125, NLE, 370, L | HIHAT_L), // 445
    r(CENTROID, 9484.806641, NLE, 462, 463), // 446
    r(B13, 0.002322, NGT, 274, L | CYMBAL_L), // 447
    r(B0, 0.164939, NGT, 464, 465), // 448
    r(B39, 6.8e-05, NLE, 415, 466), // 449
    r(B39, 2.6e-05, NLE, 449, 467), // 450
    r(B1, 0.596174, GT, L | TOM_L, 468), // 451
    r(B33, 6.6e-05, NLE, 451, 469), // 452
    r(B10, 0.000182, NLE, 417, 470), // 453
    r(B29, 4.1e-05, NLE, 380, 471), // 454
    r(B39, 2.2e-05, NLE, 401, 472), // 455
    r(B1, 0.096669, NGT, 473, 474), // 456
    r(ZCR, 62.166668, NLE, 456, 475), // 457
    r(B3, 0.007441, NGT, 438, 476), // 458
    r(B13, 0.002105, NLE, 423, 477), // 459
    r(B13, 0.001917, NLE, 405, 478), // 460
    r(B13, 0.002105, NLE, 386, 479), // 461
    r(B29, 0.007821, NGT, 370, 371), // 462
    r(B0, 0.001052, NLE, 462, 480), // 463
    r(B33, 0.000105, NLE, 226, 481), // 464
    r(B39, 2.6e-05, NLE, 464, 482), // 465
    r(B0, 0.164939, NGT, 415, 483), // 466
    r(SKEW, 4.975183, NGT, 449, 484), // 467
    r(B2, 0.349341, NGT, 396, 485), // 468
    r(B13, 0.000776, NGT, 451, 486), // 469
    r(B13, 0.000157, NLE, 417, 487), // 470
    r(B3, 0.012398, NGT, 380, L | TOM_L), // 471
    r(B0, 0.673071, NLE, 401, 488), // 472
    r(B1, 0.116272, NGT, 489, 490), // 473
    r(CENTROID, 1824.449585, NLE, 473, 491), // 474
    r(B13, 0.001969, NLE, 456, 492), // 475
    r(SKEW, 2.412478, NLE, 438, 493), // 476
    r(B10, 0.002471, NGT, 423, 494), // 477
    r(B35, 5.2e-05, NLE, 405, 495), // 478
    r(SKEW, 2.815474, NLE, 386, 496), // 479
    r(B2, 0.002537, NLE, 462, 497), // 480
    r(B0, 0.164939, NGT, 498, 499), // 481
    r(B35, 5.2e-05, NGT, 464, 500), // 482
    r(SKEW, 5.092498, NLE, 415, 501), // 483
    r(B1, 0.323158, NGT, 449, 502), // 484
    r(B27, 0.00013, NGT, 503, 504), // 485
    r(B10, 0.000677, NGT, 451, 505), // 486
    r(CENTROID, 880.857849, NGT, 417, 506), // 487
    r(B33, 1.2e-05, NLE, 401, 507), // 488
    r(B1, 0.117718, NGT, 508, 509), // 489
    r(B0, 0.120244, NLE, 489, 510), // 490
    r(B8, 0.000965, NLE, 473, 511), // 491
    r(B8, 0.004722, NLE, 456, 512), // 492
    r(B35, 0.002215, NLE, 513, 514), // 493
    r(B33, 4.1e-05, NLE, 423, 515), // 494
    r(B33, 6.2e-05, NLE, 405, 516), // 495
    r(B27, 2.3e-05, NGT, 386, 517), // 496
    r(B0, 6e-05, NLE, 462, 518), // 497
    r(B33, 0.000105, NLE, 226, 519), // 498
    r(B35, 5.5e-05, NLE, 498, 520), // 499
    r(B2, 0.19356, NLE, 464, L | KICK_L), // 500
    r(B0, 0.610714, NGT, 415, 521), // 501
    r(B2, 0.047647, NGT, 449, 522), // 502
    r(B27, 0.00013, NAN, 396, 523), // 503
    r(B2, 0.35154, NLE, 415, 416), // 504
    r(B4, 0.027923, NLE, 451, 524), // 505
    r(B13, 5.2e-05, NLE, 417, L | KICK_HAT_L), // 506
    r(SPREAD_VAR, 1108111.75, NGT, 401, L | KICK_HAT_L), // 507
    r(B1, 0.118215, NGT, 525, 526), // 508
    r(CENTROID, 1824.449585, NLE, 527, 528), // 509
    r(FLATNESS, 0.156925, NGT, 489, 529), // 510
    r(B27, 0.000153, NLE, 473, 530), // 511
    r(B29, 0.000178, NLE, 456, 531), // 512
    r(SKEW, 2.412478, NLE, 438, 532), // 513
    r(B1, 0.000754, NGT, 513, 533), // 514
    r(B27, 2.9e-05, NLE, 423, 534), // 515
    r(B8, 0.007484, NLE, 405, 535), // 516
    r(B10, 0.000328, NLE, 386, 536), // 517
    r(LOG_ATTACK, -1.788314, NLE, 462, L | HIHAT_L), // 518
    r(B0, 0.164939, NGT, 226, 537), // 519
    r(B33, 6.4e-05, NLE, 498, 538), // 520
    r(B35, 5.9e-05, NLE, 415, 539), // 521
    r(B27, 6.5e-05, NLE, 449, L | KICK_L), // 522
    r(B10, 0.000664, NLE, 396, L | KICK_L), // 523
    r(SPREAD_VAR, 973716.5, NLE, 451, 540), // 524
    r(ZCR, 268.5, NGT, 541, 542), // 525
    r(ZCR, 164.642853, NGT, 543, 544), // 526
    r(ZCR, 58.0, NLE, 545, 546), // 527
    r(SPREAD_VAR, 810824.125, NLE, 527, 547), // 528
    r(B35, 0.004607, NLE, 489, 548), // 529
    r(B4, 0.006719, NGT, 473, L | KICK_L), // 530
    r(B33, 7.3e-05, NLE, 456, 549), // 531
    r(B35, 0.002215, NLE, 438, 550), // 532
    r(FLATNESS, 0.554829, NLE, 513, 551), // 533
    r(B33, 1.2e-05, NLE, 423, L | KICK_L), // 534
    r(B27, 0.00013, NGT, 405, 552), // 535
    r(FLATNESS, 0.097968, NGT, 386, L | KICK_HAT_L), // 536
    r(B0, 0.931052, NLE, 553, 554), // 537
    r(SKEW, 5.092498, NLE, 498, 555), // 538
    r(B39, 2.1e-05, NLE, 415, 556), // 539
    r(SPREAD_VAR, 298237.53125, NGT, 451, L | TOM_L), // 540
    r(B1, 0.118215, NGT, 557, 558), // 541
    r(B2, 0.003738, NLE, 559, 560), // 542
    r(B8, 0.008453, NLE, 561, 562), // 543
    r(B0, 0.044168, NLE, 543, 563), // 544
    r(CENTROID, 1820.498535, NLE, 564, 565), // 545
    r(B29, 0.000149, GT, 566, 567), // 546
    r(B10, 0.000182, NGT, 527, 568), // 547
    r(SKEW, 0.994891, NLE, 489, 569), // 548
    r(B27, 0.000207, NLE, 456, 570), // 549
    r(B1, 0.000754, NGT, 438, 571), // 550
    r(LOG_ATTACK, -1.367288, NGT, 513, 572), // 551
    r(B2, 0.10675, NLE, 405, L | KICK_L), // 552
    r(B0, 0.164939, NGT, 226, 573), // 553
    r(B35, 5.2e-05, NLE, 553, 574), // 554
    r(B1, 0.551554, NLE, 498, 575), // 555
    r(B27, 6.3e-05, NGT, 415, 576), // 556
    r(B0, 0.204579, NLE, 577, 578), // 557
    r(B35, 0.000141, NLE, 557, 579), // 558
    r(B33, 0.008699, NGT, 580, 581), // 559
    r(B13, 0.033929, NLE, 559, 582), // 560
    r(ZCR, 164.642853, NGT, 583, 584), // 561
    r(ZCR, 164.642853, NGT, 583, 585), // 562
    r(B13, 0.000758, NLE, 543, L | SNARE_HAT_L), // 563
    r(B0, 0.120244, NLE, 586, 587), // 564
    r(B8, 0.006501, NGT, 588, 589), // 565
    r(B29, 0.000183, NGT, 545, L | KICK_L), // 566
    r(CENTROID, 1820.498535, NLE, 564, 565), // 567
    r(B13, 0.000144, NGT, 527, L | TOM_L), // 568
    r(SPREAD_VAR, 16803.294922, NGT, 489, L | SNARE_L), // 569
    r(B35, 2.9e-05, NLE, 456, 590), // 570
    r(FLATNESS, 0.554829, NLE, 438, 591), // 571
    r(B33, 0.000311, NGT, 592, 513), // 572
    r(B39, 2.6e-05, NLE, 226, 593), // 573
    r(B33, 6.2e-05, NLE, 553, 594), // 574
    r(B13, 0.000102, NLE, 498, 595), // 575
    r(LOG_ATTACK, -1.635838, NGT, 415, L | KICK_HAT_L), // 576
    r(B0, 0.172408, NLE, 596, 597), // 577
    r(B33, 0.005982, NGT, 577, 598), // 578
    r(B10, 0.01183, NLE, 557, 599), // 579
    r(B33, 0.005232, NGT, 600, 601), // 580
    r(B35, 0.001728, NLE, 580, 602), // 581
    r(B27, 0.005793, NGT, 559, 603), // 582
    r(B8, 0.008716, NLE, 604, 605), // 583
    r(LOG_ATTACK, -1.544758, GT, 606, 607), // 584
    r(B1, 0.182617, NGT, 584, 608), // 585
    r(B0, 0.04896, NLE, 609, 610), // 586
    r(SKEW, 0.633998, GT, 611, 612), // 587
    r(B8, 0.006501, NAN, 589, 613), // 588
    r(B1, 0.574832, NGT, 614, 615), // 589
    r(B29, 8.4e-05, NLE, 456, 616), // 590
    r(CENTROID, 6786.947266, NLE, 438, 617), // 591
    r(B33, 0.000311, NAN, 513, L | SNARE_HAT_L), // 592
    r(B35, 5.2e-05, NLE, 226, 618), // 593
    r(SKEW, 3.516821, NGT, 553, 619), // 594
    r(LOG_ATTACK, -1.419129, NLE, 498, 620), // 595
    r(B1, 0.118215, NGT, 621, 622), // 596
    r(B33, 0.005812, NGT, 596, 623), // 597
    r(FLATNESS, 0.647227, NGT, 577, L | CLAP_L), // 598
    r(B33, 0.000158, NGT, 557, L | TOM_L), // 599
    r(B33, 0.008699, NGT, 541, 624), // 600
    r(B4, 0.203157, NLE, 600, 625), // 601
    r(B39, 0.000451, NGT, 580, L | PERC_L), // 602
    r(B39, 0.144226, NLE, 559, 626), // 603
    r(B8, 0.008453, LE, 627, 628), // 604
    r(B4, 0.037695, GT, 629, 630), // 605
    r(B0, 0.174298, NGT, 607, L | KICK_HAT_L), // 606
    r(B39, 0.001466, NGT, 631, 632), // 607
    r(SPREAD_VAR, 23633.867188, NGT, 584, 633), // 608
    r(B0, 0.044168, NLE, 508, 634), // 609
    r(CENTROID, 5592.9375, NGT, 635, 636), // 610
    r(B10, 0.013823, NGT, 586, 637), // 611
    r(B0, 0.04896, NLE, 609, 610), // 612
    r(SPREAD_VAR, 272906.21875, NGT, 589, 638), // 613
    r(B1, 0.574832, NAN, 564, 639), // 614
    r(B0, 0.120244, NLE, 586, 587), // 615
    r(SKEW, 2.88257, NLE, 456, 640), // 616
    r(B1, 0.302479, NLE, 438, 641), // 617
    r(B33, 6.2e-05, NLE, 226, 642), // 618
    r(B13, 0.001147, NLE, 553, 643), // 619
    r(B13, 7.6e-05, NLE, 498, 644), // 620
    r(B0, 0.204579, NLE, 645, 646), // 621
    r(B35, 0.000141, NLE, 621, 647), // 622
    r(FLATNESS, 0.234608, NGT, 596, 648), // 623
    r(FLATNESS, 0.651629, NGT, 649, 650), // 624
    r(FLATNESS, 0.641316, NGT, 600, 651), // 625
    r(B29, 0.004454, NGT, 559, 652), // 626
    r(B39, 0.000285, NGT, 653, 654), // 627
    r(B39, 0.000356, NGT, 655, 656), // 628
    r(ZCR, 551.575012, NLE, 604, L | SNARE_L), // 629
    r(B8, 0.008453, LE, 627, 628), // 630
    r(B39, 0.001466, NAN, 583, 657), // 631
    r(B8, 0.008716, NLE, 604, 605), // 632
    r(LOG_ATTACK, -1.836904, NLE, 658, 659), // 633
    r(CENTROID, 5592.9375, NGT, 660, 661), // 634
    r(CENTROID, 5592.9375, NAN, 609, 662), // 635
    r(B0, 0.044168, NLE, 508, 634), // 636
    r(LOG_ATTACK, -1.523865, NLE, 586, L | SNARE_L), // 637
    r(LOG_ATTACK, -1.673163, NLE, 589, 663), // 638
    r(B33, 2.8e-05, NLE, 564, 664), // 639
    r(B13, 0.000151, NGT, 456, L | KICK_L), // 640
    r(B13, 0.022381, NLE, 438, 665), // 641
    r(SKEW, 3.516821, NLE, 226, 666), // 642
    r(LOG_ATTACK, -1.737565, NLE, 553, 667), // 643
    r(FLATNESS, 0.089403, NLE, 498, L | KICK_HAT_L), // 644
    r(B1, 0.118215, NLE, 668, 669), // 645
    r(B3, 0.006491, NLE, 645, 670), // 646
    r(B10, 0.01183, NLE, 621, 671), // 647
    r(B2, 0.128134, NGT, 672, 673), // 648
    r(B4, 0.203157, NLE, 674, 675), // 649
    r(B2, 0.023718, NGT, 676, 649), // 650
    r(B10, 0.035425, NLE, 600, 677), // 651
    r(B27, 0.056641, NLE, 559, 678), // 652
    r(B39, 0.000356, NGT, 655, 656), // 653
    r(B27, 0.018859, NLE, 653, 679), // 654
    r(ZCR, 242.25, NLE, 680, 681), // 655
    r(B27, 0.017369, NGT, 682, 683), // 656
    r(B0, 0.039207, NGT, 583, 684), // 657
    r(LOG_ATTACK, -1.544758, GT, 606, 607), // 658
    r(B0, 0.063331, NGT, 658, 685), // 659
    r(CENTROID, 5592.9375, NAN, 508, 686), // 660
    r(B1, 0.118215, NGT, 525, 526), // 661
    r(B27, 0.006313, NLE, 609, 687), // 662
    r(B13, 0.000698, NLE, 589, 688), // 663
    r(SKEW, 5.042546, NLE, 564, 689), // 664
    r(B8, 0.105559, NLE, 438, 690), // 665
    r(B0, 0.449859, NGT, 226, L | KICK_L), // 666
    r(B29, 8e-05, NLE, 553, 691), // 667
    r(ZCR, 69.583336, NLE, 692, 693), // 668
    r(B3, 0.017452, NGT, 668, 694), // 669
    r(ZCR, 279.5, NLE, 645, 695), // 670
    r(B35, 8.6e-05, NLE, 621, 696), // 671
    r(ZCR, 565.5, NGT, 596, 697), // 672
    r(B1, 0.009879, NGT, 698, 672), // 673
    r(B4, 0.180523, GT, L | SNARE_L, 699), // 674
    r(SPREAD_VAR, 1112678.875, NLE, 674, 700), // 675
    r(B2, 0.023718, NAN, 649, L | CLAP_L), // 676
    r(SKEW, 0.512145, NLE, 600, L | HIHAT_L), // 677
    r(B2, 0.001183, NLE, 559, 701), // 678
    r(B3, 0.044524, NLE, 653, L | KICK_HAT_L), // 679
    r(B13, 0.006128, NLE, 702, 703), // 680
    r(B10, 0.013182, NLE, 680, 704), // 681
    r(B27, 0.017369, NAN, 655, 705), // 682
    r(ZCR, 242.25, NLE, 680, 681), // 683
    r(SPREAD_VAR, 2698393.75, NLE, 583, 706), // 684
    r(B10, 0.003624, NLE, 658, L | KICK_HAT_L), // 685
    r(B33, 0.000773, NLE, 508, L | TOM_L), // 686
    r(B13, 0.009161, NLE, 609, 707), // 687
    r(ZCR, 12.666667, NGT, 589, L | KICK_L), // 688
    r(SPREAD_VAR, 272906.21875, NGT, 564, L | KICK_L), // 689
    r(B4, 0.015773, NGT, 438, 708), // 690
    r(B29, 3.7e-05, NLE, 553, 709), // 691
    r(B1, 0.118215, NLE, 710, 711), // 692
    r(B29, 0.000149, NLE, 692, 712), // 693
    r(B33, 0.005982, NGT, 668, 713), // 694
    r(FLATNESS, 0.332863, NLE, 645, 714), // 695
    r(B13, 0.001742, NGT, 621, 715), // 696
    r(SPREAD_VAR, 51057.488281, NGT, 596, 716), // 697
    r(B1, 0.009879, NAN, 672, L | SNARE_HAT_L), // 698
    r(SPREAD_VAR, 1112678.875, NLE, 541, 717), // 699
    r(B2, 0.015946, NLE, 674, 718), // 700
    r(LOG_ATTACK, -2.38676, NGT, 559, L | CYMBAL_L), // 701
    r(B35, 0.000141, NLE, 525, 719), // 702
    r(B27, 0.004377, NGT, 720, 721), // 703
    r(SKEW, 0.891182, NGT, 680, 722), // 704
    r(B35, 0.001997, NGT, 655, L | HIHAT_L), // 705
    r(B27, 0.002679, NLE, 583, L | SNARE_L), // 706
    r(B39, 0.000163, NLE, 609, 723), // 707
    r(LOG_ATTACK, -2.456918, NGT, 438, 724), // 708
    r(B27, 8.5e-05, NLE, 553, 725), // 709
    r(B0, 0.204579, GT, 726, 727), // 710
    r(B3, 0.017452, NGT, 710, 728), // 711
    r(B27, 0.000171, NLE, 692, 729), // 712
    r(B2, 0.021789, NGT, 668, 730), // 713
    r(B0, 0.002923, NLE, 645, L | PERC_L), // 714
    r(B2, 0.098478, NGT, 621, L | TOM_L), // 715
    r(SKEW, 0.359137, NGT, 596, L | HIHAT_L), // 716
    r(B1, 0.001614, NLE, 731, 732), // 717
    r(CENTROID, 5583.759277, NGT, 674, 733), // 718
    r(B13, 0.013451, NLE, 525, 734), // 719
    r(B39, 6.8e-05, GT, L | KICK_HAT_L, 702), // 720
    r(B3, 0.03978, NLE, 720, L | SNARE_L), // 721
    r(B35, 0.000111, NGT, 680, 735), // 722
    r(B3, 0.137188, NLE, 609, L | SNARE_L), // 723
    r(B35, 0.001997, NLE, 438, 736), // 724
    r(SPREAD_VAR, 1756809.125, NLE, 553, 737), // 725
    r(B35, 0.000228, NGT, 738, 739), // 726
    r(CENTROID, 3782.881836, NGT, 740, 741), // 727
    r(B1, 0.001875, NGT, 710, 742), // 728
    r(B13, 0.001183, NLE, 692, 743), // 729
    r(B10, 0.003393, NGT, 668, 744), // 730
    r(B2, 0.016599, NLE, 541, 745), // 731
    r(B33, 0.064991, NGT, 731, 746), // 732
    r(B27, 0.009618, NGT, 674, 747), // 733
    r(B35, 8.6e-05, NLE, 748, 749), // 734
    r(B4, 0.038886, NLE, 680, L | KICK_L), // 735
    r(SPREAD_VAR, 493572.6875, NLE, 438, L | SNARE_L), // 736
    r(SKEW, 8.20505, NLE, 553, 750), // 737
    r(B39, 0.000121, NGT, 751, 752), // 738
    r(B29, 0.002442, NGT, 738, 753), // 739
    r(ZCR, 302.661774, NGT, 754, 755), // 740
    r(B3, 0.077798, NGT, 740, 756), // 741
    r(ZCR, 550.1875, NLE, 710, 757), // 742
    r(B39, 2.2e-05, NLE, 692, 758), // 743
    r(CENTROID, 6360.896484, NLE, 668, L | HIHAT_L), // 744
    r(CENTROID, 5583.759277, NGT, 541, 759), // 745
    r(B2, 0.000214, NGT, 760, 731), // 746
    r(B35, 0.006322, NGT, 674, 761), // 747
    r(B33, 0.000158, NLE, 525, 762), // 748
    r(B27, 0.000703, NLE, 763, 764), // 749
    r(FLATNESS, 0.085894, NGT, 765, 553), // 750
    r(B39, 0.000159, NLE, 766, 767), // 751
    r(B29, 0.00261, GT, L | SNARE_L, 751), // 752
    r(B8, 0.000475, NGT, 738, 768), // 753
    r(ZCR, 293.75, NGT, 769, 770), // 754
    r(B33, 0.000171, NGT, 754, 771), // 755
    r(B29, 0.009791, NGT, 772, 773), // 756
    r(LOG_ATTACK, -2.197281, NGT, 710, 774), // 757
    r(B27, 4.4e-05, NLE, 692, 775), // 758
    r(B1, 0.00017, NGT, 541, 776), // 759
    r(B2, 0.000214, NAN, 731, L | CYMBAL_L), // 760
    r(LOG_ATTACK, -1.986427, NLE, 674, 777), // 761
    r(B35, 8.6e-05, NLE, 525, 778), // 762
    r(B35, 8.6e-05, NLE, 748, 779), // 763
    r(B0, 0.26918, NLE, 763, 780), // 764
    r(FLATNESS, 0.085894, NAN, 553, L | KICK_L), // 765
    r(B35, 0.000312, NLE, 727, 781), // 766
    r(B13, 0.001629, NGT, 766, 782), // 767
    r(ZCR, 344.75, NGT, 783, 738), // 768
    r(B0, 0.172408, NGT, 784, 785), // 769
    r(B27, 0.000667, NGT, 786, 787), // 770
    r(B2, 0.049065, NGT, 754, 788), // 771
    r(FLATNESS, 0.2168, NLE, 789, 790), // 772
    r(B13, 0.0088, NLE, 772, 791), // 773
    r(LOG_ATTACK, -1.565257, NLE, 710, 792), // 774
    r(SPREAD_VAR, 864499.25, NGT, 692, 793), // 775
    r(B2, 0.003892, NGT, 541, L | HIHAT_L), // 776
    r(B0, 0.001858, GT, L | CYMBAL_L, 674), // 777
    r(FLATNESS, 0.169789, NLE, 525, 794), // 778
    r(B33, 0.000158, NLE, 525, 795), // 779
    r(B4, 0.028479, NGT, 796, 763), // 780
    r(B13, 0.001742, NLE, 727, 797), // 781
    r(B39, 2.3e-05, GT, L | KICK_HAT_L, 766), // 782
    r(ZCR, 344.75, NAN, 738, L | HIHAT_L), // 783
    r(ZCR, 293.75, NGT, 798, 799), // 784
    r(B39, 0.00017, NLE, 784, 800), // 785
    r(B27, 0.000667, NAN, 787, L | PERC_L), // 786
    r(SKEW, 0.526475, NLE, 801, 802), // 787
    r(B4, 0.021629, NGT, 754, 803), // 788
    r(B39, 0.003826, NGT, 804, 805), // 789
    r(B1, 0.00553, NGT, 806, 789), // 790
    r(B35, 0.003289, GT, L | HIHAT_L, 772), // 791
    r(SKEW, 0.801017, NGT, 710, 807), // 792
    r(B27, 2.6e-05, NLE, 692, 808), // 793
    r(B27, 0.000703, NLE, 525, 809), // 794
    r(FLATNESS, 0.169789, NLE, 748, 810), // 795
    r(B4, 0.028479, NAN, 763, L | KICK_HAT_L), // 796
    r(ZCR, 80.0, NLE, 811, 812), // 797
    r(ZCR, 333.75, NGT, 813, 814), // 798
    r(B33, 0.017599, NGT, 815, 816), // 799
    r(B13, 0.001742, NLE, 784, 817), // 800
    r(SKEW, 0.531343, NLE, 818, 819), // 801
    r(B0, 0.120244, NLE, 801, 820), // 802
    r(LOG_ATTACK, -1.62866, NLE, 754, 821), // 803
    r(B4, 0.009792, NGT, 740, 822), // 804
    r(B27, 0.00451, GT, L | HIHAT_L, 804), // 805
    r(B1, 0.00553, NAN, 789, L | CYMBAL_L), // 806
    r(B10, 0.024136, NLE, 710, 823), // 807
    r(B0, 0.659804, NGT, 692, 824), // 808
    r(B35, 2.7e-05, NLE, 525, 825), // 809
    r(B27, 0.000703, NLE, 748, 826), // 810
    r(B4, 0.005862, NLE, 727, 827), // 811
    r(B4, 0.005548, NLE, 811, 828), // 812
    r(B35, 0.001354, NGT, 829, 830), // 813
    r(FLATNESS, 0.180374, NLE, 831, 832), // 814
    r(B33, 0.034879, NGT, 833, 834), // 815
    r(B29, 0.014155, NGT, 815, 835), // 816
    r(B39, 4.7e-05, NLE, 784, 836), // 817
    r(SKEW, 0.526475, NLE, 837, 838), // 818
    r(B3, 0.05261, GT, L | HIHAT_L, 839), // 819
    r(B4, 0.000257, NGT, 840, 801), // 820
    r(B0, 0.041541, NGT, 841, 754), // 821
    r(B35, 0.000819, NGT, 740, 842), // 822
    r(B39, 0.000502, NLE, 710, 843), // 823
    r(B2, 0.030661, NLE, 692, L | KICK_L), // 824
    r(B33, 1.7e-05, NLE, 525, 844), // 825
    r(B35, 2.7e-05, NLE, 748, 845), // 826
    r(ZCR, 80.0, NLE, 727, L | KICK_HAT_L), // 827
    r(CENTROID, 810.690186, NLE, 811, 846), // 828
    r(B0, 0.204579, NGT, 847, 848), // 829
    r(B13, 0.002608, NLE, 849, 850), // 830
    r(FLATNESS, 0.1674, NGT, 813, 851), // 831
    r(B3, 0.007449, GT, L | HIHAT_L, 831), // 832
    r(LOG_ATTACK, -1.930109, NGT, 798, 852), // 833
    r(ZCR, 958.5, NGT, 853, 833), // 834
    r(B13, 0.003091, NGT, 815, 854), // 835
    r(B27, 3.5e-05, NGT, 784, 855), // 836
    r(LOG_ATTACK, -1.951592, NGT, 769, 856), // 837
    r(B27, 0.010015, NGT, 857, 837), // 838
    r(B1, 0.004945, GT, L | CYMBAL_L, 818), // 839
    r(B4, 0.000257, NAN, 801, L | HIHAT_L), // 840
    r(B0, 0.041541, NAN, 754, L | SNARE_L), // 841
    r(B4, 0.306389, NLE, 740, 858), // 842
    r(SPREAD_VAR, 868294.5625, NLE, 710, L | SNARE_L), // 843
    r(B2, 0.036328, NGT, 525, 859), // 844
    r(B3, 0.016393, NGT, 748, 860), // 845
    r(B2, 0.037712, NGT, 861, 811), // 846
    r(B35, 0.0013, NGT, 862, 863), // 847
    r(B13, 0.001742, NLE, 847, 864), // 848
    r(LOG_ATTACK, -1.951592, NLE, 829, 865), // 849
    r(ZCR, 525.833313, GT, L | SNARE_HAT_L, 849), // 850
    r(B10, 0.00094, NGT, 813, 866), // 851
    r(ZCR, 510.0, NLE, 798, 867), // 852
    r(ZCR, 958.5, NAN, 833, L | PERC_L), // 853
    r(B8, 0.004432, GT, L | CYMBAL_L, 815), // 854
    r(B29, 0.000111, NGT, 868, 784), // 855
    r(ZCR, 502.0, NGT, 869, 870), // 856
    r(B27, 0.010015, NAN, 837, L | SNARE_HAT_L), // 857
    r(B1, 0.043555, NGT, 871, 740), // 858
    r(SPREAD_VAR, 336116.125, NGT, 525, 872), // 859
    r(B4, 0.004144, NGT, 748, 873), // 860
    r(B2, 0.037712, NAN, 811, L | TOM_L), // 861
    r(B2, 0.006656, NGT, 874, 875), // 862
    r(B1, 0.023008, NLE, 862, 876), // 863
    r(ZCR, 45.75, NGT, 877, 847), // 864
    r(B3, 0.000154, NLE, 878, 879), // 865
    r(B27, 0.001955, NGT, 813, 880), // 866
    r(B35, 0.000626, NGT, 798, 881), // 867
    r(B29, 0.000111, NAN, 784, L | KICK_L), // 868
    r(ZCR, 510.0, NLE, 769, 882), // 869
    r(B0, 0.037361, NLE, 869, 883), // 870
    r(B1, 0.043555, NAN, 740, L | SNARE_L), // 871
    r(SKEW, 8.688819, NLE, 525, L | KICK_L), // 872
    r(ZCR, 17.6, NGT, 884, 748), // 873
    r(B3, 0.002732, NLE, 885, 886), // 874
    r(B0, 0.204579, NGT, 874, 887), // 875
    r(B2, 0.006523, NGT, 862, 888), // 876
    r(ZCR, 45.75, NAN, 847, L | TOM_L), // 877
    r(LOG_ATTACK, -2.419129, NGT, 829, 889), // 878
    r(B0, 0.00063, NGT, 890, 878), // 879
    r(B29, 0.036838, NLE, 813, 891), // 880
    r(B27, 0.029435, NLE, 798, 892), // 881
    r(B35, 0.000787, NLE, 769, 893), // 882
    r(FLATNESS, 0.342093, GT, L | HIHAT_L, 869), // 883
    r(ZCR, 17.6, NAN, 748, L | TOM_L), // 884
    r(B3, 0.006132, NLE, 894, 895), // 885
    r(B4, 0.000249, NGT, 885, 896), // 886
    r(B13, 0.001742, NLE, 874, 897), // 887
    r(B39, 0.000232, GT, L | HIHAT_L, 862), // 888
    r(B29, 0.007526, NGT, 829, 898), // 889
    r(B0, 0.00063, NAN, 878, L | CYMBAL_L), // 890
    r(CENTROID, 4705.570801, NGT, 813, 899), // 891
    r(B33, 0.008315, NLE, 798, 900), // 892
    r(B27, 0.020068, GT, L | CYMBAL_L, 769), // 893
    r(B3, 0.002658, NLE, 901, 902), // 894
    r(FLATNESS, 0.1674, NGT, 903, 894), // 895
    r(ZCR, 332.75, NLE, 885, 904), // 896
    r(CENTROID, 3029.397217, NGT, 905, 874), // 897
    r(ZCR, 410.25, GT, L | HIHAT_L, 829), // 898
    r(B13, 0.006037, NLE, 813, 906), // 899
    r(FLATNESS, 0.201142, GT, L | CLAP_L, 798), // 900
    r(B13, 0.022214, NGT, 907, 908), // 901
    r(B10, 0.007709, NGT, 901, 909), // 902
    r(FLATNESS, 0.1674, NAN, 894, L | PERC_L), // 903
    r(SKEW, 1.147013, GT, L | CLAP_L, 885), // 904
    r(CENTROID, 3029.397217, NAN, 874, L | KICK_L), // 905
    r(B33, 0.004905, GT, L | CYMBAL_L, 813), // 906
    r(LOG_ATTACK, -1.855564, NGT, 910, 911), // 907
    r(B39, 0.000608, NGT, 907, 912), // 908
    r(B3, 0.000277, GT, L | CYMBAL_L, 901), // 909
    r(B8, 0.123085, NGT, 913, 914), // 910
    r(FLATNESS, 0.381687, NGT, 915, 916), // 911
    r(FLATNESS, 0.365025, GT, L | HIHAT_L, 907), // 912
    r(LOG_ATTACK, -1.895476, NGT, 917, 918), // 913
    r(B13, 0.015828, NGT, 919, 913), // 914
    r(B0, 0.130988, NLE, 910, 920), // 915
    r(B0, 0.037361, NGT, 921, 915), // 916
    r(LOG_ATTACK, -1.846479, NGT, 922, 923), // 917
    r(LOG_ATTACK, -1.354181, NLE, 917, 924), // 918
    r(B13, 0.015828, NAN, 913, L | PERC_L), // 919
    r(B1, 0.001361, NGT, 910, 925), // 920
    r(B0, 0.037361, NAN, 915, L | SNARE_HAT_L), // 921
    r(B39, 7.5e-05, NLE, 926, 927), // 922
    r(B1, 0.001181, GT, L | CLAP_L, 922), // 923
    r(B3, 0.002645, GT, L | SNARE_L, 917), // 924
    r(B13, 0.029904, GT, L | CLAP_L, 910), // 925
    r(B39, 6.7e-05, NLE, 928, 929), // 926
    r(B33, 0.000975, NGT, 930, 931), // 927
    r(ZCR, 69.583336, NGT, 932, 933), // 928
    r(ZCR, 269.75, NLE, 928, 934), // 929
    r(CENTROID, 3798.797607, NGT, 926, 935), // 930
    r(B8, 0.023748, NGT, 936, 930), // 931
    r(ZCR, 73.0, NGT, 937, 938), // 932
    r(B10, 0.013473, NLE, 939, 940), // 933
    r(B27, 0.003518, NGT, 928, 941), // 934
    r(B2, 0.004987, GT, L | SNARE_L, 926), // 935
    r(B8, 0.023748, NAN, 930, L | HIHAT_L), // 936
    r(ZCR, 73.0, NAN, 938, L | TOM_L), // 937
    r(B39, 0.000506, NGT, 942, 943), // 938
    r(B8, 0.010279, NLE, 944, 945), // 939
    r(B0, 0.191364, NLE, 939, 946), // 940
    r(B10, 0.045946, NGT, 947, 928), // 941
    r(B4, 0.04824, NGT, 948, 949), // 942
    r(B33, 0.00367, NGT, 950, 942), // 943
    r(B10, 0.014356, NLE, 932, 951), // 944
    r(B2, 0.038379, NGT, 952, 944), // 945
    r(FLATNESS, 0.180016, NGT, 939, 953), // 946
    r(B10, 0.045946, NAN, 928, L | TOM_L), // 947
    r(B4, 0.0709, NGT, 954, 955), // 948
    r(B2, 0.006049, NGT, 948, 956), // 949
    r(B33, 0.00367, NAN, 942, L | CLAP_L), // 950
    r(FLATNESS, 0.180016, NGT, 932, 957), // 951
    r(B2, 0.038379, NAN, 944, L | CYMBAL_L), // 952
    r(SKEW, 2.112, NLE, 939, 958), // 953
    r(B1, 0.009551, NGT, 959, 960), // 954
    r(B0, 0.007531, NGT, 961, 954), // 955
    r(B27, 0.001181, GT, L | SNARE_L, 948), // 956
    r(CENTROID, 4486.775391, NGT, 962, 932), // 957
    r(B1, 0.001721, GT, L | SNARE_L, 939), // 958
    r(B29, 0.004079, NLE, L | SNARE_L, L | HIHAT_L), // 959
    r(B4, 0.026158, GT, L | SNARE_L, 963), // 960
    r(B0, 0.007531, NAN, 954, L | SNARE_HAT_L), // 961
    r(CENTROID, 4486.775391, NAN, 932, L | PERC_L), // 962
    r(B1, 0.022663, NGT, 964, 959), // 963
    r(B1, 0.022663, NAN, 959, L | CLAP_L), // 964
];

#[cfg(test)]
mod tests {
    use super::*;

    const SR: f32 = 44100.0;

    /// A deterministic noise source in −1..1.
    fn noise(n: usize, seed: u32) -> Vec<f32> {
        let mut s = seed;
        (0..n)
            .map(|_| {
                s = s.wrapping_mul(1664525).wrapping_add(1013904223);
                (s >> 8) as f32 / (1u32 << 23) as f32 - 1.0
            })
            .collect()
    }

    fn kick(n: usize) -> Vec<f32> {
        (0..n)
            .map(|i| {
                let t = i as f32 / SR;
                (std::f32::consts::TAU * 55.0 * t).sin() * (-t * 8.0).exp()
            })
            .collect()
    }

    fn hat(n: usize) -> Vec<f32> {
        // White noise high-passed by a first difference, decaying fast.
        let w = noise(n + 1, 7);
        (0..n).map(|i| (w[i + 1] - w[i]) * 0.5 * (-(i as f32 / SR) * 40.0).exp()).collect()
    }

    #[test]
    fn slice_types() {
        let want = [0, 1, 1, 1, 2, 2, 3, 2, 3, 3, 4, 4];
        for (raw, &t) in want.iter().enumerate() {
            assert_eq!(slice_type(raw as u8), t, "raw {raw}");
        }
        assert_eq!(slice_type(12), 0);
        assert_eq!(slice_frames(200_000, 44100.0), 88200);
        assert_eq!(slice_frames(1000, 44100.0), 1000);
    }

    #[test]
    fn rule_graph_is_closed_and_acyclic() {
        // Every successor is a row or a result, and every path ends.
        let mut state = vec![0u8; RULES.len()];
        fn visit(i: usize, state: &mut [u8]) {
            assert_ne!(state[i], 1, "cycle through row {i}");
            if state[i] == 2 {
                return;
            }
            state[i] = 1;
            for next in [RULES[i].yes, RULES[i].no] {
                if next >= L {
                    assert!(((next - L) as usize) < SOUND_NAMES.len());
                } else {
                    visit(next as usize, state);
                }
            }
            state[i] = 2;
        }
        visit(0, &mut state);
        assert!(state.iter().all(|&s| s == 2), "unreachable rows");
    }

    #[test]
    fn first_rule_is_a_kick() {
        // [re:be0457]'s first rule: band 0 > 0.134251, band 33 <= 0.00022,
        // centroid <= 618.45197, band 1 <= 0.421437, band 13 <= 0.000611,
        // flatness <= 0.037943, band 10 <= 0.000397, zcr <= 7.785714.
        let mut f = Features { zero_crossings: 5.0, centroid: 500.0, flatness: 0.01, log_attack: -2.0, ..Features::default() };
        f.bands[0] = 0.2;
        f.bands[1] = 0.3;
        f.bands[33] = 0.0001;
        f.bands[13] = 0.0001;
        f.bands[10] = 0.0001;
        assert_eq!(matches_rule(&f), KICK);
    }

    /// Feature sets (f32, NaN included) and the sound a direct simulation of
    /// [re:be0457]'s instructions gives for them.
    #[rustfmt::skip]
    const SIMULATED: [(u8, [f32; 6], [f32; BANDS]); 12] = [
        (11, [11.363070487976074, -0.7627065777778625, 4279.65771484375, 379945.4375, 0.3553732931613922, 0.4908615052700043], [0.14060494303703308, 0.5485893487930298, f32::NAN, 0.007457993924617767, 0.00853660050779581, 0.0, 0.0, 0.0, 0.010998941957950592, 0.0, 0.01347300037741661, 0.0, 0.0, 0.0013397708535194397, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, f32::NAN, 0.0, 0.0, 0.0004351714451331645, 0.0, 3.0786653951508924e-05, 0.0, 0.0, 0.0, 5.091830826131627e-05, 0.0, 0.001728000002913177, 0.0, 0.0, 0.0, 2.7852205676026642e-05]),
        (7, [12.341473579406738, -1.148298740386963, 1582.7347412109375, 372621.65625, 47.52880859375, 0.1809631586074829], [0.0014959474792703986, 0.008162321522831917, 0.27684494853019714, 0.012967233546078205, 0.18768174946308136, 0.0, 0.0, 0.0, 0.008940362371504307, 0.0, 0.01760103553533554, 0.0, 0.0, f32::NAN, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, f32::NAN, 0.0, 6.048382056178525e-05, 0.0, 0.007843893952667713, 0.0, 0.0, 0.0, 8.007677934074309e-06, 0.0, 0.003003218211233616, 0.0, 0.0, 0.0, 3.428157287999056e-05]),
        (9, [452.1927490234375, -1.1619212627410889, 3335.753662109375, 272906.21875, 0.5313429832458496, 0.1305503100156784], [0.030839256942272186, 0.005100064445286989, 0.10604654252529144, 0.0002771308645606041, 0.006423638202250004, 0.0, 0.0, 0.0, f32::NAN, 0.0, 0.024135999381542206, 0.0, 0.0, 0.0007505100802518427, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, f32::NAN, 0.0, 0.0, 0.0, 0.0009834247175604105, 0.0, f32::NAN, 0.0, 0.0, 0.0, 7.166624709498137e-05, 0.0, 0.0004030261479783803, 0.0, 0.0, 0.0, 0.00015900000289548188]),
        (10, [69.58333587646484, -1.8592356443405151, 375.39166259765625, 289870.3125, 1.4008793830871582, 0.4196910262107849], [5.16876025358215e-05, 0.308665007352829, 0.005266656633466482, 0.003045732155442238, 0.10285283625125885, 0.0, 0.0, 0.0, 0.00909354630857706, 0.0, 0.013438213616609573, 0.0, 0.0, 0.0028211046010255814, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 6.299999949987978e-05, 0.0, 0.0013382920296862721, 0.0, 0.0, 0.0, 0.005622981581836939, 0.0, 1.7589578419574536e-05, 0.0, 0.0, 0.0, 3.534064308041707e-05]),
        (4, [10.290444374084473, -2.1731553077697754, 1548.6129150390625, 3028001.5, 0.7136574387550354, 0.09935248643159866], [0.1649390012025833, 0.6625741124153137, 0.4523182511329651, 0.044523999094963074, 0.1952332854270935, 0.0, 0.0, 0.0, 0.0016585119301453233, 0.0, 0.0008001577225513756, 0.0, 0.0, 0.0023983207065612078, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, f32::NAN, 0.0, 0.0, 9.682441304903477e-05, 0.0, 0.00012249808060005307, f32::NAN, 0.0, 0.0, 1.5125547179195564e-05, 0.0, 0.0031656797509640455, 0.0, 0.0, 0.0, 2.300000051036477e-05]),
        (1, [52.53559494018555, -1.5883842706680298, 537.5776977539062, 251084.328125, 2.992694139480591, 0.09145180881023407], [0.1543794423341751, 0.013907304033637047, 0.20874330401420593, 0.04657299816608429, 0.026099499315023422, 0.0, 0.0, 0.0, 0.0008555036038160324, 0.0, 0.0005844190018251538, 0.0, 0.0, 0.0008545636082999408, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 9.528048394713551e-05, 0.0, 0.019973989576101303, 0.0, 0.0, 0.0, 0.00021699999342672527, 0.0, 0.0018763454863801599, 0.0, 0.0, 0.0, 0.15733255445957184]),
        (5, [110.67539978027344, -1.1915557384490967, 5164.599609375, 1947189.375, 0.300833135843277, 0.3816869854927063], [0.11452843248844147, 0.5961740016937256, 0.016417227685451508, 0.01612996496260166, 0.00979200005531311, 0.0, 0.0, 0.0, 8.221196185331792e-05, 0.0, 0.012878753244876862, 0.0, 0.0, 0.001917000045068562, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0007389999809674919, 0.0, 0.00620545819401741, 0.0, 0.0, 0.0, 4.920332139590755e-05, 0.0, 3.670068326755427e-05, 0.0, 0.0, 0.0, 0.07930149137973785]),
        (3, [153.8493194580078, -1.1657536029815674, 8655.375, f32::NAN, 4.251392841339111, 0.05327616631984711], [0.1670096516609192, 0.800683856010437, 0.5127522945404053, 0.03977999836206436, 0.033972032368183136, f32::NAN, 0.0, 0.0, 0.0013884985819458961, 0.0, 0.005134926177561283, 0.0, 0.0, 0.00916100014001131, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.00025239866226911545, 0.0, 5.154141763341613e-05, 0.0, 0.0, 0.0, 1.700000029813964e-05, 0.0, 6.0207017668290064e-05, 0.0, 0.0, 0.0, 0.0005697451415471733]),
        (6, [f32::NAN, -1.230202555656433, 1106.7784423828125, 1112678.875, f32::NAN, 0.4267606735229492], [0.15367087721824646, f32::NAN, 0.00013100949581712484, 0.005756158847361803, 0.0025843041948974133, 0.0, 0.0, 0.0, 0.0006810000049881637, 0.0, 0.0008242878247983754, 0.0, 0.0, 7.200000254670158e-05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 3.018009192601312e-05, 0.0, 3.9184626075439155e-05, 0.0, 0.0, 0.0, 0.0033158818259835243, 0.0, 1.5134748537093401e-05, 0.0, 0.0, 0.0, 0.0003947269287891686]),
        (11, [406.404052734375, -1.8326059579849243, 3029.397216796875, 250755.875, 2.4504096508026123, 0.028639746829867363], [0.07229555398225784, 0.6024479866027832, 0.04219868406653404, 0.006643530912697315, 0.03298405930399895, 0.0, 0.0, 0.0, 0.0307852104306221, 0.0, 0.0027844354044646025, 0.0, 0.0, 0.006992911454290152, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.030378635972738266, 0.0, 0.0007389999809674919, 0.0, 0.0, 0.0, 0.00032022412051446736, 0.0, 5.199999941396527e-05, 0.0, 0.0, 0.0, 3.619240669650026e-05]),
        (6, [69.58333587646484, -2.1896209716796875, 4325.0654296875, f32::NAN, f32::NAN, 0.5930935144424438], [0.6056420207023621, 0.0025462082121521235, 0.12475501000881195, 0.0001534879265818745, 0.003716329112648964, 0.0, 0.0, 0.0, 0.0012710000155493617, 0.0, 0.0050978511571884155, 0.0, 0.0, 0.033829908818006516, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.5999999706982635e-05, 0.0, 0.0030926386825740337, 0.0, 0.0, 0.0, 6.374464192049345e-06, 0.0, 0.0025555237662047148, 0.0, 0.0, 0.0, 4.70000013592653e-05]),
        (4, [20.74728012084961, -2.6139986515045166, 4609.65380859375, 27231.83984375, 2.6851956844329834, 0.209570974111557], [0.13666445016860962, 0.03181295469403267, 0.03771200031042099, 0.050797589123249054, 0.0072209895588457584, 0.0, 0.0, 0.0, 0.0005875193746760488, 0.0, 0.0009399999980814755, 0.0, 0.0, 0.07103697210550308, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 3.8922662497498095e-05, 0.0, 0.0007389999809674919, 0.0, 0.0, 0.0, 0.00012404879089444876, 0.0, 2.3878561478340998e-05, 0.0, 0.0, 0.0, 2.499999936844688e-05]),
    ];

    #[test]
    fn rules_match_the_simulation() {
        for (i, (want, v, bands)) in SIMULATED.iter().enumerate() {
            let f = Features { zero_crossings: v[0], log_attack: v[1], centroid: v[2], spread_variance: v[3], skew: v[4], flatness: v[5], bands: *bands, ..Features::default() };
            assert_eq!(matches_rule(&f), *want, "set {i}");
        }
    }

    #[test]
    fn mel_filters() {
        let c = Classifier::new(1, 1, SR);
        let m = &c.mel;
        assert_eq!(m.weights.len(), BANDS);
        // 10 Hz .. ~15 kHz, increasing, each filter peaking at 2/(upper − lower).
        assert!((m.lower[0] - 10.0).abs() < 0.01, "{}", m.lower[0]);
        assert!((m.upper[BANDS - 1] - 14999.0).abs() < 1.0, "{}", m.upper[BANDS - 1]);
        assert!(m.lower.windows(2).all(|w| w[0] < w[1]));
        let peak = max(&m.weights[20]);
        assert!(peak > 0.0 && peak <= 2.0 / (m.upper[20] - m.lower[20]) * 1.0001);
    }

    #[test]
    fn kick_and_hat() {
        let k = kick(22050);
        let raw = classify(&k, None, SR, k.len());
        assert_eq!(slice_type(raw), 1, "kick classified as {}", SOUND_NAMES[raw as usize]);
        let h = hat(11025);
        let raw = classify(&h, None, SR, h.len());
        assert!(matches!(slice_type(raw), 2 | 3), "hat classified as {}", SOUND_NAMES[raw as usize]);
    }

    #[test]
    fn stereo_is_the_channel_mean() {
        let k = kick(8000);
        let mut a = Classifier::new(k.len(), 1, SR);
        let mut b = Classifier::new(k.len(), 2, SR);
        let mut c = Classifier::new(k.len(), 2, SR);
        a.process(&k, None);
        b.process(&k, Some(&k));
        c.process(&k, None);
        // (l + l)·0.5 = l; a missing right channel is the left one.
        assert_eq!(a.features(), b.features());
        assert_eq!(a.features(), c.features());
    }

    #[test]
    fn spreads_carry_over_between_slices() {
        // v1 computes the spreads for the first slice only: the same slice
        // again gives the same features, a longer one (more frames, zeroed
        // spreads) a non-finite skew and zero spread variance.
        let short = hat(4000);
        let long = hat(20000);
        let mut c = Classifier::new(long.len(), 1, SR);
        c.process(&short, None);
        let first = c.features().clone();
        c.process(&short, None);
        assert_eq!(&first, c.features());
        c.process(&long, None);
        assert!(!c.features().skew.is_finite(), "{}", c.features().skew);
        assert_eq!(c.features().spread_variance, 0.0);
    }

    #[test]
    fn silence_and_bad_input() {
        assert_eq!(classify(&[], None, SR, 0), UNKNOWN);
        assert_eq!(Classifier::new(10, 3, SR).process(&[0.5; 10], None), UNKNOWN);
        let mut c = Classifier::new(4096, 1, SR);
        c.process(&[0.0; 4096], None);
        let f = c.features();
        // Silent: no attack (end = the slice length), no energy.
        assert_eq!((f.attack_start, f.attack_end, f.end), (0.0, 1.0, 4096.0));
        assert!(f.bands.iter().all(|&b| b == 0.0));
    }

    /// The features of a fixed slice, pinned so changes show.
    #[test]
    fn pinned_features() {
        let n = 30000;
        let w = noise(n, 1);
        let x: Vec<f32> = (0..n)
            .map(|i| {
                let t = i as f32 / SR;
                ((std::f32::consts::TAU * 180.0 * t).sin() * 0.8 + w[i] * 0.3) * (-t * 12.0).exp()
            })
            .collect();
        let mut c = Classifier::new(n, 1, SR);
        let raw = c.process(&x, None);
        let f = c.features().clone();
        let close = |a: f32, b: f32| (a - b).abs() <= 1e-4 * b.abs().max(1e-3);
        assert_eq!(raw, PINNED_RAW);
        assert_eq!((f.attack_start, f.attack_end, f.end, f.frames), PINNED_TIMES);
        for (name, a, b) in [
            ("zero crossings", f.zero_crossings, PINNED[0]),
            ("log attack", f.log_attack, PINNED[1]),
            ("centroid", f.centroid, PINNED[2]),
            ("spread variance", f.spread_variance, PINNED[3]),
            ("skew", f.skew, PINNED[4]),
            ("flatness", f.flatness, PINNED[5]),
        ] {
            assert!(close(a, b), "{name}: {a} vs {b}");
        }
        for (i, (&a, &b)) in f.bands.iter().zip(&PINNED_BANDS).enumerate() {
            assert!((a - b).abs() <= 1e-4 * b.abs() + 1e-9, "band {i}: {a} vs {b}");
        }
    }

    const PINNED_RAW: u8 = SNARE;
    const PINNED_TIMES: (f32, f32, f32, usize) = (37.0, 552.0, 12367.0, 28);
    /// Zero crossings, log attack, centroid, spread variance, skew, flatness.
    const PINNED: [f32; 6] = [163.25, -1.9326314, 9107.206, 8708.828, 0.21253844, 0.71345365];
    #[rustfmt::skip]
    const PINNED_BANDS: [f32; BANDS] = [0.0075989594, 0.21029165, 0.74162024, 0.028466504, 0.0012558086, 0.00046464722, 0.0003364553, 0.00038300667, 0.0002784434, 0.00038797813, 0.0002389122, 0.00024170546, 0.0003428117, 0.0003942618, 0.00036744366, 0.00037450952, 0.0003475783, 0.0003561164, 0.0003036397, 0.0002019377, 0.00022236524, 0.0003065082, 0.00034709662, 0.00032159008, 0.00033387323, 0.00030192354, 0.00031893217, 0.00030026413, 0.0002792253, 0.00023799462, 0.0002542277, 0.00027110154, 0.0003145655, 0.00023588791, 0.00025347972, 0.00028655442, 0.00029692205, 0.00031162085, 0.00027939075, 0.0002739608];
}
