//! Synthetic Geist v1 documents (no v1 user files exist): built from the
//! serializer field lists in `docs/re/loading-v1.md`, converted with
//! `Document::to_geist2` and loaded by Dreist's formats and engine.

use std::path::{Path, PathBuf};

use einst_formats::{FileKind, v1::Document};
use geist_engine::{Geist, RenderOptions};
use geist_formats::Node;

/// A factory metronome click from the v1 install, used as every layer's sample.
fn samples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/geist1/System/Metronomes/Default")
}

/// A v1 layer with every attribute `Layer::SerializeAsXML` writes (`Ver` 3.2).
fn layer(name: &str, ver: &str, extra: &str) -> String {
    format!(
        r#"<Layer name="{name}" Ver="{ver}" LockState="false" Gain="0.793701" Pan="0.5" Mute="false" Solo="false" OutputLocation="Parent"
            AuxSend0="0" AuxSend1="0" AuxSend2="0" AuxSend3="0" IsReversed="false" PhasePosition="1" UsesTimestretch="false" TSTracksBPM="false"
            SourceBPM="120" DestinationBPM="120" V2C="0.5" V2P="0.5" EnvTracksPitch="false" LayerGain="0.793701" LayerPan="0.5" CoarseTune="0.5"
            FineTune="0.5" PreDelayTime="0" Cutoff="1" Resonance="0" FilterType="0" FilterDrive="0.3" FilterAlgorithm="0.3" FilterActive="0"
            Envelope0Param0="0" Envelope0Param1="0.02" Envelope0Param2="1" Envelope0Param3="-1" Envelope0Param4="0.001" Envelope0Param5="0.5" Envelope0Param6="0.5" Envelope0Param7="0.5"
            Envelope1Param0="0" Envelope1Param1="0.02" Envelope1Param2="1" Envelope1Param3="-1" Envelope1Param4="0.001" Envelope1Param5="0.5" Envelope1Param6="0.5" Envelope1Param7="0.5"
            EnvelopeType0="ADSR" EnvelopeType1="ADSR" CutMode="Self" PlaybackMode="One shot" TSMode="Default" AudioFileName="Beat.wav" {extra}>
            <EffectChain name="EffectChain" FXVer="2.1" IsActive="true" ChainMix="1"><FixedString name="ChainPresetName" string="Default" /></EffectChain>
            <LayerVelocitySplit name="VelocitySplit" Ver="2" LowPoint="0" LowTurn="0" HighTurn="127" Highest="127" />
            <AudioFileMarkers name="StartEndMarkers" FileMarkerVersion="2" StartSample="0" EndSample="4000" SampleLength="4000" />
            <AudioFileMarkers name="LoopMarkers" FileMarkerVersion="2" StartSample="0" EndSample="4000" SampleLength="4000" />
            <FixedString name="LayerName" string="Click" />
            <LayerModulationRouting name="ModRouting0" Ver="1" ModulationDepth="0.5" ModulationDestination="No destination" />
            <LayerModulationRouting name="ModRouting1" Ver="1" ModulationDepth="0.5" ModulationDestination="No destination" />
            <LayerModulationRouting name="ModRouting2" Ver="1" ModulationDepth="0.5" ModulationDestination="No destination" />
            <LayerModulationRouting name="ModRouting3" Ver="1" ModulationDepth="0.5" ModulationDestination="No destination" />
        </Layer>"#
    )
}

fn pad(name: &str, layers: &str) -> String {
    format!(
        r#"<Pad name="{name}" Ver="2" LockState="false" Gain="0.793701" Pan="0.5" Mute="false" Solo="false" OutputLocation="Parent"
            AuxSend0="0" AuxSend1="0" AuxSend2="0" AuxSend3="0" V2A="1" PadMode="All layers">
            <EffectChain name="EffectChain" FXVer="2.1" IsActive="true" ChainMix="1">
                <Effect name="Effect0" EffectVersion="2" IsActive="true" Mix="1" Level="1" Parameter0="0.5" Parameter1="0" Parameter2="0" Parameter3="0" Parameter4="0" Parameter5="0" EffectType="Gain">
                    <FixedString name="PresetName" string="Default" />
                </Effect>
                <FixedString name="ChainPresetName" string="Default" />
            </EffectChain>
            {layers}
            <FixedString name="PadName" string="Click" />
        </Pad>"#
    )
}

