//! Einst on dreist-commons: the profile, the part loads, v1's effect nodes
//! and menu, and the engine and screen goldens (`tests/golden`; skipped
//! without the factory content, `UPDATE_GOLDEN=1` rewrites them).

use dreist_dsp::fx::{Ctx, Effect};
use dreist_formats::profile::{self, DocKind};

fn setup() {
    einst::install();
}

#[test]
fn profile_is_einsts() {
    setup();
    let p = profile::get();
    assert_eq!((p.product, p.save.project, p.theme), ("Einst", "einst", "Einst"));
    assert_eq!(profile::extensions_of(DocKind::Kit), ["einstkit", "geistkit", "dreistkit", "gkit"]);
    assert!(profile::open_extensions().contains(&"geistpad"));
    assert!(dreist_formats::paths::ui_dir().join("Skin/Styles/Einst.style").is_dir());
    assert!(dreist_formats::paths::ui_dir().join("Fonts/Einst-Icons.ttf").is_file());
    assert!(dreist_formats::paths::resources_dir().join("Effects/Gain").is_dir());
    assert_eq!(dreist_gui::skin::Skin::themes(), ["Einst", "Ember"]);
}

/// v1's menu offers v1's 32 types, Gain first, Tin Can Verb last.
#[test]
fn effect_menu_is_v1s() {
    setup();
    let menu: Vec<usize> = (0..6).flat_map(einst::effect_menu).collect();
    assert_eq!(menu.len(), 32);
    assert_eq!(einst::effect_menu(1).first().copied(), dreist_formats::params::effect_type_index("Gain"));
    for name in ["Maul", "Limiter"] {
        assert!(!menu.contains(&dreist_formats::params::effect_type_index(name).unwrap()));
    }
}

/// Effects build v1's nodes: Gain with v1's fast 2^x, the Envelope
/// Shaper with v1's hidden ports (WetMix 1; Linked pushed as 1).
#[test]
fn effects_build_v1_nodes() {
    setup();
    let mut gain = Effect::new(einst_dsp::GAIN, 44_100.0);
    gain.active = true;
    gain.mix = 1.0;
    gain.push_parameter(0, 0.6);
    let x = gain.node.as_mut().unwrap().ports()[0].value;
    let ones = vec![1.0f32; 64];
    let (mut l, mut r) = (vec![0.0; 64], vec![0.0; 64]);
    gain.process(&Ctx::new(44_100.0), [&ones, &ones], [&mut l, &mut r]);
    assert!((l[63] - einst_dsp::gain::Gain::target(x)).abs() < 1e-6, "{} vs {}", l[63], einst_dsp::gain::Gain::target(x));
    let mut es = Effect::new(einst_dsp::ENVELOPE_SHAPER, 44_100.0);
    assert_eq!(es.node.as_mut().unwrap().ports()[0].value, 1.0);
}

/// The editor's part loads (`project::load_part`, the kit browser) take v1 files.
#[test]
fn editor_part_loads_take_v1_files() {
    setup();
    const LAYER: &str = r#"<Layer name="Layer0" Ver="3.2" AudioFileName="Beat.wav" Envelope0Param3="-1" />"#;
    let dir = std::env::temp_dir().join(format!("einst-parts-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut model = dreist_editor::Model::new(dreist_editor::project::new_document());
    let pad = dir.join("Q.geistpad");
    std::fs::write(&pad, format!(r#"<root><Pad name="Pad" Ver="2" PadMode="Random">{LAYER}</Pad></root>"#)).unwrap();
    assert!(dreist_editor::project::load_part(&mut model.doc, None, dreist_editor::project::Part::Pad(1, 3), &pad));
    let p = model.doc.child("Engine", "Engine1").and_then(|e| e.child("Pad", "Pad3")).unwrap();
    assert_eq!(p.attr("PadMode"), Some("Random"));
    assert_eq!(p.children_of("Layer").next().and_then(|l| l.version), Some(3300));
    let kit = dir.join("K.geistkit");
    std::fs::write(&kit, format!(r#"<root><Pad name="Pad5" Ver="2">{LAYER}</Pad></root>"#)).unwrap();
    assert!(dreist_editor::browser::load_kit_into_doc(&mut model, &kit));
    let e = model.doc.child("Engine", &format!("Engine{}", model.engine)).unwrap();
    assert!(e.child("Pad", "Pad5").and_then(|p| p.children_of("Layer").next()).is_some_and(|l| l.version == Some(3300)));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn engine_renders_match_einsts_goldens() {
    setup();
    dreist_engine::harness::check_scenarios("engine");
}

#[test]
fn screens_match_einsts_goldens() {
    setup();
    dreist_editor::screens::check_screens("screens");
}

/// Dreist's nodes take v1's fast 2^x once Einst is installed: Stereo
/// Width settles at v1's Gain target.
#[test]
fn shared_nodes_use_v1s_fast_exp2() {
    setup();
    assert_eq!(dreist_dsp::fx::exp2(1.51), einst_dsp::fast_exp2(1.51));
    let mut s = Effect::new(einst_dsp::STEREO_WIDTH, 44_100.0);
    s.active = true;
    s.mix = 1.0;
    let node = s.node.as_mut().unwrap();
    node.ports()[0].value = 1.3;
    let ones = vec![1.0f32; 64];
    let (mut l, mut r) = (vec![0.0; 64], vec![0.0; 64]);
    node.process(&Ctx::new(44_100.0), [&ones, &ones], [&mut l, &mut r]);
    assert!((l[63] - einst_dsp::gain::Gain::target(1.3)).abs() < 1e-3, "{} vs {}", l[63], einst_dsp::gain::Gain::target(1.3));
}

/// v1's envelope: a decay ending at zero sustain stops at once (v2 fades
/// over 32 samples).
#[test]
fn envelope_stops_at_zero_sustain() {
    setup();
    use dreist_dsp::envelope::{Envelope, EnvelopeParams};
    let mut e = Envelope::new(44_100.0);
    let p = EnvelopeParams { attack: 0.0, decay: 0.2, sustain: 0.0, hold: 0.0, release: 0.2, attack_curve: 0.5, decay_curve: 0.6, release_curve: 0.5 };
    e.initialize(&p, false, 1.0);
    e.note_on();
    let mut buf = vec![1.0f32; 8192];
    let mut ended_at = None;
    for b in 0..20 {
        if !e.process(&mut buf) {
            ended_at = Some(b);
            break;
        }
    }
    assert!(ended_at.is_some());
    assert!(buf.iter().rev().take(32).all(|v| *v == 0.0));
    assert!(dreist_dsp::rules::get().envelope_v1_ends && dreist_dsp::rules::get().loop_v1);
}
