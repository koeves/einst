# Geist v1.1.1.2 compared with Geist2 2.1.2.13 (Dreist's target)

Static comparison, 2026-11-02. v1: thin x86_64 engine `original/Geist-engine-x86_64`
(sha256 `1751b602…8554`). Geist2: Dreist's `re/out/symbols.txt` (engine sha256 `395d2ccf…e46d`).
Tags as in Dreist's ledger: [file] [sym] [decomp] [runtime].

## Shared codebase [sym]

- Both are built from FXpansion's VFoundation / VDSP / VCtrl framework. v1's source paths say
  `Geist64Bit/FXpansion/Grid`: "Grid" is v1's internal name for the engine (`Grid::LoadEntireState`).
- Class-level overlap (classes with 2+ symbols): v1 843, Geist2 822, shared 480
  (`re/classes-shared-with-geist2.txt`, `re/classes-v1-only.txt`). Most v1-only classes are GUI,
  AU SDK, `VApplication` and the older VFoundation containers.
- **DSP:** 93 of v1's 97 `VDSP::` classes also exist in Geist2. v1-only: `VAmpM`,
  `VDSPTransientDesigner` (+ `VDSPTransientDesignerConvertParam` is shared), `VFilterEQ2`,
  `IDisplayVisualisation`.
- **Effects:** all 32 v1 effect types exist by name in Geist2, and Dreist has ported all 47 Geist2 effects.
  Of v1's 260 factory `.fxpreset` files, 198 are byte-identical to Geist2's, 62 differ, and none are
  v1-only [file]. The changed ones still need a diff (values or line endings).
- **File I/O:** v1 serializes through `VFoundation::VXMLFile` + TinyXML (`SerializeAsXML`);
  Geist2 uses `SerializationNode` but **still contains** `VXMLFileIOTinyXml` and the v1 field
  names `StateVersion`, `FileManager`, `TimeInfo`, `Scenes`, `Bindings`, `Envelope%dParam%d`,
  `ModRouting%d`, `V2C`/`V2P`. Dreist's formats.md notes that Geist2 upgrades layers below version 3300
  (`V2C`/`V2P`/`UsesTimestretch`/`TSTracksBPM`/`TSMode`/`Envelope%dParam%d`/`ModRouting%d`).
  v1 itself is now decompiled: `loading-v1.md` has the full load path and every serializer
[decomp]. v1.1.1.2 writes layer `Ver` 3.2 (= Geist2's 3200), so Geist2's "below 3300" layer
upgrade applies to v1 layers.

## What einst needs beyond Dreist

1. v1 file reader: `.geist` (root `StateVersion`, `FileManager`, `Engine%d`, `GlobalAux%d`,
   `Scenes`, `Song`, `TimeInfo`, `Bindings`), `.geistkit` (`Pad0..15`), `.geistpad`, `.geistengine`,
   `.geistpattern`, `.geistbank` (`StepPattern%d`). Field list: `re/object-model.md`.
   Port Geist2's upgrade code where it exists, and decompile v1's `SerializeAsXML` for the rest.
2. v1-only DSP: `VAmpM` backs Gain, `VFilterEQ2` Vari-EQ, `VDSPTransientDesigner` Envelope
   Shaper (`Effect::AddAllSupportedEffects`). Geist2 has stereo `S` versions of all three
   [decomp, see `loading-v1.md`].
3. v1 behaviour differences in voice, sequencer and mixer: diff the decompiled functions that both
   versions have.
4. v1 layout, feature set and branding (Einst), as Dreist did for Geist2.
