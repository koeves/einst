//! Tooltip value texts. v1's providers ([re:d1702c] and the rest below,
//! each "<display name>: <value>" or, for the pitch / pre-delay /
//! timestretch ones, "<display name> :<value>") are what the layer, pad,
//! mixer, header and effect controls show ([`v1_layer_tooltip`]).
//!
//! `Conv` is the v2 conversion set ([re:0c929e], [re:48679f]: name, ": ",
//! value, space, suffix) kept only for the dormant v2-only transmod panel
//! and controls v1 does not have.

/// A number typed into a value box (or read from a text file): finite
/// only, since Rust's parser also takes "inf" and "NaN".
pub fn number(s: &str) -> Option<f32> {
    s.trim().parse::<f32>().ok().filter(|v| v.is_finite())
}

/// [`number`] in double precision.
pub fn number64(s: &str) -> Option<f64> {
    s.trim().parse::<f64>().ok().filter(|v| v.is_finite())
}

/// C `%.Nd`: at least `n` digits, sign only when negative.
fn digits(v: i64, n: usize) -> String {
    let s = format!("{:0n$}", v.unsigned_abs());
    if v < 0 { format!("-{s}") } else { s }
}

/// A value converter: its value text and suffix.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Conv {
    /// Percent [re:ae320e]: `%.3d` of ⌊100v⌋, "%".
    Percent,
    /// Depth [re:9d4b8d]: `%+03d` of ⌊200v − 100⌋, "%".
    Depth,
    /// On / off [re:7c2c57].
    OnOff,
    /// Degrees [re:663133]: `%03d` of ⌊360v⌋, "°".
    Degrees,
    /// AD envelope stages ([re:aa1c09]
    /// [re:b0d075]): `%0.2f` of 5.5v, "secs".
    AdSecs,
    /// Sequencer step count ([re:d967c0]): `%d` of ⌊15v + 1⌋.
    Steps,
}

impl Conv {
    pub fn value(self, v: f32) -> String {
        match self {
            Conv::Percent => digits((v * 100.0).floor() as i64, 3),
            Conv::Depth => format!("{:+03}", (v * 200.0 - 100.0).floor() as i64),
            Conv::OnOff => if v >= 0.5 { "On" } else { "Off" }.into(),
            Conv::Degrees => format!("{:03}", (v * 360.0).floor() as i64),
            Conv::AdSecs => format!("{:.2}", v * 5.5),
            Conv::Steps => format!("{}", (15.0 * v + 1.0).floor() as i64),
        }
    }

    pub fn suffix(self) -> &'static str {
        match self {
            Conv::Percent | Conv::Depth => "%",
            Conv::AdSecs => "secs",
            Conv::Degrees => "°",
            _ => "",
        }
    }

    /// Value, space, suffix.
    pub fn text(self, v: f32) -> String {
        format!("{} {}", self.value(v), self.suffix())
    }

    /// The value text under `name`.
    pub fn named(self, name: &str, v: f32) -> String {
        let mut s = name.to_string();
        if !s.is_empty() && !s.contains(':') {
            s.push_str(": ");
        }
        s + &self.text(v)
    }

    /// [re:7274d9] (type, param).
    pub fn modulator(t: dreist_formats::params::ModulatorType, param: &str) -> Conv {
        use dreist_formats::params::ModulatorType as M;
        match (t, param) {
            (_, "Sync" | "Drunk" | "Reverse" | "Trig") => Conv::OnOff,
            (M::LFO | M::MorphingLFO | M::SampleAndHold, "Phase") => Conv::Degrees,
            (M::MorphingLFO, "Swing") | (M::ADEnvelope, "AttackCurve" | "DecayCurve") => Conv::Depth,
            (M::ADEnvelope, "Attack" | "Decay") => Conv::AdSecs,
            (M::MiniSequencer, "NumberOfSteps") => Conv::Steps,
            _ => Conv::Percent,
        }
    }
}