/// A 16-step pattern, track 0 on every quarter note.
fn pattern(name: &str) -> String {
    let steps: String = (0..16).step_by(4).map(|i| format!(r#"<Step name="S{i}" Ver="2" State="true" Velocity="0.9" Ampenvdepth1="0.7" FX1Active="1" />"#)).collect();
    format!(
        r#"<StepPattern name="{name}" Ver="2" LockState="false" PatternSteps="16" MonoModeActive="false" SyncMode="Slave" Resolution="1/16">
            <FixedString name="PatternName" string="Beat" />
            <Track name="Track0" Ver="2.2" LockState="false" Gain="0.793701" Pan="0.5" Mute="false" Solo="false" NumberOfSteps="16" VelocityRand="0.6" AmpenvdepthRand="0.7">{steps}</Track>
        </StepPattern>"#
    )
}

fn engine(name: &str) -> String {
    format!(
        r#"<Engine name="{name}" Ver="2" Gain="0.793701" Pan="0.5" Mute="false" Solo="false" OutputLocation="Parent" AuxSend0="0" AuxSend1="0" AuxSend2="0" AuxSend3="0"
            Pitch="0.5" TempoNumerator="1" TempoDenominator="1" SwingTemplateDepth="0" EnginePadMode="Normal">
            <EffectChain name="EffectChain" FXVer="2.1" IsActive="true" ChainMix="1" />
            <SwingTemplate name="SwingTemplate" Ver="2" NumberOfSteps="16"><FixedString name="SwingTemplateName" string="Default" /></SwingTemplate>
            <EngineState name="EngineState" Ver="2" PatternPower="true" CurrentPattern="0" CurrentPad="0" CurrentStep="0" FullLevels="127" FullLevelEnabled="false"
                NoteEraseEnabled="false" NoteRepeatEnabled="false" NoteRepeatSynced="true" />
            {pattern}
            {pad}
        </Engine>"#,
        pattern = pattern("StepPattern0"),
        pad = pad("Pad0", &layer("Layer0", "3.2", "")),
    )
}

fn project(engine0: &str) -> String {
    format!(
        r#"<root StateVersion="2.1" SendMIDINotes="true" SendMIDICCs="true" CurrentEngine="0" PreviewVolume="0.79" PreviewOutput="Master">
            <AudioFileManager name="FileManager" Ver="2" NumberOfSamples="1"><VString name="AudioFile0Path" string="Beat.wav" /></AudioFileManager>
            <MasterChannel name="MasterChannel" Ver="2" Gain="0.793701" Pan="0.5" Mute="false" Solo="false" OutputLocation="Master" Pitch="0.5">
                <EffectChain name="EffectChain" FXVer="2.1" IsActive="true" ChainMix="1" />
            </MasterChannel>
            <BindingMap name="Bindings" />
            <SceneManager name="Scenes" CurrentScene="0" EditorBase="0"><Scene name="Scene0" used="false"><FixedString name="name" string="" /></Scene></SceneManager>
            <SongSequencer name="Song" SongActive="false" SongLoopEnabled="false" SongLoopStart="0" SongLoopEnd="16" SongEditorZoom="16" SongEditorStart="0">
                <Song name="Song"><SongTrack name="Track0" /></Song>
            </SongSequencer>
            <TimeInfo name="TimeInfo" Tempo="120" TSN="4" TSD="4" Metro="false" Quantize="false" />
            <Sampler name="Sampler" version="2" recordsource="Input 1" inputgain="1" outputgain="1" />
            <Metronome name="Metronome" Ver="2" Gain="0.793701" Pan="0.5" Mute="true" Solo="false" OutputLocation="Master" />
            {engine0}
            <GlobalAux name="GlobalAux0" Ver="2" Gain="0.793701" Pan="0.5" Mute="false" Solo="false" OutputLocation="Buss 1" IsPreFade="false" />
        </root>"#
    )
}

fn convert(kind: FileKind, text: &str) -> Node {
    Document::from_str(kind, text).unwrap().to_geist2().unwrap()
}

/// Render `secs` seconds; return the peak.
fn render(doc: &Node, secs: f32) -> f32 {
    let sr = 44_100.0;
    let mut g = Geist::from_document(doc, &samples(), sr, RenderOptions::default(), &mut Default::default()).expect("Dreist loads the converted project");
    g.start();
    let (mut l, mut r) = (vec![0.0; 512], vec![0.0; 512]);
    let mut peak = 0.0f32;
    for _ in 0..(secs * sr / 512.0) as usize {
        g.process(&mut l, &mut r);
        for v in l.iter().chain(r.iter()) {
            assert!(v.is_finite(), "non-finite sample");
            peak = peak.max(v.abs());
        }
    }
    peak
}

fn first<'a>(n: &'a Node, class: &str) -> &'a Node {
    n.children_of(class).next().unwrap_or_else(|| panic!("no <{class}> in <{}>", n.class))
}

