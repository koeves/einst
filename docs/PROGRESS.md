# Progress

The log up to 2026-11-03 (with its reverse-engineering detail) is in the private `dreist-re`
repository, `einst/docs/PROGRESS.md`.

## State (2026-11-03)

- Einst is a thin app on dreist-commons. `crates/einst` installs Einst's profile (names, Einst
  theme and fonts, settings folder `Einst`, the `.einst*` save family, v1 parts in the Open dialog,
  v1's effect menu, v1 documents through `einst_formats`) and v1's effect nodes.
- File formats: every v1 file kind reads and converts onto the v2 document; every v1 factory
  effect preset, chain, pattern and swing template loads (`einst-formats` tests).
- Effects compared with v2 (ledger in dreist-re `einst/docs/re/effects-v1.md`):
  - v1-only DSP: Gain, Vari-EQ, Envelope Shaper (`einst_dsp`).
  - Differ, fixed: Stereo Width, Phaser, Frequency Shifter (v1's fast 2^x), Noise Gate (port
    order), Channel Compressor (flat input gain, fast 2^x), Bus Compressor (fast 2^x for the
    side-chain high-pass), Modulated Filter (fast 2^x for attack, release and cutoff).
  - Identical: Flanger, Chorus, Breverb Hall / Plate / Room / Inverse.
  - Porting fix in dreist-commons found here: Modulated Filter FM per sample from the left input.
- Goldens (`crates/einst/tests/golden`): engine renders and editor screens under Einst's profile,
  byte-identical to before the move to dreist-commons.
- Bundles build and pass `tools/plugincheck.py`. No CLAP/VST3 validator is installed
  (downloading one needs the owner's OK).

## Next

1. Effects batch 3: Japan Filter, Ring Mod, Tin Can Verb, Parametric / DJ EQ, Filter, Audio
   Freezer, Delay, Distortion, the Noise Gate DSP; then the likely-equal ones (Comb, Three Oh!
   Three, Hoover, Grandad, Little Fatty, Child Genius, Bit Crusher).
2. Convert v1 MIDI/host bindings once their message objects are traced.
3. Compare the voice, sequencer and mixer code both versions share and record v1's differences.