/// A transmod knob's text ([re:7eb9d2] and the
/// other panels): with a source shown the description is "<source> ->
/// <name>:\n", and a non-zero depth adds " -> " and the modulated value.
pub fn transmod_text(conv: Conv, name: &str, v: f32, source: Option<(&str, f32)>) -> String {
    match source {
        Some((src, d)) => {
            let mut t = conv.named(&format!("{src} -> {name}:\n"), v);
            if d.abs() >= 1e-6 {
                t.push_str(" -> ");
                t.push_str(&conv.text((v + d).clamp(0.0, 1.0)));
            }
            t
        }
        None => conv.named(name, v),
    }
}

/// [re:d1702c].
pub fn bool_text(name: &str, on: bool) -> String {
    format!("{name}: {}", if on { "On" } else { "Off" })
}

/// [re:518285] ("N: -3.52dB", no space).
pub fn db15_text(name: &str, v: f32) -> String {
    format!("{name}: {}", db15(v))
}

pub fn db15(v: f32) -> String {
    let c = 2.0 * v * v * v;
    let db = if c.abs() < 1e-6 { -1000.0 } else { (20.0 * c.log10()).min(6.0) };
    if db >= -100.0 { format!("{db:.2}dB") } else { "-∞dB".into() }
}

/// [re:8f2f90].
pub fn long_percent_text(name: &str, v: f32) -> String {
    format!("{name}: {:03}%", (v * 100.0) as i64)
}

/// [re:38c895].
pub fn depth_text(name: &str, v: f32) -> String {
    format!("{name}: {:+04}%", ((2.0 * v - 1.0) * 100.0) as i64)
}

/// [re:0017d0].
pub fn midi_text(name: &str, v: f32) -> String {
    format!("{name}: {:03}", (v * 128.0) as i64)
}

/// [re:0e7e30].
pub fn pan_text(v: f32) -> String {
    let p = (v.clamp(0.0, 1.0) * 200.0 - 100.0) as i64;
    match p {
        0 => "C".into(),
        p if p < 0 => format!("{:3}% L", -p),
        p => format!("{p:3}% R"),
    }
}

/// [re:474b5f] (natural log, kept).
#[allow(dead_code)]
pub fn db_unity(v: f32) -> String {
    let db = if v > 1.9287e-22 { 20.0 * v.ln() } else { -1000.0 };
    if db < -100.0 { "-oodB".into() } else { format!("{db:.2}dB") }
}

/// [re:34f000]: "N: Off!" at 0, else "N: %03d%".
pub fn off_percent_text(name: &str, v: f32) -> String {
    if v <= 0.0 { format!("{name}: Off!") } else { format!("{name}: {:03}%", (v * 100.0) as i64) }
}

/// [re:505814]: "N :%+0.02f semitones" of the rounded pitch (24v − 12).
pub fn coarse_pitch_text(name: &str, v: f32) -> String {
    let f = v * 24.0;
    let st = (if f < 0.0 { f - 0.5 } else { f + 0.5 }) as i32 as f32 - 12.0;
    format!("{name} :{st:+.2} semitones")
}

/// [re:12fd49]: "N :%+03d cents" of 100 (2v − 1).
pub fn fine_pitch_text(name: &str, v: f32) -> String {
    format!("{name} :{:+03} cents", ((v + v - 1.0) as f64 * 100.0) as i64)
}

/// [re:0ea591]: v³ · 1000 ms: "N : %03d msec", from a second "N :%d sec".
pub fn predelay_text(name: &str, v: f32) -> String {
    let ms = (v * v * v * 1000.0) as i64;
    if ms < 1000 { format!("{name} : {ms:03} msec") } else { format!("{name} :{} sec", ms / 1000) }
}

/// [re:622e65]: 440 · 2^(12v − 6): "N : %0.02f kHz" from 1 kHz, else Hz.
pub fn cutoff_text(name: &str, v: f32) -> String {
    let f = 440.0 * 2f32.powf(v * 12.0 - 6.0);
    if f >= 1000.0 { format!("{name} : {:.2} kHz", f / 1000.0) } else { format!("{name} : {f:.2} Hz") }
}

/// [re:dd87f2]: the stretch percentage, "N :%03d%".
pub fn timestretch_text(name: &str, v: f32) -> String {
    let pct = if v > 0.5 {
        let x = v + v - 1.0;
        (x * x * x * 3.0 + 1.0) * 100.0
    } else {
        let x = 1.0 - (v + v);
        ((1.0 - x * x * x) * 0.9 + 0.1) * 100.0
    };
    format!("{name} :{:03}%", pct as i64)
}