#[test]
fn project_converts_loads_and_plays() {
    let doc = convert(FileKind::Project, &project(&engine("Engine0")));
    assert_eq!(doc.version, Some(3000));
    assert!(doc.attr("StateVersion").is_none());
    assert!(doc.children_of("BindingMap").next().is_none());
    assert_eq!(first(&doc, "GlobalAux").attr("OutputLocation"), Some("Aux 1"));
    let p = geist_formats::Project::from_node(doc.clone()).unwrap();
    assert_eq!(p.audio_files, ["Beat.wav"]);
    let layer = &p.engines[0].pads[0].layers[0];
    assert_eq!(layer.raw.version, Some(3300));
    assert_eq!(layer.audio_file.as_deref(), Some("Beat.wav"));
    assert_eq!(layer.name.as_deref(), Some("Click"));
    // 3.2 layers take Geist2's < 3300 branch: envelopes and switches renamed; a hold of -1 stays
    // (v1 recomputes it on load, and so does Dreist).
    for k in ["Ver", "V2C", "V2P", "UsesTimestretch", "TSMode", "Envelope0Param0", "EnvelopeType0"] {
        assert!(layer.raw.attr(k).is_none(), "{k} left over");
    }
    assert_eq!(layer.raw.f32("AmpEnvDecay"), Some(0.02));
    assert_eq!(layer.raw.f32("FreeEnvHold"), Some(-1.0));
    assert!(geist_engine::hold::needs_reset_on_load(&layer.raw));
    assert_eq!(layer.raw.f32("TimestretchActive"), Some(0.0));
    assert_eq!(layer.raw.f32("Cutoff_Velocity"), Some(0.0));
    assert!(layer.raw.children_of("LayerModulationRouting").next().is_none());
    // Steps keep Geist2's lanes only; the track keeps modifiers of those lanes.
    let track = &p.engines[0].patterns[0].tracks[0];
    assert_eq!(track.steps.len(), 4);
    assert_eq!(track.steps[1].velocity, Some(0.9));
    assert!(track.steps[0].raw.attr("Ampenvdepth1").is_none());
    assert_eq!(track.raw.attr("VelocityRand"), Some("0.6"));
    assert!(track.raw.attr("AmpenvdepthRand").is_none());
    assert!(render(&doc, 3.0) > 1e-3, "silent");
}

