//! The frequency-response curves the EQ and filter effect editors plot,
//! as v1's nodes compute them for display (Parametric / DJ EQ
//! [re:1afe17], Filter [re:d9b038], Vari-EQ [re:2c171a]):
//!
//! - 256 points, x = i/256 (0..1), at 20·10^(3x) Hz (20 Hz..20 kHz,
//!   [re:29d627]);
//! - the response of the shown biquad sections at z = e^{jω}, ω = 2π·f/fs
//!   ([re:a020f8]; complex product over the sections, unity when none);
//! - y = (clamp(20·log10|H|, −20, 20) + 20)/40 ([re:4a1f8b]), so 0 dB is
//!   the middle and ±20 dB the edges.
//!
//! The arithmetic follows v1's precision: the frequency and ω in single
//! precision, the response in double.

/// Points per curve.
pub const POINTS: usize = 256;

/// One biquad section: `[b0, b1, b2, a1, a2]` (a0 = 1).
pub type Section = [f64; 5];

/// The x of point `i` (0..1).
pub fn x_at(i: usize) -> f32 {
    i as f32 * (1.0 / POINTS as f32)
}

/// `ConvertToUnitsFromVal01X`: x (0..1) → Hz.
pub fn hz(x: f32) -> f32 {
    (10f64.powf(x as f64 * 3.0) * 20.0) as f32
}

/// `ConvertToVal01FromUnitsY`: dB → y (0..1).
pub fn y_of_db(db: f32) -> f32 {
    let a = if db <= -20.0 { -20.0 } else { db };
    let b = if a < 20.0 { a } else { 20.0 };
    let v = (b + 20.0) * 0.025;
    let v = if v <= 0.0 { 0.0 } else { v };
    if v >= 1.0 { 1.0 } else { v }
}

/// `Hz`: one section's response at `w` (re, im).
fn response(c: &Section, (wr, wi): (f64, f64)) -> (f64, f64) {
    let w2i = wr * wi + wr * wi;
    let w2r = wr * wr - wi * wi;
    let di = c[3] * wi + c[4] * w2i;
    let dr = c[4] * w2r + c[3] * wr + 1.0;
    let ni = wi * c[1] + w2i * c[2];
    let nr = c[2] * w2r + c[1] * wr + c[0];
    let m = di * di + dr * dr;
    ((nr * dr + ni * di) / m, (dr * ni - nr * di) / m)
}