/// A layer setup / envelope / timestretch or pad control's tooltip by its
/// v1 provider ([re:51e937], [re:ac1ae4], [re:917012], [re:a6bbec]);
/// `None` for controls v1 gives none.
pub fn v1_layer_tooltip(name: &str, label: &str, v: f32, on: bool) -> Option<String> {
    Some(match name {
        "GainKnob" | "Gain" => db15_text(label, v),
        "PanKnob" | "Pan" => format!("{label}: {}", pan_text(v)),
        "RevButton" | "PhaseButton" | "FilterActiveButton" | "TrackPitchButton" | "TSActiveButton" | "TrackBPMButton" => bool_text(label, on),
        "ResKnob" | "DriveKnob" | "Send1Knob" | "Send2Knob" | "Send3Knob" | "Send4Knob" | "Send1" | "Send2" | "Send3" | "Send4" => long_percent_text(label, v),
        "TuneKnob" => coarse_pitch_text(label, v),
        "FineKnob" => fine_pitch_text(label, v),
        "V2C" | "V2P" | "AmpEnvDepth1Knob" | "AmpEnvDepth2Knob" | "FreeEnvDepth1Knob" | "FreeEnvDepth2Knob" => depth_text(label, v),
        "PreDelayKnob" => predelay_text(label, v),
        "CutoffKnob" => cutoff_text(label, v),
        "AmountKnob" => timestretch_text(label, v),
        "V2A" => off_percent_text(label, v),
        "FindBPMButton" => label.to_string(),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversion_texts() {
        crate::app::test_setup();
        assert_eq!(Conv::Percent.named("Mix", 0.5), "Mix: 050 %");
        assert_eq!(Conv::Depth.named("Variance", 0.75), "Variance: +50 %");
        assert_eq!(Conv::Depth.value(0.5), "+00");
        assert_eq!(Conv::OnOff.named("Reverse", 1.0), "Reverse: On ");
        // v1's layer providers.
        assert_eq!(cutoff_text("Layer filter cutoff", 0.5), "Layer filter cutoff : 440.00 Hz");
        assert_eq!(cutoff_text("C", 1.0), "C : 28.16 kHz");
        assert_eq!(coarse_pitch_text("Layer coarse tune", 0.75), "Layer coarse tune :+6.00 semitones");
        assert_eq!(fine_pitch_text("Layer fine tune", 0.55), "Layer fine tune :+10 cents");
        assert_eq!(predelay_text("Pre delay", 0.5), "Pre delay : 125 msec");
        assert_eq!(predelay_text("Pre delay", 1.0), "Pre delay :1 sec");
        assert_eq!(timestretch_text("Timestretch amount", 0.5), "Timestretch amount :100%");
        assert_eq!(off_percent_text("Velocity to Amplitude", 0.0), "Velocity to Amplitude: Off!");
        assert_eq!(v1_layer_tooltip("PanKnob", "Pan", 0.5, false).as_deref(), Some("Pan: C"));
        assert_eq!(v1_layer_tooltip("FilterModeKnob", "Mode", 0.5, false), None);
        assert_eq!(transmod_text(Conv::Percent, "Cutoff", 0.5, Some(("LFO 1", 0.25))), "LFO 1 -> Cutoff:\n050 % -> 075 %");
        assert_eq!(pan_text(0.25), " 50% L");
        assert_eq!(db15(0.0), "-∞dB");
        assert_eq!(depth_text("Scale", 0.75), "Scale: +050%");
        assert_eq!(digits(-5, 3), "-005");
        use dreist_formats::params::ModulatorType as M;
        assert_eq!(Conv::modulator(M::LFO, "Phase").named("Phase", 0.25), "Phase: 090 °");
        assert_eq!(Conv::modulator(M::ADEnvelope, "Attack").named("Attack", 1.0), "Attack: 5.50 secs");
        assert_eq!(Conv::modulator(M::MiniSequencer, "NumberOfSteps").named("Number Of Steps", 1.0), "Number Of Steps: 16 ");
    }
}