/// v1's `Layer::ResetHoldTime(true)` (ev_97d7ba71), transcribed: region (or
/// whole file) in seconds, divided by 2^((coarse + fine)/12) unless
/// timestretching, minus A⁴·10 + (S ≠ 1 ? D⁴·10 : 0) + R⁴·15, clamped to
/// 0..60 s, then (x/60)^0.25 clamped to 0..1. Coarse is
/// `round(c·24) − 12` semitones, fine `2f − 1`.
fn v1_reset_hold_time(l: &Node, region: f64, total: f64, sr: f32) -> f32 {
    let p = |k: &str, d: f32| l.f32(k).unwrap_or(d);
    let mut t = (region / sr as f64).max(0.0);
    if t.abs() < 1e-6 {
        t = (total as f32 / sr) as f64;
    }
    let coarse = {
        let v = p("CoarseTune", 0.5) * 24.0;
        (if v < 0.0 { v - 0.5 } else { v + 0.5 }) as i32 as f32 - 12.0
    };
    let fine = p("FineTune", 0.5) * 2.0 - 1.0;
    let ratio = 2f64.powf((fine as f64 + coarse as f64) / 12.0);
    if ratio.abs() >= 1e-6 && p("TimestretchActive", 0.0) < 0.5 {
        t *= 1.0 / ratio;
    }
    let q = |x: f32| x * x * x * x;
    let d = if (p("AmpEnvSustain", 1.0) - 1.0).abs() >= 1e-6 { q(p("AmpEnvDecay", 0.02)) * 10.0 } else { 0.0 };
    let x = ((t - (d + q(p("AmpEnvAttack", 0.0)) * 10.0 + q(p("AmpEnvRelease", 0.001)) * 15.0) as f64) as f32).min(60.0).max(0.0);
    (x / 60.0).powf(0.25).min(1.0).max(0.0)
}