/// The curve (y per point) of `sections` in series at `sample_rate`.
pub fn curve(sections: &[Section], sample_rate: f32) -> [f32; POINTS] {
    let inv_sr = 1.0 / sample_rate;
    std::array::from_fn(|i| {
        let th = (inv_sr * -6.2831855 * hz(x_at(i))) as f64;
        // e^{−jω}, inverted: e^{jω}.
        let (c, s) = (th.cos(), th.sin());
        let m = s * s + c * c;
        let w = (c / m, -s / m);
        let (mut re, mut im) = (1.0f64, 0.0f64);
        for sec in sections {
            let (hr, hi) = response(sec, w);
            let t = re * hi;
            re = re * hr - im * hi;
            im = im * hr + t;
        }
        y_of_db(((im * im + re * re).sqrt().log10() * 20.0) as f32)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axes_span_20hz_to_20khz_and_40db() {
        assert_eq!(x_at(0), 0.0);
        assert_eq!(x_at(128), 0.5);
        assert!((hz(0.0) - 20.0).abs() < 1e-4);
        assert!((hz(1.0 / 3.0) - 200.0).abs() < 1e-2);
        assert!((hz(1.0) - 20000.0).abs() < 1e-1);
        assert_eq!(y_of_db(0.0), 0.5);
        assert_eq!(y_of_db(-30.0), 0.0);
        assert_eq!(y_of_db(30.0), 1.0);
        assert!((y_of_db(10.0) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn no_sections_is_flat_at_0db() {
        assert!(curve(&[], 44100.0).iter().all(|y| *y == 0.5));
        // A pass-through section too.
        assert!(curve(&[[1.0, 0.0, 0.0, 0.0, 0.0]], 48000.0).iter().all(|y| (*y - 0.5).abs() < 1e-6));
    }

    #[test]
    fn a_gain_section_shifts_the_curve() {
        // b0 = 10^(6/20): +6 dB everywhere.
        let g = 10f64.powf(6.0 / 20.0);
        assert!(curve(&[[g, 0.0, 0.0, 0.0, 0.0]], 44100.0).iter().all(|y| (*y - 0.65).abs() < 1e-5));
        // Two in series add up (+12 dB).
        assert!(curve(&[[g, 0.0, 0.0, 0.0, 0.0]; 2], 44100.0).iter().all(|y| (*y - 0.8).abs() < 1e-5));
    }

    /// The point nearest `f` Hz.
    fn at_hz(c: &[f32; POINTS], f: f32) -> f32 {
        c[((f / 20.0).log10() / 3.0 * POINTS as f32).round() as usize]
    }

    #[test]
    fn eq_bands_plot_their_peak_or_nothing_when_off() {
        use dreist_dsp::fx::{ports, ports_from};
        let mut p: Vec<f32> = ports_from(ports::PARAMETRIC_EQ).iter().map(|p| p.value).collect();
        // LMF at 1 kHz, +12 dB, Q 1 — but switched off: flat.
        p[5] = 1000f32.log2() - 1.0;
        p[6] = (12.0f64 / 8.0).atan() as f32 / 1.19029;
        p[4] = 0.0;
        p[7] = 0.0;
        assert!(crate::ssl_eq::display_part(&p, 1, 44100.0).iter().all(|y| *y == 0.5));
        p[7] = 1.0;
        let c = crate::ssl_eq::display_part(&p, 1, 44100.0);
        assert!((at_hz(&c, 1000.0) - 0.8).abs() < 0.01, "{}", at_hz(&c, 1000.0));
        assert!((c[0] - 0.5).abs() < 0.01 && (c[POINTS - 1] - 0.5).abs() < 0.01);
        assert!(c.iter().all(|y| *y <= at_hz(&c, 1000.0) + 1e-3));
        // 0 dB gain: flat.
        p[6] = 0.0;
        assert!(crate::ssl_eq::display_part(&p, 1, 44100.0).iter().all(|y| (*y - 0.5).abs() < 1e-4));
    }

    #[test]
    fn filter_parts_are_low_and_high_pass() {
        use dreist_dsp::fx::{ports, ports_from};
        let mut p: Vec<f32> = ports_from(ports::FILTER).iter().map(|p| p.value).collect();
        // HP at 200 Hz, LP at 2 kHz, both on.
        p[0] = 1.0;
        p[1] = 200f32.log2() - 1.0;
        p[2] = 1.0;
        p[3] = 2000f32.log2() - 1.0;
        let lp = crate::ssl_filter::display_part(&p, 0, 48000.0);
        let hp = crate::ssl_filter::display_part(&p, 1, 48000.0);
        // −3 dB (Q 1/√2) at the corner, 0 dB in the pass band, falling off outside.
        assert!((at_hz(&lp, 2000.0) - y_of_db(-3.0)).abs() < 0.01);
        assert!((at_hz(&hp, 200.0) - y_of_db(-3.0)).abs() < 0.01);
        assert!((at_hz(&lp, 50.0) - 0.5).abs() < 1e-3 && at_hz(&lp, 15000.0) < 0.1);
        assert!((at_hz(&hp, 10000.0) - 0.5).abs() < 1e-3 && at_hz(&hp, 25.0) < 0.1);
        p[2] = 0.0;
        assert!(crate::ssl_filter::display_part(&p, 0, 48000.0).iter().all(|y| *y == 0.5));
    }

    #[test]
    fn vari_eq_plots_its_mode() {
        use dreist_dsp::fx::{ports, ports_from};
        let mut p = ports_from(ports::VARI_EQ);
        // Bell, Freq 0 (2^(0 + 1)·400 = 800 Hz), Q 1, +9 dB.
        p[0].value = 0.0;
        p[1].value = 0.0;
        p[2].value = 1.0;
        p[3].value = 9.0;
        let c = crate::vari_eq::display(&p, 44100.0);
        assert!((at_hz(&c, 800.0) - y_of_db(9.0)).abs() < 0.01, "{}", at_hz(&c, 800.0));
        // Gain 0: flat.
        p[3].value = 0.0;
        assert!(crate::vari_eq::display(&p, 44100.0).iter().all(|y| (*y - 0.5).abs() < 1e-4));
    }
}
