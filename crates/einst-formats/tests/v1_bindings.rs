//! v1 bindings (synthetic: no v1 user files exist), converted and read by
//! Dreist's `BindingMap`: keys, message kinds, indices, values and the
//! controller-value mapping.

use dreist_engine::binding::{BindingMap, Message, Stuffer, id, message_id};
use dreist_formats::{LayerParam, Node, STEP_LANES, TransmodSource};
use einst_formats::bindings::convert_binding_map;
use einst_formats::v2::convert_root;

/// A v1 binding (attribute and child names of v1's binding and message
/// serializers).
/// `min` is the binding's `Min`.
fn binding(name: &str, ty: &str, index: [i32; 6], values: &[&str], stuffer: usize, min: &str) -> String {
    let idx = ["EngineIndex", "PadIndex", "LayerIndex", "PatternIndex", "TrackIndex", "StepIndex"].iter().zip(index).map(|(k, v)| format!(r#"{k}="{v}""#)).collect::<Vec<_>>().join(" ");
    let vals: String = values.iter().enumerate().map(|(i, v)| format!(r#"<VValue name="MessageValue{i}" type="float" value="{v}" />"#)).collect();
    format!(
        r#"<Binding name="{name}" Min="{min}" Max="1" MIDI="true" MajorIndex="0" MinorIndex="0" TargetMode="false" EnterTargetMode="false" EditorOnly="false">
            <FixedString name="Path" string="Engine 1" /><FixedString name="Name" string="{ty}" />
            <VString name="MessageStuffer" string="Binding::StuffToValue{stuffer}" />
            <VString name="MessagePopper" string="Binding::GetValueFromValue{stuffer}" />
            <Message name="MessagePrototype" {idx}><VString name="Type" string="{ty}" />{vals}</Message>
            <VString name="Type" string="Continuous" />
        </Binding>"#
    )
}

const L: [i32; 6] = [0, 0, 0, -1, -1, -1];

fn document() -> Node {
    let b = [
        binding("MIDI0CC10Binding0", "e_setLayerParameter", L, &["5", "0.5"], 1, "0"),
        binding("MIDI0CC10Binding1", "e_setLayerV2C", L, &["0.5"], 0, "0"),
        binding("MIDI0CC11Binding0", "e_setLayerModulationDepth", L, &["2", "0.5"], 1, "0.25"),
        binding("MIDI0CC12Binding0", "e_setLayerEnvelopeParameter", L, &["1", "4", "0.1"], 2, "0"),
        binding("MIDI1CC7Binding0", "e_setEngineGain", [1, -1, -1, -1, -1, -1], &["0.8"], 0, "0"),
        binding("MIDI2CC1Binding0", "e_setLayerTimestretchSourceBPM", L, &["120"], 0, "0"),
        binding("MIDI2CC1Binding1", "e_mutePad", [0, 3, -1, -1, -1, -1], &["1"], 0, "0"),
        binding("MIDI2CC2Binding0", "e_setInstoMIDIGraphValue", [0, -1, -1, 0, 5, -1], &["0", "23"], 0, "0"),
        binding("Auto3Binding0", "e_setGraphScaling", [0, -1, -1, 0, 5, -1], &["0.5", "1"], 0, "0"),
        binding("Auto3Binding1", "e_setFXParameter", [0, 0, -1, -1, -1, -1], &["1", "2", "0.5"], 2, "0"),
    ]
    .concat();
    let text = format!(
        r#"<root StateVersion="2.1">
            <BindingMap name="Bindings" HostBindingCount3="2" MIDI0CC10BindingCount="2" MIDI0CC11BindingCount="1" MIDI0CC12BindingCount="1"
                MIDI1CC7BindingCount="1" MIDI2CC1BindingCount="2" MIDI2CC2BindingCount="1">{b}</BindingMap>
            <MIDIMap name="MIDIMap" version="2" />
            <Engine name="Engine0" Ver="2"><Pad name="Pad0" Ver="2"><Layer name="Layer0" Ver="3.2">
                <LayerModulationRouting name="ModRouting2" Ver="1" ModulationDepth="0.5" ModulationDestination="Cutoff" />
            </Layer></Pad></Engine>
        </root>"#
    );
    Node::parse(&text).unwrap()
}

/// The message a binding posts for controller value `v`.
fn fire(map: &mut BindingMap, midi: Option<(usize, usize)>, host: usize, v: f32) -> Vec<Message> {
    let mut out = Vec::new();
    match midi {
        Some((ch, cc)) => map.process_midi_cc(ch, cc, v, &mut out),
        None => map.process_host(host, v, &mut out),
    }
    map.update(&mut out, 0);
    out
}

#[test]
fn bindings_convert_to_what_the_engine_reads() {
    let v1 = document();
    let doc = convert_root(&v1);
    assert!(doc.children_of("MIDIMap").next().is_none());
    let node = doc.children_of("BindingMap").next().expect("binding map kept");
    let mut map = BindingMap::from_node(node);
    let lp = |p: LayerParam| p as i32 as f32;

    // v1 parameter 5 (Cutoff) → v2's FilterCutoff, base column.
    let b = &map.midi_bindings(0, 10)[0];
    assert_eq!((b.prototype.kind, b.prototype.index), (id::SET_LAYER_PARAMETER, L));
    assert_eq!(b.stuffer, Stuffer::Value1);
    assert_eq!((b.prototype.value[0], b.prototype.value[2]), (lp(LayerParam::FilterCutoff), 0.0));
    // V2C → the cutoff's velocity depth, 2v − 1.
    let b = &map.midi_bindings(0, 10)[1];
    assert_eq!((b.prototype.value[0], b.prototype.value[2]), (lp(LayerParam::FilterCutoff), TransmodSource::Velocity as i32 as f32));
    assert_eq!((b.min, b.max), (-1.0, 1.0));
    let out = fire(&mut map, Some((0, 10)), 0, 0.75);
    assert_eq!(out.len(), 2);
    assert!((out[0].value[1] - 0.75).abs() < 1e-6);
    assert!((out[1].value[1] - 0.5).abs() < 1e-6, "{:?}", out[1]);

    // Routing 2 (free envelope → Cutoff): the FreeEnvelope depth of Cutoff.
    let b = &map.midi_bindings(0, 11)[0];
    assert_eq!((b.prototype.value[0], b.prototype.value[2]), (lp(LayerParam::FilterCutoff), TransmodSource::FreeEnvelope as i32 as f32));
    assert_eq!((b.min, b.max), (-0.5, 1.0));
    // Envelope 1 parameter 4 → FreeEnvRelease; v1's value slot 2 is v2's 1.
    let b = &map.midi_bindings(0, 12)[0];
    assert_eq!(b.prototype.value[0], lp(LayerParam::FreeEnvRelease));
    assert_eq!(b.stuffer, Stuffer::Value1);

    // Engine gain → e_setEngineParameter [value, Gain, base].
    let b = &map.midi_bindings(1, 7)[0];
    assert_eq!((b.prototype.kind, b.prototype.index[0], b.prototype.value[1]), (id::SET_ENGINE_PARAMETER, 1, 0.0));
    let out = fire(&mut map, Some((1, 7)), 0, 0.6);
    assert!((out[0].value[0] - 0.6).abs() < 1e-6);

    // The BPM binding is dropped; the pad mute moves up to Binding0.
    let b = map.midi_bindings(2, 1);
    assert_eq!(b.len(), 1);
    assert_eq!((b[0].prototype.kind, b[0].prototype.index[1]), (id::MUTE_PAD, 3));

    // Graph lane 23 (v1 FX1Active) is v2's FX1Active lane.
    let fx1 = STEP_LANES.iter().position(|l| *l == "FX1Active").unwrap();
    assert_eq!(map.graph_binding(0, 0, 5, fx1), Some((2, 2)));

    // Host parameter 3: scaling dropped, the effect parameter renumbered to
    // Auto3Binding0 with the slot in Index3 and [param, source, value].
    let out = fire(&mut map, None, 3, 0.4);
    assert_eq!(out.len(), 1);
    let m = &out[0];
    assert_eq!((m.kind, m.index[3], m.value[0], m.value[1]), (id::SET_FX_PARAMETER, 1, 2.0, 0.0));
    assert!((m.value[2] - 0.4).abs() < 1e-6);
}

#[test]
fn dropped_bindings_are_listed() {
    let v1 = document();
    let map = v1.children_of("BindingMap").next().unwrap();
    let (out, dropped) = convert_binding_map(map, Some(&v1));
    let names: Vec<_> = dropped.iter().map(|d| (d.binding.as_str(), d.message.as_str())).collect();
    assert_eq!(names, [("MIDI2CC1Binding0", "e_setLayerTimestretchSourceBPM"), ("Auto3Binding0", "e_setGraphScaling")]);
    assert_eq!(out.attr("MIDI2CC1BindingCount"), Some("1"));
    assert_eq!(out.attr("HostBindingCount3"), Some("1"));
    // Every kept prototype is a v2 message type, unsmoothed as in v1.
    for b in out.children_of("Binding") {
        let p = b.children.iter().find(|c| c.name.as_deref() == Some("MessagePrototype")).unwrap();
        assert_eq!(p.class, "TMessage_I6_V3");
        assert!(message_id(p.string("Type").unwrap()).is_some(), "{:?}", p.string("Type"));
        assert_eq!(b.attr("NeedsIterp"), Some("false"));
    }
    // Without the document the routing cannot be resolved.
    let (_, dropped) = convert_binding_map(map, None);
    assert!(dropped.iter().any(|d| d.message == "e_setLayerModulationDepth"));
}
