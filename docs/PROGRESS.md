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
- `einst-editor` (below): Einst's names, settings folder, theme, factory resources
  (`Contents/Resources/Einst`: v1's effect presets, chains, pattern and swing templates), icon
  placeholder (`assets/einst`, `scripts/icon.py`), and v1 files through its Open dialog, browser
  and drag and drop.

- Hold of v1 layers recomputed like v1 (owner's decision, 2026-11-03): a v1 Hold of −1 stays −1,
  so Dreist's load-time `reset_hold_time` (the same formula as v1's `Layer::ResetHoldTime`,
  ev_97d7ba71) derives it from the sample; `tests/v1_documents.rs` checks the two agree.

## `einst-editor`: Einst's copy of Dreist's editor

Copied from Dreist `crates/geist-editor` at commit `bc729de21717d9788e959a71f91b99e1276004db`
(2026-11-02, "Dreist beta 1" tree). To re-sync: copy that crate again from the newer Dreist commit
and re-apply Einst's changes, which are all marked `Einst` or listed here:

- `Cargo.toml`: crate `einst-editor`, depends on `einst-formats`; `geist_editor::` → `einst_editor::`
  in `examples/`.
- `lib.rs`: `PRODUCT = "Einst"`, `DEFAULT_THEME = "Einst"`, `OPEN_EXTENSIONS`, modules `paths`
  and `v1io` (both Einst's own files).
- `window.rs`: Open/Save filter "Einst project" (Open takes the v1 extensions).
- `options.rs`: settings folder `~/Library/Application Support/Einst` (`EINST_SETTINGS_DIR`).
- `modal.rs`: About text, "start Einst", the theme menu (Einst first as "Default", Dreist's base
  theme not offered twice). `editor/tools_menu.rs`: "Reset Einst", default preset via `v1io`.
- `editor/requests.rs` (Open Project, Recent, browser and dropped projects), `project.rs`
  (`load_part`: engines, pads, patterns, banks), `browser.rs` (kits): read through
  `v1io::read_project` / `read_document`, so Geist v1 files are converted. A converted v1 project
  has no project path (Save asks for a new name, as for Guru presets).
- Every `geist_formats::paths::skin_dir()` → `crate::paths::skin_dir()` (Einst's factory data).
- `fxchain.rs`: the effect type menu lists v1's types per category (`einst_formats::v1::
  effect_menu`); Dreist's 47-type `category` table is removed.
- `tests.rs` / `screens_test.rs`: `crate::DEFAULT_THEME`, "Remove all modulation from Einst";
  `tests/golden/screens.txt`: `modal-about` re-recorded (the About text), all other screens as
  Dreist's.

## `einst-engine` and `einst-dsp`: Geist v1's own effects

- `einst-dsp`: v1's mono effect nodes as `geist_dsp::fx::Node`s: `gain::Gain` (VAmpM),
  `vari_eq::VariEq` (VFilterEQ2) and `envelope_shaper::EnvelopeShaper` (VDSPTransientDesigner;
  Dreist's Geist2 node with Linked held off). The recovered DSP is in
  `docs/re/loading-v1.md`; tests check it against the decompiled formulas.
- Wiring: Dreist's engine keeps its effect chains private and builds every effect in one place
  (`geist-engine/src/fx.rs`, `Effect::new` → `geist_dsp::fx::nodes::create`), and Cargo cannot
  `[patch]` a path dependency. So `einst-engine` is a copy of Dreist's `crates/geist-engine` at
  the same commit as the editor (`bc729de2…`). Its library keeps the name `geist_engine`, and
  dependents use `geist-engine = { package = "einst-engine", ... }`, so the copied editor and
  plugin compile unchanged. The only change is in `src/fx.rs` (marked `Einst`): after
  `Effect::new`, `einst_dsp::create(kind)` replaces the node for Gain, Vari-EQ and Envelope
  Shaper, with v1's internal ports set. To re-sync, copy the crate again and re-apply that
  block, the `einst_tests` module at the end of `fx.rs`, and `Cargo.toml`.
  `tests/golden/engine.txt` is re-recorded: the 16 Blister/Hollow scenarios differ from Dreist's
  because both demos use Gain (and Blister Vari-EQ); with the v1 nodes switched off the copy
  reproduces Dreist's fingerprints exactly (checked 2026-11-03).
- The editor's effect menu offers v1's 32 types in v1's order and categories
  (`einst_formats::v1::{EFFECT_TYPES, effect_menu}`). Effect files store type names, which v1 and
  Geist2 share, so the converter keeps them; a test checks all 32.

## Known gaps (plugin shell / editor)

- Dreist's UI graphics (skin, fonts, cursors, metronome sounds) still come from
  `Contents/Resources/Dreist` (`geist_formats::paths::ui_dir`, read inside `geist-gui` and
  `geist-engine`); the UI font is still "Dreist Sans". Changing those needs Dreist's crates.
- Geist2-only effect types in a Geist2 document still load and play with Dreist's nodes; Einst's
  menu just does not offer them.
- The browser lists `.geist` and `.geistkit` as Dreist's does; v1 `.geistpad/.geistengine/
  .geistpattern/.geistbank` open through the Open dialog (into engine 0) and drag and drop.
- No CLAP/VST3 validator is installed (clap-validator, pluginval); downloading one needs the user's
  OK. Checked instead: `tools/plugincheck.py` (CLAP descriptor, create/init/activate; VST3
  factory), and Dreist's validation harnesses (`geist-dsp` fx_harness / fx_realtime,
  `geist-engine` harness / realtime) pass when run against its sources with
  `--target-dir target/dreist`.

## Next

1. Confirm the remaining conversion guesses in `docs/re/loading-v1.md` ("Guesses and
   deliberate choices"). Convert v1 MIDI/host bindings once `Binding`'s message objects are
   traced.
2. Remaining UI gaps above need Dreist's `geist-gui` / `geist-engine` (owner, 2026-11-03: keep
   Dreist untouched).
3. The other 29 effects share their DSP class with Geist2 by name; check where v1's copies of
   those classes differ from Geist2's (as VFilterEQCommon did).
4. Diff behaviour of functions both versions share (voice, sequencer, mixer) and record v1 deltas.
