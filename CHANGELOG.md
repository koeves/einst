# Changelog

## [0.1.0] - 2026-11-03

First release: the Geist v1.1.1.2 rebuild, a thin app on dreist-commons (Apple Silicon:
VST3, CLAP, standalone).

### Ported from Geist v1

- v1 files: projects, kits, pads, engines and patterns convert onto the v2 document on load,
  with v1's object defaults and layer holds; all of v1's factory effect presets, chains, pattern
  and swing templates load. Einst saves its own `.einst` family.
- v1's factory resources (effects, chains, templates) are bundled.
- Effects: v1's own Gain, Vari-EQ and Envelope Shaper, and v1's differences in the effects it
  shares with Geist2 (Stereo Width, Phaser, Frequency Shifter, Noise Gate, Channel and Bus
  Compressor, Modulated Filter, Comb Filter, Ladder, Japan Filter, Little Fatty, Bit Crusher,
  Distortion, EQ, Filter, Delay, Tin Can Verb, Audio Freezer, Ring Mod), including v1's fast
  2^x approximation.
- Voice: v1's pitch, pitch envelope and filter modulation model, envelope end and forward-loop
  rules, sampler auto-stop, and the fast 2^x in the voice filters.
- Einst's app profile: names, theme, fonts, settings folder, v1's effect menu.

### Not yet

- v1 MIDI and host bindings (converter in progress), v1's per-step routing-depth lanes and some
  timestretch, reverse and mixing details.
