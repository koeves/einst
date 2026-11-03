# Geist v1's effects compared with Geist2's (Dreist's ports)

v1 engine `original/Geist-engine-x86_64` (GCC build) against Geist2 2.1.2.13 (clang), whose
effect DSP Dreist ports in `dreist_dsp::fx::nodes`. The class behind each v1 effect type is in
`loading-v1.md` ("Effect types and the v1-only DSP"); the three v1-only classes (Gain, Vari-EQ,
Envelope Shaper) are described there.

## Method

Per effect, four comparisons:

1. **Port tables:** v1's `InitializePorts` (port names, defaults and ranges, in creation order)
   against Dreist's `dreist_dsp::fx::ports` (generated from Geist2's) [disasm, decomp].
2. **Front-panel → port mapping:** v1's `Effect::PushParameterValue` 0x2efc60 (ev_a735042f)
   against Dreist's `nodes::node_port` [decomp].
3. **Hidden ports:** v1's `Effect::SetupInternalParameters` 0x2f0250 (ev_5107cdbc) against
   `nodes::setup_internal` [decomp].
4. **DSP fingerprint:** for the class's functions, and every DSP helper they call, the
   float/double constants each one loads, the strings it references and the named functions it
   calls, compared by demangled name. Constants are compared by absolute value, without the
   trivial ones (0, ±0.5, ±1). v1's inlined fast 2^x (`loading-v1.md`) is flagged separately
   [disasm].

   Scripts: `re/scripts/fx_fingerprint.py` (fingerprints), `fx_compare.py` (per-function diff),
   `fx_summary.py` (per-class summary):

       python3 re/scripts/fx_fingerprint.py original/Geist-engine-x86_64 v1.json 'VDSP::|TDImpl|Breverb|Effect::'
       python3 re/scripts/fx_fingerprint.py <Geist2 binary> g2.json 'VDSP::|TDImpl|Breverb|Effect::'
       python3 re/scripts/fx_summary.py VFlangerS      # in the folder holding v1.json / g2.json

A matching fingerprint means no differing constant, string or callee. It does not prove the
two compile from identical code; arithmetic can still be reordered. A differing fingerprint
can still be algebraically equal: GCC and clang fold constants differently, e.g. π·2x vs 2π·x,
π/ln 2 = 4.53236 vs 1/ln 2 = 1.4427, 4π² = 39.4784, x·0.05 vs x/20. Those cases are marked
"to decompile".

## Results (2026-11-03, batch 1)

Mapping (2) and hidden ports (3) match Geist2 for every type except Noise Gate. Port tables
(1) match for every class except Noise Gate.

| v1 type | Effect | Class | Result |
|---|---|---|---|
| 1 | Gain | VAmpM | v1-only class: **differs, fixed** (`einst_dsp::gain`) |
| 2 | Stereo Width | VStereoMixer | **differs, fixed**: v1's fast 2^x for the gain; no other difference in the fingerprint (`einst_dsp::stereo`) |
| 3 | Parametric EQ | VEQSSLESeriesS | differs, **to decompile**: `VSSLEqCommon::peqx` has 0.02 where Geist2 has 0.01 and −π²; the shelves are inlined in Geist2 with clamps (0.001, 0.45) v1 lacks; v1 fast 2^x in `UpdateCoeffs`; bell/peak gain written as ·0.025/·0.05 vs /40, /20 |
| 4 | DJ EQ | VEQSSLESeriesS | as Parametric EQ |
| 5 | Vari-EQ | VFilterEQ2 | v1-only class: **differs, fixed** (`einst_dsp::vari_eq`) |
| 6 | Channel Compressor | VCompressorUrei1176S | **differs, fixed** (batch 2): flat input gain (no per-sample ramp), v1's fast 2^x for attack/release; solvers equal (`einst_dsp::urei1176`) |
| 7 | Bus Compressor | VCompressorSSLMixbusS | **differs, fixed** (batch 2): v1's fast 2^x for the side-chain high-pass; circuit, tables and diode solve equal (`einst_dsp::bus_comp`) |
| 8 | Noise Gate | VNoiseGateS | **differs**: v1's port order (Hysteresis, LPFreq, HPFreq, Preview at 4–7; Sidechain 8, WetMix 9; ev_df3a50d3) — **fixed** (`einst_dsp::noise_gate` adapter, v1's hidden ports); DSP **to decompile**: fast 2^x, π vs 20 |
| 9 | Envelope Shaper | VDSPTransientDesigner | v1-only class: **differs, fixed** (`einst_dsp::envelope_shaper`) |
| 10 | Delay | VDelayStereo | differs, **to decompile**: v1 fast 2^x; 1e−10 / 1e10 (denormal guard?), π vs 2π; Geist2 `PrepareStream` and aligned buffers |
| 11 | Flanger | VFlangerS | **no difference found** |
| 12 | Chorus | VChorusS | **no difference found** |
| 13 | Phaser | VFilterPhaser8 | **differs, fixed**: v1's fast 2^x for the pitch; nothing else (`einst_dsp::phaser`) |
| 14 | Distortion | VAudioShaperS | **to decompile**: v1 fast 2^x; 0.0497512 (= 1/20.1), π vs 2π |
| 15 | Bit Crusher | VBitCrusherS | **to decompile**: v1 fast 2^x; π vs 2π only |
| 16 | Ring Mod | VRingModS | differs, **to decompile**: v1 0.2, 0.4, 0.625 vs Geist2 0.125; Geist2's class has 23 functions to v1's 10; v1 fast 2^x |
| 17 | Frequency Shifter | VFreqShifterS | **differs, fixed**: v1's fast 2^x for the shift; nothing else (`einst_dsp::freq_shifter`) |
| 18 | Filter | VFiltersSSLChanS | differs, **to decompile**: v1 uses VFilterEQCommon-style clamps (0.001, 0.451, Q 0.5..1000) and π/2, Geist2 1e−5 and √2; v1 fast 2^x |
| 19 | Modulated Filter | VFilterAutoS | **differs, fixed** (batch 2): v1's fast 2^x for attack, release and cutoff; FM from the left input only (as Geist2; Dreist's port takes it per channel); rest equal (`einst_dsp::auto_filter`) |
| 20 | Comb Filter | VFilterComb | **to decompile**: v1 fast 2^x; 4π², ·0.05 vs /20 (likely equal forms) |
| 21 | Three Oh! Three Filter | VFilterTB303 | **to decompile**: v1 fast 2^x; π/ln 2, 4π² (equal forms?) and 2.4 vs 0.0625, 2.0 |
| 22 | Hoover Filter | VFilterJuno106 | **to decompile**: v1 fast 2^x; π/ln 2, 4π² only (likely equal forms) |
| 23 | Grandad Filter | VFilterMoog4 | **to decompile**: v1 fast 2^x; π/ln 2 vs 1/ln 2 and 2π (likely equal forms) |
| 24 | Little Fatty Filter | VFilterMS20 | **to decompile**: v1 fast 2^x; π vs 2π, 20π |
| 25 | Child Genius Filter | VFilterMoogProdigy | **to decompile**: v1 fast 2^x; π/ln 2, 4π² only (likely equal forms) |
| 26 | Japan Filter | VFilterSH101 | differs, **to decompile**: the circuit constants move between `ProcessBuffer` and `InitializePorts`/converters, and several are v1-only (0.0227549, 43.9467, 0.0245, 1/120) |
| 27 | Audio Freezer | VAudioFreeze | differs, **to decompile**: Geist2 has a note-length sync table (1/16, 1/16T, … strings) v1 lacks; v1 1e−10/1e10; port constant 4.0 v1-only |
| 28–31 | Breverb Hall / Plate / Room / Inverse | VDSPBreverb* | **identical** (batch 2): the "v1-only constants" were a reachability artefact (Geist2 reaches them through `VDSPBreverbBase`); a copy built from v1's code renders bit-identically to Dreist's |
| 32 | Tin Can Verb | VReverbS | differs, **to decompile**: v1 0.0625, 0.49, 0.495, 5.5 vs Geist2 0.125, 0.2, 0.8, 0.98, 0.99, 5.0 |

