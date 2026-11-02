# Geist v1 file formats

XML written by `VFoundation::VXMLFile` through TinyXML (`VXMLFileIOTinyXml`) [sym]. Same element
shape as Geist2's `SerializationNode`: tag = class, `name` = instance key, parameters as attributes,
strings as `FixedString`/`VString` children. `geist_formats::Node` parses them unchanged [file].

| Ext | Writer [sym] | Contents | Status |
|---|---|---|---|
| `.geist` | `Grid::SaveEntireStateToFile` → `Grid::SaveEntireState` | `StateVersion`, `FileManager`, `MasterChannel`, `Bindings`, `Scenes`, `Song`, `TimeInfo`, `Sampler`, `Metronome`, `SendMIDINotes`, `SendMIDICCs`, `CurrentEngine`, `PreviewVolume`, `PreviewOutput`, `Engine%d`, `GlobalAux%d` | structure only |
| `.geistkit` | `Grid::SerializeCurrentKit` | `Pad0..Pad15` | structure only |
| `.geistpad` | `Grid::SerializeCurrentPad` | one `Pad` | structure only |
| `.geistengine` | `Grid::SerializeCurrentEngine` | one `Engine` | structure only |
| `.geistpattern` | `Grid::SerializeCurrentPattern` | one `StepPattern` | structure only |
| `.geistbank` | `Grid::SerializeCurrentPatternBank` | `StepPattern%d` | structure only |
| `.fxpreset` | | one `Effect` (`EffectVersion="2"`, `Parameter0..5`, `Mix`, `Level`, `IsActive`, `EffectType`) | = Geist2 [file] |
| `.fxchain` | | `EffectChain` (`FXVer="2"`) of `Effect%d` | = Geist2 [file] |
| `.patterntemplate` | | `InsertablePattern` (`PatternVersion="2"`, `Element0..N`) | = Geist2 [file] |
| `.swingtemplate` | | `SwingTemplate` (`SwingTemplateVersion="4"`, `Step%d`) | = Geist2 [file] |

All writers go through `XMLFile::Initialize(bool, VString, AudioFileManager*, bool, bool)` with the
`SaveSamplesWithChunks` / `AllocatorRestoreOutputs` options [sym].

## Objects (field order from the save code) [sym]

- `Engine`: `Ver`, mix state (`Gain Pan Mute Solo`), output state, `EffectChain`, aux sends,
  `SwingTemplate`, `EngineState`, `Pitch`, `TempoNumerator`, `TempoDenominator`, `SwingTemplateDepth`,
  `StepPattern%d`, `Pad%d`, `EnginePadMode`.
- `Pad`: `Ver`, `LockState`, mix, output, `EffectChain`, aux sends, `V2A`, `PadMode`, `PadName`, `Layer0..7`.
- `Layer`: `Ver`, `LockState`, mix, output, `EffectChain`, aux sends, `VelocitySplit`, `StartEndMarkers`,
  `LoopMarkers`, `LayerName`, `IsReversed`, `PhasePosition`, `UsesTimestretch`, `TSTracksBPM`, `SourceBPM`,
  `DestinationBPM`, `V2C`, `V2P`, `EnvTracksPitch`, `Envelope%dParam%d`, `ModRouting%d`,
  `EnvelopeType%d`, `CutMode`, `PlaybackMode`, …
- `StepPattern`: `Ver`, `LockState`, `PatternName`, `PatternSteps`, `MonoModeActive`, `Track%d`,
  `SyncMode`, `Resolution`. `Step`: `Ver`, `State`.
- `SlicePoint`: `Ver`, `Start`, `End`, `Locked`, `Deleted`, `Pos`, `Strength`, `DynSense`, `Mean`, `Type`.
- Full list: `re/object-model.md`.

`Ver` is a per-object float; Geist2 reads a legacy layer `Ver` as `round(Ver*1000)` and upgrades
layers below 3300 (Dreist `docs/re/formats.md`), which matches these v1 field names.

Types, defaults, version branches, enum strings and sample-path rules: `loading-v1.md` [decomp].
The document element of every v1 file is `<root>`; a `.geist` with `StateVersion < 2.0` is
rejected.
