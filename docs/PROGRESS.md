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

## Next

1. REA (Ghidra) on v1: decompile `Grid::LoadEntireState`, `Grid::SerializeKitFileToEngine`, the
   `SerializeAsXML` bodies of Engine/Pad/Layer/StepPattern/Track/Step/Song/SceneManager, and
   Geist2's legacy-load branches (`Ver` < 3300, `StateVersion`, `FileManager`) to write
   `v1::Document::to_geist2` for projects, kits, pads, engines, patterns and banks.
2. `einst-plugin`: nih-plug shell with Einst IDs, wrapping Dreist's engine/editor. Needs either a small
   upstream change in Dreist (a product/ID parameter in `geist-plugin`) or an Einst copy of the shell.
   Ask the user before touching Dreist.
3. v1-only DSP: `VDSPTransientDesigner`, `VAmpM`, `VFilterEQ2`; find which effect types use them.
4. Diff behaviour of functions both versions share (voice, sequencer, mixer) and record v1 deltas.