v1's fast 2^x (within 0.4 % of 2^x) appears in nearly every v1 `ProcessBuffer`. Where Dreist's
node calls `exp2` at the same point, v1 computes a slightly different parameter, typically a
frequency up to about 7 cents off or a gain up to 0.03 dB off. The batch-1 fixes copy Dreist's
node and swap that one call. The tests (`crates/einst-dsp/tests/v1_nodes.rs`) show that each
copied node equals Dreist's when Dreist's is fed log2(fast2^v), i.e. that this call is the only
difference.

## Batch 2 (2026-11-03): compressors, Breverb, Modulated Filter

Decompiled and compared function by function with Dreist's ports.

- **Channel Compressor** (`Urei1176Comp`): `process` (ev_9b3b4723 `Solve1176A`,
  ev_7029d2a1 `SolveDRCR2`) applies the input gain 10^((in − 10)/20) flat per block where Geist2
  ramps it per sample; attack/release are v1's fast 2^x times 23636362 / 4545454.5. The solver
  constants that differ in the fingerprint are folded RC products (e.g. 4.7e6 = 470·1e4) and
  give the same values (test `channel_compressor_solver_constants`).
- **Bus Compressor** (`MixbusComp`): constructor ev_15cdcb65, `setRatio` ev_a170b69d,
  `setRelease` ev_a8999690, `setSampleRate` ev_746a5beb, `setSidechainHP` ev_af0d4af3,
  `process` ev_53f3c025 and the two-diode Newton solve `RDI012` ev_32d287e3 equal Geist2's
  with GCC's folded reciprocals (·3.030303e−7 = /3.3e6, ·2127659.5 = /4.7e−7, ·147058.83 =
  /6.8e−6, ·0.12820512 = /7.8, ·0.0060975607 = /164). `Voice::ProcessBuffer` ev_4014c08a
  takes the side-chain HP frequency from v1's fast 2^x. v1 also smooths a gain-reduction meter
  after the loop (not modelled, as in Dreist).
- **Modulated Filter** (`VFilterAutoS`): `ProcessBuffer` 0x553a10 (ev_9fed2d28),
  `Process121NonlinSem2` ev_2cbcc7ed, `Voice::Reset` ev_51bf7baa, `UpdateSampleRate`
  ev_d94dfc71. Same circuit; v1's fast 2^x at the three exp2 points. FM: v1 and Geist2
  (`ProcessBuffer` 0x4df120, one `exp2f` and one 0.24 clamp per sample per mode loop
  [disasm]) compute one coefficient per sample from the left side-chain input; Dreist's port
  computes it per channel, which is a porting slip in Dreist (to fix there).
- **Breverb**: identical (see the table).

Goldens: Blister and Hollow change (both use Channel Compressor and Modulated Filter; no demo
uses Bus Compressor).

## Goldens

`crates/einst-engine/tests/golden/engine.txt` changes only for the Dreist demos that use the
changed effects: Blister uses Gain, Vari-EQ, Stereo Width and Frequency Shifter; Hollow uses
Gain. With the v1 nodes switched off, the engine copy reproduces Dreist's fingerprints exactly.
