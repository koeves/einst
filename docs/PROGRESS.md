# Progress / resume guide

## Decisions (user)

- 2026-11-02: Einst is its own app (VST3 + CLAP + standalone) on Dreist's crates as dependencies.
- Same principles as Dreist (see `../geist/docs/PROGRESS.md`): faithful to the decompiled original,
  no licensing code, no FXpansion graphics in Einst's own UI.
- The user has no Geist v1 user files; the v1 factory install (`assets/geist1`) is the only real
  content to test against.

## Done

- Feasibility report: `docs/feasibility.md`.
- Static maps of v1's save code: `re/serializers-x86_64.txt` (342 functions), `re/object-model.md`.
- v1 vs Geist2 comparison: `docs/re/v1-vs-geist2.md`.
- `einst-formats`: reads every v1 factory effect preset, chain, pattern/swing template and skin;
  effect presets/chains/templates map 1:1 onto Geist2's schema.
- REA (Ghidra) on v1's engine: load path, sample-path resolution, every project/kit/pad/engine/
  pattern serializer, version constants and load-time upgrades, enum name tables, and the effect
  type → DSP class registry: `docs/re/loading-v1.md`. Driver for long analyses:
  `re/scripts/rea_drive.py`.
- v1 object defaults (constructors / `Initialize` / `ResetState`) and the Sampler, BindingMap and
  MIDIMap serializers: `docs/re/loading-v1.md`.
- `Document::to_geist2` for projects, kits, pads, engines, patterns and banks
  (`einst_formats::geist2`), following Geist2's own legacy branches (decompiled: root,
  `Layer`, `Step`, `Track`). Tested on synthetic v1 documents built from the serializer field
  lists (`tests/v1_documents.rs`): every kind converts, Dreist's `geist_formats` loads the
  result, and `Geist::from_document` renders non-silent, finite audio from a pad playing v1's
  factory metronome click.

- `einst-plugin` (VST3 + CLAP + standalone), `xtask` and `scripts/build.sh`: a copy of Dreist's
  shell and build with Einst's names and IDs, on Dreist's crates, vendored patches and exact
  dependency versions (Cargo.lock seeded from Dreist's). Geist v1 files open through
  `load_project` (saved project path or `EINST_PROJECT`).

## Known gaps (plugin shell)

- Dreist's editor is used unpatched, so some UI text still says "Dreist": `geist_editor::PRODUCT`
  (window title, transmodulation menu labels), the Open/Save dialog filter "Dreist project", the
  UI theme name and font. Only the macOS menu bar uses Einst's own `PRODUCT`.
- The editor's Open dialog, browser and drag and drop load files through Dreist's Geist2 readers;
  a v1 file opened there is read as if it were Geist2. Fix needs a document-import hook in
  `geist-editor` (an upstream change: ask the user before touching Dreist).
- Einst shares Dreist's settings folder (`Geist-rs`, or `GEIST_SETTINGS_DIR`) and resource
  lookups (`Contents/Resources/{Geist2,Dreist}`, `GEIST2_RESOURCES`, `DREIST_UI`).
- The bundles carry Dreist's app icon, resources and licence files. v1's own factory resources
  (`assets/geist1`) are not bundled yet.
- No CLAP/VST3 validator is installed (clap-validator, pluginval); downloading one needs the user's
  OK. Checked instead (2026-11-03): `tools/plugincheck.py` loads Einst.clap (descriptor
  `rs.einst.einst`, create/init/activate at 48 kHz) and Einst.vst3 (class `EinstRsDrumSmplr`),
  and Dreist's own validation harnesses (`geist-dsp` fx_harness / fx_realtime, `geist-engine`
  harness / realtime) pass when run against its sources with `--target-dir target/dreist`.

## Next

1. Decide the open conversion choices in `docs/re/loading-v1.md` ("Guesses and deliberate
   choices"), above all the Hold on v1 3.x layers (Geist2 clamps −1 to 0, v1 recomputes it).
   Convert v1 MIDI/host bindings once `Binding`'s message objects are traced.
2. Close the plugin-shell gaps above: needs small upstream hooks in Dreist (product name,
   document import, settings folder). Ask the user before touching Dreist.
3. v1-only DSP: `VAmpM` (Gain), `VFilterEQ2` (Vari-EQ), `VDSPTransientDesigner` (Envelope
   Shaper) are mono versions of Geist2's `VAmpS`/`VFilterEQ2S`/`VDSPTransientDesignerS`; diff
   them against Dreist's ports.
4. Diff behaviour of functions both versions share (voice, sequencer, mixer) and record v1 deltas.