/// A v1 layer saved with Hold -1 gets the hold v1 computes from its sample:
/// Dreist's load-time recomputation (which the converted layer triggers)
/// gives the same value for v1's semitone-grid coarse tunes.
#[test]
fn hold_is_recomputed_like_v1() {
    let text = layer("Layer0", "3.2", "")
        .replace(r#"CoarseTune="0.5""#, r#"CoarseTune="0.583333""#)
        .replace(r#"FineTune="0.5""#, r#"FineTune="0.6""#)
        .replace(r#"Envelope0Param0="0" Envelope0Param1="0.02" Envelope0Param2="1""#, r#"Envelope0Param0="0.2" Envelope0Param1="0.3" Envelope0Param2="0.5""#);
    let doc = convert(FileKind::Pad, &format!("<root>{}</root>", pad("Pad", &text)));
    let l = first(first(&doc, "Pad"), "Layer");
    assert_eq!(l.f32("AmpEnvHold"), Some(-1.0));
    assert!(geist_engine::hold::needs_reset_on_load(l));
    let lp = |k: geist_formats::LayerParam| l.f32(k.name()).unwrap_or(k.default_value()).clamp(0.0, 1.0);
    // A region shorter than the envelope gives 0; longer ones a hold in between.
    for (region, total, sr, inside) in [(4000.0, 4000.0, 44_100.0, false), (0.0, 88_200.0, 44_100.0, true), (120_000.0, 200_000.0, 48_000.0, true)] {
        let dreist = geist_engine::hold::reset_hold_time(&lp, false, region, total, sr, true);
        let v1 = v1_reset_hold_time(l, region, total, sr);
        assert!((dreist - v1).abs() < 1e-5, "region {region}: Dreist {dreist} vs v1 {v1}");
        assert_eq!(v1 > 0.0 && v1 < 1.0, inside, "region {region}: {v1}");
    }
    // A saved hold is kept (clamped to 0..1).
    let kept = convert(FileKind::Pad, &format!("<root>{}</root>", pad("Pad", &text.replace(r#"Envelope0Param3="-1""#, r#"Envelope0Param3="0.4""#))));
    let k = first(first(&kept, "Pad"), "Layer");
    assert_eq!((k.f32("AmpEnvHold"), k.f32("FreeEnvHold")), (Some(0.4), Some(-1.0)));
    assert!(geist_engine::hold::needs_reset_on_load(k), "either hold negative → reset, as in v1");
}

#[test]
fn defaults_only_project_plays() {
    let text = r#"<root StateVersion="2.1">
        <Engine name="Engine0">
            <Pad name="Pad0"><Layer name="Layer0" AudioFileName="Beat.wav" /></Pad>
            <StepPattern name="StepPattern0"><Track name="Track0"><Step name="S0" State="true" /><Step name="S8" State="true" /></Track></StepPattern>
        </Engine>
    </root>"#;
    let doc = convert(FileKind::Project, text);
    let layer = first(first(first(&doc, "Engine"), "Pad"), "Layer");
    // No `Ver`: Geist2 reads it as 3300 and converts nothing.
    assert_eq!(layer.version, Some(3300));
    assert!(layer.attr("TimestretchActive").is_none());
    assert!(render(&doc, 2.0) > 1e-3, "silent");
}

#[test]
fn old_state_version_is_rejected() {
    assert!(Document::from_str(FileKind::Project, r#"<root StateVersion="1.9" />"#).is_err());
    assert!(Document::from_str(FileKind::Project, "<root />").is_err());
}

#[test]
fn layer_version_branches() {
    let text = |ver: &str| {
        let l = layer("Layer0", ver, "")
            .replace(r#"UsesTimestretch="false" TSTracksBPM="false""#, r#"UsesTimestretch="true" TSTracksBPM="true""#)
            .replace(r#"V2C="0.5" V2P="0.5""#, r#"V2C="0.75" V2P="0.25""#)
            .replace(r#"TSMode="Default""#, r#"TSMode="Fast""#);
        format!("<root>{}</root>", pad("Pad", &l))
    };
    // 3.1: below 3200, cutoff and resonance take the old scales.
    let doc = convert(FileKind::Pad, &text("3.1").replace(r#"Cutoff="1" Resonance="0""#, r#"Cutoff="0.5" Resonance="0.5""#));
    let l = first(first(&doc, "Pad"), "Layer");
    let c = l.f32("Cutoff").unwrap();
    let want = ((0.5f32.powi(4) * 22020.0 + 30.0) / 55.0).log2() / 12.0 + 0.25;
    assert!((c - want).abs() < 1e-4, "{c} vs {want}");
    assert!((l.f32("Resonance").unwrap() - (0.5 * 1200.0 - 101.0) / 909.0).abs() < 1e-4);
    assert_eq!(l.f32("Cutoff_Velocity"), Some(0.5));
    assert_eq!(l.f32("CoarseTune_Velocity"), Some(-0.5));
    assert_eq!((l.f32("TimestretchActive"), l.f32("TimestretchTracksBPM"), l.f32("TimestretchMode")), (Some(1.0), Some(1.0), Some(1.0)));
    // 3.2: cutoff untouched.
    let doc = convert(FileKind::Pad, &text("3.2"));
    assert_eq!(first(first(&doc, "Pad"), "Layer").f32("Cutoff"), Some(1.0));
    // 2.9: below 3000, holds -1 and the old default decay becomes 0.02.
    let old = text("2.9").replace(r#"Envelope0Param1="0.02""#, r#"Envelope0Param1="0.5""#).replace(r#"Envelope1Param1="0.02""#, r#"Envelope1Param1="0.3""#);
    let doc = convert(FileKind::Pad, &old);
    let l = first(first(&doc, "Pad"), "Layer");
    assert_eq!((l.f32("AmpEnvHold"), l.f32("FreeEnvHold")), (Some(-1.0), Some(-1.0)));
    assert_eq!((l.f32("AmpEnvDecay"), l.f32("FreeEnvDecay")), (Some(0.02), Some(0.3)));
    assert!(geist_engine::hold::needs_reset_on_load(l));
}

#[test]
fn modulation_routings_become_envelope_depths() {
    let routed = layer("Layer0", "3.2", "")
        .replacen(r#"ModulationDepth="0.5" ModulationDestination="No destination""#, r#"ModulationDepth="0.75" ModulationDestination="Cutoff""#, 1)
        .replacen(r#"ModulationDepth="0.5" ModulationDestination="No destination""#, r#"ModulationDepth="0.9" ModulationDestination="No destination""#, 1)
        .replacen(r#"ModulationDepth="0.5" ModulationDestination="No destination""#, r#"ModulationDepth="0.25" ModulationDestination="Pitch""#, 1)
        .replacen(r#"ModulationDepth="0.5" ModulationDestination="No destination""#, r#"ModulationDepth="1" ModulationDestination="Drive""#, 1);
    let doc = convert(FileKind::Pad, &format!("<root>{}</root>", pad("Pad", &routed)));
    let l = first(first(&doc, "Pad"), "Layer");
    assert_eq!(l.f32("Cutoff_AmpEnvelope"), Some(0.5));
    assert_eq!(l.f32("CoarseTune_FreeEnvelope"), Some(-0.5));
    assert_eq!(l.f32("FilterDrive_FreeEnvelope"), Some(1.0));
    assert!(l.attrs.iter().all(|(k, _)| !k.ends_with("_AmpEnvelope") || k == "Cutoff_AmpEnvelope"));
}

#[test]
fn legacy_track_steps_array() {
    let text = r#"<root><StepPattern name="StepPattern" PatternSteps="4">
        <Track name="Track0" TrackVersion="2" NumberOfSteps="4">
            <VObjectArray name="Steps"><Step name="Step" State="true" /><Step name="Step" State="false" /><Step name="Step" State="true" /></VObjectArray>
        </Track></StepPattern></root>"#;
    let doc = convert(FileKind::Pattern, text);
    let t = first(first(&doc, "StepPattern"), "Track");
    let names: Vec<_> = t.children_of("Step").map(|s| (s.name.clone().unwrap(), s.bool("State").unwrap())).collect();
    assert_eq!(names, [("S0".to_string(), true), ("S1".to_string(), false), ("S2".to_string(), true)]);
    assert!(t.attr("TrackVersion").is_none());
}

/// Kit, pad, engine, pattern and bank files: convert, then load through
/// Dreist the way its editor does (pads and patterns placed in an engine).
#[test]
fn single_object_files() {
    let wrap = |inner: &str| Node::parse(&format!(r#"<root version="3000"><Engine name="Engine0">{inner}</Engine></root>"#)).unwrap();
    let xml = |n: &Node| n.children.iter().map(Node::to_xml).collect::<String>();

    let kit = convert(FileKind::Kit, &format!("<root>{}{}</root>", pad("Pad0", &layer("Layer0", "3.2", "")), pad("Pad1", &layer("Layer0", "3.2", ""))));
    let dir = std::env::temp_dir().join(format!("einst-kit-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("k.geistkit");
    std::fs::write(&path, kit.to_xml()).unwrap();
    let k = geist_formats::Kit::load(&path).unwrap();
    assert_eq!(k.pads.len(), 2);
    assert_eq!(k.pads[1].layers[0].audio_file.as_deref(), Some("Beat.wav"));
    let _ = std::fs::remove_dir_all(&dir);

    let p = convert(FileKind::Pad, &format!("<root>{}</root>", pad("Pad", &layer("Layer0", "3.2", ""))));
    let mut pad0 = first(&p, "Pad").clone();
    pad0.name = Some("Pad0".into());
    let mut doc = wrap(&pattern("StepPattern0"));
    doc.children[0].children.push(pad0);
    let pat = convert(FileKind::Pattern, &format!("<root>{}</root>", pattern("StepPattern")));
    assert_eq!(first(&pat, "StepPattern").attr("Resolution"), Some("1/16"));
    assert!(render(&Node::parse(&doc.to_xml()).unwrap(), 2.0) > 1e-3, "pad file silent");

    let e = convert(FileKind::Engine, &format!("<root>{}</root>", engine("Engine")));
    let mut eng = first(&e, "Engine").clone();
    eng.name = Some("Engine0".into());
    let mut doc = Node::parse(r#"<root version="3000" />"#).unwrap();
    doc.children.push(eng);
    assert!(render(&doc, 2.0) > 1e-3, "engine file silent");

    let bank = convert(FileKind::PatternBank, &format!("<root>{}{}</root>", pattern("StepPattern0"), pattern("StepPattern1")));
    let doc = wrap(&xml(&bank));
    let p = geist_formats::Project::from_node(doc).unwrap();
    assert_eq!(p.engines[0].patterns.len(), 2);
    assert_eq!(p.engines[0].patterns[1].tracks[0].steps.len(), 4);
}
