# Geist v1 load path and object serializers

Static RE, 2026-11-02. Target: thin x86_64 engine `original/Geist-engine-x86_64`
(sha256 `1751b602…8554`), Ghidra 12.1.4 via REA (image base 0, so addresses = `nm` addresses).
Tags as in Dreist's ledger: [file] read from the binary's data, [sym] symbol name only,
[decomp] Ghidra pseudocode (Evidence ID prefix given; full IDs in the REA evidence records).

All save/load goes through one function per class: `SerializeAsXML(VXMLFile&)` both reads and
writes, branching on `IsSaving()`/`IsLoading()`. Attribute types below come from the `VXMLFile`
vtable slot each call uses (slot map dumped from `vtable for XMLFile`, see "Tooling") [file, decomp].

Conventions for the tables:

- **type**: `float`, `double`, `int`, `uint`, `bool` = `SerializeAttribute(name, T&)`;
  `string` = `SerializeAttribute(name, VString&, default)`; `child X` = `SerializeObject(name, obj)`
  (a child element; `FixedString` children hold one `string` attribute).
- **default on load**: what the loader sets when the attribute is missing. "keep" means the
  loader leaves the in-memory value alone, so the default is whatever the constructor /
  `Initialize` set (not recovered here, see "Unknown").
- `Ver` is written only when saving and is **not read back** unless noted: most v1 classes have no
  load-time version branches. The only ones with version logic are `Grid` (`StateVersion`),
  `Layer`, `Track`, `SliceAnalysis`.

## Version constants [file]

Static `g2*Version` floats in `__const` (current values a v1.1.1.2 save writes):

| Writer | Attribute | Value |
|---|---|---|
| `Grid::SaveEntireState` | `StateVersion` | 2.1 (also written as immediate `0x40066666`) |
| `Engine`, `Pad`, `EngineState`, `PadState`, `StepPattern`, `Step`, `SwingTemplate`, `MasterChannel`, `GlobalAux`, `Metronome`, `LayerVelocitySplit`, `AudioFileManager`, `SlicePoint` | `Ver` | 2.0 |
| `Effect` | `EffectVersion` | 2.0 |
| `EffectChain` | `FXVer` | 2.1 |
| `AudioFileMarkers` | `FileMarkerVersion` | 2.0 |
| `LayerModulationRouting` | `Ver` | 1.0 |
| `InsertablePattern` | `PatternVersion` | 2.0 |
| `Layer` | `Ver` | 3.2 (local, not a static) [decomp ev_7c48010c] |
| `Track` | `Ver` | 2.2 (local) [decomp ev_2b2d8acb] |
| `SliceAnalysis` | `Ver` | 2.1 (local) [decomp ev_3845204b] |
| `AudioFileMetaData` | `Ver` | int 2 [decomp ev_3a1b344c] |

Geist2 reads a layer `Ver` as `round(Ver*1000)` (Dreist ledger, `docs/re/formats.md` in
`dreist-re`), so v1.1.1.2 layers land at 3200: Geist2's "below 3300" upgrade
(`V2C`/`V2P`/`UsesTimestretch`/`TSTracksBPM`/`TSMode`/`Envelope%dParam%d`/`ModRouting%d`)
applies to them and its "below 3200" cutoff/resonance remap does not. That matches v1's own
`< 3.2` remap below. [decomp v1 and Geist2 `Layer::Serialize` 0x71c10 ev_5075e9a1; see
"Conversion to Geist2"]

## File entry points [decomp]

Every reader opens the file with `VXMLFile::OpenFileForRead(path, "root")`, so the document element
is `<root>`, then calls `XMLFile::Initialize(saveSamplesWithChunks, path, AudioFileManager*,
restoreOutputs, false)`.

| File | Function | Root contents read | Evidence |
|---|---|---|---|
| `.geist` | `Grid::LoadEntireStateFromFile(path, isPreset)` → `Grid::LoadEntireState(XMLFile&)` | see below | ev_17865088, ev_b8366a5a |
| `.geistkit` | `Grid::SerializeKitFileToEngine(path, Engine*)` | `Pad0`..`Pad15` via `ReadObject` into the engine's pads; a missing `Pad%d` leaves that pad unchanged | ev_6c2c28fd |
| `.geistpad` | `Grid::SerializePadFileToPad(path, Pad*)` | one child named by the class, `Pad` | ev_fcee998d |
| `.geistengine` | `Grid::SerializeEngineFiletoEngine(path, Engine*)` | one child `Engine` | ev_fecb1e63 |

Kit/pad/engine readers pass `restoreOutputs = Options "AllocatorRestoreOutputs"` (default true)
and `saveSamplesWithChunks = Options "SaveSamplesWithChunks"` (default false). There is no file
version for kits, pads or engines.

`LoadEntireStateFromFile` passes `saveSamplesWithChunks = true` and
`restoreOutputs = AllocatorRestoreOutputs || !isPreset`. When `isPreset` and the path equals
`Paths::GetDefaultPresetPath()`, it temporarily clears the binding-map flag at `BindingMap+0xc3c0`
so the default preset always loads `Bindings`.

### `Grid::LoadEntireState` (`.geist` root) [decomp ev_b8366a5a]

1. `StateVersion` float, default 0. **If `StateVersion < 2.0` the function returns false and loads
   nothing** (no older-format branch). Saves write 2.1.
2. `TimeInfo::Reset`, `ForceClearAllSamples`.
3. In this order: `FileManager` (child, `AudioFileManager`), `MasterChannel`, `Bindings` (only if
   `BindingMap+0xc3c0 == 0`), `Scenes` (`SceneManager`), `Song` (`SongSequencer`), `TimeInfo`,
   `Sampler`, `Metronome`.
4. `CurrentEngine` int (default 0), `PreviewVolume` float (keep), `PreviewOutput` string →
   `AudioOutputs::GetOutputLocationForName`.
5. `SendMIDINotes`, `SendMIDICCs` bool, **default true** when missing.
6. `Engine0`..`Engine7`, `GlobalAux0`..`GlobalAux3`.
7. `AudioFileManager::UnloadSingleRefSamples`, `StartFileRequestRoutine`, a virtual post-load
   hook (`Grid` vtable +0x9e8), editor message 0x10c.

`Grid::SaveEntireState` (ev_a13c55d8) writes the same names in the same order, except
`SendMIDINotes`/`SendMIDICCs` come before `CurrentEngine`.

Note the sample table (`FileManager`) is read **before** the engines, so layer `AudioFileName`s
usually hit the cache.

## Sample paths: `XMLFile` + `AudioFileManager` [decomp]

`XMLFile::Initialize` (ev_5c50de29) sets four flags and two paths:

| Field | Meaning |
|---|---|
| `+0x30` relative mode | `Options "AudioDeveloperMode"` && `Options "SaveRelativePathsWithChunks"` && !arg5 |
| `+0x31` restore outputs | arg4 (read by `SerializeOutputState`) |
| `+0x32` / `+0x33` | absolute paths / copy samples ("with chunks"); `= !arg1 / arg1`, both 0 in relative mode |
| `+0x38` | the document path |
| `+0x70` | the document path with its extension replaced by `samples` (`Song.geist` → `Song.samples`) |

`XMLFile::ConvertSamplePath(path)` (ev_e162a1c9), load direction:

1. A REX path ending in `.truncated` is stripped of that extension (and re-added at the end).
2. If the path exists on disk as given, use it.
3. Else start from the `.samples` folder path (`+0x70`), make the stored path use host separators
   (so v1's `\` separators work), then for each leading `../` drop one component of the base
   (the first drop removes `.samples` itself, so `../x.wav` is relative to the document's folder,
   `../../x.wav` to its parent).
4. A `_RELATIVE_` prefix drops the `.samples` component and is removed (path relative to the
   document's folder).
5. Append the rest. If that file exists return it, else return the empty string.

So a bare file name resolves into `<doc>.samples/`, `_RELATIVE_sub/x.wav` to `<doc dir>/sub/x.wav`.

Save direction: relative mode writes `MakePathRelative(doc dir, path)`, prefixed `_RELATIVE_` when
it does not start with `../`; "with chunks" mode writes only the file name and
`XMLFile::SaveAudioFile` (ev_62ec1379) copies the sample into `<doc>.samples/` (creating it,
mode 0777) unless already there; otherwise the absolute path is written. A path that does not
exist is written as empty.

`AudioFileManager::SerializeAsXML` (ev_3b365518), the root `FileManager` child:

| Attribute | Type | Notes |
|---|---|---|
| `Ver` | float | 2.0, write only |
| `NumberOfSamples` | int | read with `ReadAttribute`, default 0 |
| `AudioFile%dPath` | child `VString` (`string` attr) | 0..N-1 |

Load: `UnloadSingleRefSamples`, clear the missing-files list (`+0xe0`), set "loading" (`+0xc1`).
For each path: `ConvertSamplePath`; if it resolves, `LoadSample` (preload into the cache); if it
does not resolve or the load fails, push the **original** path string onto the missing-files list.
Save: skips `InMemorySamplerFile.sampler` (the sampler's recording buffer) and files the delegate
(`+0x28`, vtable +0x10) rejects ("Skipping audio file on path…"), writes the converted path,
then `SaveAudioFile`. Only works when the archive is an `XMLFile` (`IsKindOfClass` check).

`Layer` `AudioFileName` load (ev_7c48010c): string; if non-empty, `ConvertSamplePath`, then
`AudioFileManager::GetFileWithIdentity(path, true, engineIndex)` (ev_a54b476f: strips
`.truncated`; if the file exists → `LoadSample`, else look it up in the cache by identity).
If still nothing, queue `AudioFileManager::RequestAudioFileLoad` with the layer's
`StartEndMarkers`/`LoopMarkers` (async load). Saving with no sample writes `AudioFileName=""`.

## Shared helpers [decomp]

| Function | Attributes | Load behaviour | Evidence |
|---|---|---|---|
| `MixableObject::SerializeMixState` | `Gain` float, `Pan` float, `Mute` bool, `Solo` bool | keep; sets an internal field `+0x1c = 2` on load | ev_93a99c2e |
| `AudioOutputObject::SerializeOutputState` | `OutputLocation` string | read **only** if the archive is not an `XMLFile` or its restore-outputs flag (`+0x31`) is set; missing → `"Parent"`. Names below | ev_ac6800d7 |
| `AuxSendingObject::SerializeAuxSends` | `AuxSend0`..`AuxSend3` float | keep | ev_d8194a7f |
| `LockableObject::SerializeLockState` | `LockState` bool | keep | ev_3f50143c |
| `FixedString::SerializeAsXML` | `string` (char*) | set only if present and non-null | ev_458852d7 |

Output location names (`AudioOutputs::GetNameOfOutputLocation(id, false)`, ev_a80119bf; the
`true` form gives the short `S n`/`B n` labels for the UI): 0 `Parent`, 1 `Master`,
2..16 `Sub output 1`..`Sub output 15`, 17..20 `Buss 1`..`Buss 4`, else `ERROR`.

## Objects [decomp]

Order = file order. "keep" as defined above.

### `Engine` (ev_b56901c0)

| Name | Type | Notes |
|---|---|---|
| `Ver` | float | 2.0, write only |
| mix, output | helpers | |
| `EffectChain` | child | |
| aux sends | helper | |
| `SwingTemplate` | child | |
| `EngineState` | child | |
| `Pitch` | float | keep |
| `TempoNumerator`, `TempoDenominator` | int | keep |
| `SwingTemplateDepth` | double | keep |
| `StepPattern0`..`StepPattern23` | child | 24 patterns. Save writes only patterns where `!StepPattern::IsDefaulted()`; a missing one is reset with `StepPattern::Initialize(engine, index)` |
| `Pad0`..`Pad15` | child | |
| `EnginePadMode` | string | `Normal`(0) `Chromatic`(1) `Velocity`(2) `Engine M+S`(3); unknown → 0 (ev_0b154fe7, ev_3c4003b1) |

### `EngineState` (ev_1e21bd43)

Load first calls `EngineState::Initialize()`, so every missing attribute gets its init default.

| Name | Type | Notes |
|---|---|---|
| `Ver` | float | 2.0, write only |
| `PatternPower` | bool | |
| `CurrentPattern`, `CurrentPad`, `CurrentStep`, `FullLevels` | int | |
| `FullLevelEnabled`, `NoteEraseEnabled`, `NoteRepeatEnabled`, `NoteRepeatSynced` | bool | |
| `PadState0`..`PadState15` | child | |
| `P%dZR` (0..23) | int | per pattern (probably editor zoom); default 1, written only if ≠ 1 |
| `P%dSP` (0..23) | float | per pattern (probably editor scroll); default 0, written only if ≠ 0 |

### `PadState` (ev_d9a27c1e)

| Name | Type | Default on load |
|---|---|---|
| `Ver` | float | 2.0, write only |
| `Avis` | bool | false |
| `CurrentLayer` | uint | 0 |
| `Atype` | int | -1 |
| `MTrack0`..`MTrack3` | int | -1 (written only if ≠ -1) |

### `Pad` (ev_c1a9a96d)

| Name | Type | Notes |
|---|---|---|
| `Ver` | float | 2.0, write only |
| lock, mix, output | helpers | |
| `EffectChain` | child | |
| aux sends | helper | |
| `V2A` | float | velocity → amp |
| `Layer0`..`Layer7` | child | save writes only layers with `Layer::IsWaveformValid()`; a missing layer is reset (layer vtable +0x78 called with `true`) |
| `PadMode` | string | `All layers`(0) `Velocity split` `Round robin` `Round robin reset` `Random`(4); unknown → 0 (ev_be45bacf, ev_d7d418d6) |
| `PadName` | child `FixedString` | missing or empty → `Pad::SetNameToDefault` |

### `Layer` (ev_7c48010c)

`Ver` is read: local initialised to **3.2**, so a missing `Ver` counts as current.

| Name | Type | Notes |
|---|---|---|
| `Ver` | float | read; see upgrades |
| lock, mix, output | helpers | |
| `EffectChain` | child | |
| aux sends | helper | |
| `VelocitySplit` | child `LayerVelocitySplit` | |
| `StartEndMarkers`, `LoopMarkers` | child `AudioFileMarkers` | |
| `LayerName` | child `FixedString` | |
| `IsReversed` | bool | |
| `PhasePosition` | float | |
| `UsesTimestretch`, `TSTracksBPM` | bool | |
| `SourceBPM`, `DestinationBPM` | float | |
| `V2C`, `V2P` | float | velocity → cutoff / pitch |
| `EnvTracksPitch` | bool | missing on load → false |
| `LayerGain` `LayerPan` `CoarseTune` `FineTune` `PreDelayTime` `Cutoff` `Resonance` `FilterType` `FilterDrive` `FilterAlgorithm` `FilterActive` | float | names from `Layer::GetNameOfParameter` 0..10 (ev_163ab894); all stored as float |
| `Envelope%dParam%d` | float | envelope 0..1 (0 = amp, 1 = free env), param 0..7 = Attack, Decay, Sustain, Hold, Release, Attack curve, Decay curve, Release curve (ev_04997009) |
| `ModRouting0`..`ModRouting3` | child `LayerModulationRouting` | |
| `EnvelopeType0`, `EnvelopeType1` | string | written as `AHR`(0)/`ADSR`(1). **On load the value is read and ignored: both are forced to 1 (ADSR)** |
| `CutMode` | string | `Not cut`(0) `Self` `Pad` `By other pads` `Self + Other` `Pad type` `Engine group 1` `Engine group 2` `Global group 1` `Global group 2`(9) (ev_6fd388e9, ev_dadf09bb) |
| `PlaybackMode` | string | `One shot`(0) `Gated`(1) `Looping`(2) (ev_72c3636e) |
| `TSMode` | string | `Default`(0) `Fast`(1) (ev_c7e1ef23) |
| `AudioFileName` | string | see "Sample paths"; only on `XMLFile` archives |

Load-time upgrades (only when loading):

- `Ver < 3.2`: `Cutoff` c → clamp01((3·ln2 + ln((c⁴·22020 + 30)/55)) / (12·ln2)), i.e. the old
  value meant f = c⁴·22020 + 30 Hz and the new one is log2(f/55)/12 + 0.25.
  `Resonance` r → clamp01((r·1200 − 101)/909).
- `Ver < 3.0`: after reading each envelope, its Hold (`Param3`) is set to −1. Then, for each
  envelope whose A=0, D=0.5, S=1, R=0.001 (the old defaults), Decay is set to 0.02.
- Finally `Layer::ResetHoldTime(true)` if `Ver < 3.0` or either Hold is negative;
  `UpdateAllAudioMarkers` always.

`Layer::ResetHoldTime(bool pitch)` 0xae910 (ev_97d7ba71; constants read from `__const`,
`powf` exponent from the disassembly at 0xaeac0) [decomp, file]. Only with a loaded sample
(rate `sr`, `frames`):

1. t = (EndSample − StartSample)/sr of `StartEndMarkers`, at least 0; if |t| < 1e-6, the whole
   file: frames/sr.
2. With `pitch` (the load call passes true) and `UsesTimestretch` off: t /= 2^((coarse + fine)/12)
   (skipped if that ratio is ~0), where coarse = `ConvertNormalisedValueToPitch(CoarseTune)` =
   round(c·24) − 12 semitones (ev_d199da8c) and fine =
   `ConvertNormalisedValueToPitchInCents(FineTune)` = 2f − 1 semitones (ev_439ec203).
3. x = t − (A⁴·10 + (S ≠ 1 ? D⁴·10 : 0) + R⁴·15) with the amp envelope (`Envelope0`) values,
   clamped to 0..`Envelope::kMaxHoldInSeconds` (60).
4. Hold = (x/60)^0.25, clamped to 0..1, written to **both** envelopes' Hold.

Geist2's `Layer::ResetHoldTime` (as ported in Dreist's `geist_engine::hold`) is the same
computation, except that it uses CoarseTune unrounded. The two agree for coarse values on v1's
semitone grid (the only values v1's UI writes).

### `LayerVelocitySplit` (ev_33d709c0)

`Ver` 2.0 (write only); `LowPoint`, `LowTurn`, `HighTurn`, `Highest` int, each clamped to ≤ 127 on load.

### `LayerModulationRouting` (ev_60a0a2a7)

`Ver` 1.0 (write only); `ModulationDepth` float; `ModulationDestination` string:
`No destination`(0) `Pitch` `Cutoff` `Resonance` `Drive`(4), unknown → 0 (ev_89296b88, ev_27270727).

### `AudioFileMarkers` (ev_86482fb7)

`FileMarkerVersion` 2.0 (write only); `StartSample`, `EndSample`, `SampleLength` int.

### `StepPattern` (ev_5f8961fe)

| Name | Type | Notes |
|---|---|---|
| `Ver` | float | 2.0, write only |
| lock | helper | |
| `PatternName` | child `FixedString` | |
| `PatternSteps` | int | |
| `MonoModeActive` | bool | |
| `Track0`..`Track15` | child | load: each track is first sized to `PatternSteps` (`AllocateSteps` or `InitializeSteps`). Save: no `Track%d` at all if all 16 are `IsEmpty(true)` |
| `SyncMode` | string | written `Slave`(0) `Slave restart`(1) `Gate`(2) `Gate restart`(3); read also accepts the older names `Sync`→0, `Trigger`→1, `Shot`→3; unknown → 0 (ev_9fa80493, ev_693f3175) |
| `Resolution` | string | `1` `1T` `1/2` `1/2T` `1/4` `1/4T` `1/8` `1/8T` `1/16` `1/16T` `1/32` `1/32T` `1/64` `1/64T` (0..13); name default `1/16` (ev_3ee069b1) |

### `Track` (ev_2b2d8acb)

v1 has a `Track` class (16 per pattern).

| Name | Type | Notes |
|---|---|---|
| `Ver` | float | read; local default 2.2. If `Ver` is missing, `TrackVersion` is read instead |
| lock, mix | helpers | `Gain Pan Mute Solo` per track |
| `NumberOfSteps` | int | |
| `S%d` | child `Step` | save: only steps with `IsStepOn()`. Load: steps sized to `NumberOfSteps`; a missing `S%d` is reset |
| `Steps` | child `VObjectArray` of `Step` | legacy: read instead of `S%d` when `Ver ≠ 2.1` and the child exists |
| `<Lane>Rand`, `<Lane>Scal`, `<Lane>Comp` | float | per step lane (77, names below with spaces removed); default 0.5, written only if ≠ 0.5 |

### `Step` (ev_f1c8d4e3)

`Ver` 2.0 (write only); `State` bool; then one float per step lane, attribute name =
`Step::GetNameOfStepGraphItem(i)` with all white space removed (ev_3060d6bc), written only if it
differs from the lane default; missing → default.

| i | Lane | Default | | i | Lane | Default |
|---|---|---|---|---|---|---|
| 0 | `Velocity` | 1.0 | | 12 | `FilterType` | 0.0 |
| 1 | `Pan` | 0.5 | | 13 | `FilterDrive` | 0.5 |
| 2 | `Repeats` | 0.0 | | 14 | `ForceLayer` | 0.0 |
| 3 | `Shift` | 0.5 | | 15..18 | `Send1`..`Send4` | 0.5 |
| 4 | `Reverse` | 0.0 | | 19, 20 | `Ampenvdepth1`, `Ampenvdepth2` | 0.5 |
| 5 | `CoarsePitch` | 0.5 | | 21, 22 | `Freeenvdepth1`, `Freeenvdepth2` | 0.5 |
| 6 | `FinePitch` | 0.5 | | 23 + 9k + 0 | `FX{k+1}Active` | 1.0 |
| 7 | `StartPoint` | 0.5 | | 23 + 9k + 1 | `FX{k+1}Mix` | 1.0 |
| 8 | `Duration` | 1.0 | | 23 + 9k + 2 | `FX{k+1}OutputLevel` | 1.0 |
| 9 | `LinkToPrevious` | 0.0 | | 23 + 9k + 3..8 | `FX{k+1}Param1`..`Param6` | 1.0 |
| 10 | `FilterCutoff` | 0.5 | | | (k = 0..5, i up to 76) | |
| 11 | `FilterResonance` | 0.5 | | | | |

(Source strings: `Amp env depth 1`, `Free env depth 1`, `FX1 Output Level`, … .)
`GetStepGraphItemForName` also accepts `Volume` for lane 0 (ev_fa1a72f5).

### `SwingTemplate` (ev_75db4209)

`Ver` 2.0 (write only); `SwingTemplateName` child `FixedString`; `NumberOfSteps` int;
`S0`..`S127` double, written only if ≠ 0; load falls back to `Step%d`, then 0.

### `SongSequencer` = `.geist` root `Song` (ev_688e9ad0)

| Name | Type | Notes |
|---|---|---|
| `Song` | child `Song` (pointer) | |
| `SongActive`, `SongLoopEnabled` | bool (`SerializeBoolAttribute`, default false) | |
| `SongLoopStart`, `SongLoopEnd` | double | `SongPart::QuantizeTimeRound` on load |
| `SongEditorZoom`, `SongEditorStart` | double | |

`Song` (ev_861f90dc): `Track0`..`Track7` = `SongTrack` (one per engine). `SongTrack`
(ev_9826a77c): `Part0`..`PartN` until the first missing one (load clears first). `SongPart`
(ev_406c5fa4, cleared on load): `mute` bool, `starttime` double, `length` double, `patt` uint,
`phase` double.

### `SceneManager` = `.geist` root `Scenes` (ev_fd4e6e22)

`Scene0`..`Scene71` (72 scenes), `CurrentScene` int, `EditorBase` uint.
`Scene` (ev_6ee860cd): `used` bool (default false; unused scenes are cleared on load and write
only `used` and `name`), `name` child `FixedString`, then only if used: `mode` enum
`both`/`pat`/`mix`, `start` enum `imm`/`beat`/`bar` (`SerializeEnumAttribute` with
`Scene::kModeStrings` / `kStartModeStrings` [file]), `Eng0`..`Eng7` = `SceneEngine`
(ev_2c1f657b): `patt` uint, `pattpower` bool.

### `MasterChannel`, `GlobalAux`, `Metronome`, `TimeInfo`

- `MasterChannel` (ev_47d5e525): `Ver` 2.0, mix, output, `EffectChain`, `Pitch` float.
- `GlobalAux` (ev_f6de6e53): `Ver` 2.0, mix, output, `EffectChain`, `IsPreFade` bool.
- `Metronome` (ev_9fd05cf9): `Ver` 2.0, mix, output.
- `TimeInfo` (ev_89f6ee6c): `Tempo` double, `TSN` uint, `TSD` uint, `Metro` bool,
  `Quantize` bool; load recomputes 1/tempo.

### `EffectChain` and `Effect`

`EffectChain` (ev_597a41f8): `FXVer` 2.1 (write only); `Effect0`..`Effect5` (save only slots
whose type ≠ 0; a missing slot loads as type 0, "No Effect"); `IsActive` bool;
`ChainPresetName` child `FixedString`; `ChainMix` float, default 1.0.

`Effect` (ev_82ed17a2): `EffectVersion` 2.0 (write only); `IsActive` bool, `Mix` float,
`Level` float, `PresetName` child `FixedString`, `Parameter0`..`Parameter5` float,
`EffectType` string. Load: `GetEffectTypeForType(name)` → `SetEffectType(type, songInfo, false)`
then pushes the six parameter values; the bypass/mix/level state is mirrored into the
"previous" fields. Same shape as the factory `.fxpreset`/`.fxchain` files and Geist2.

### Other serializers seen

- `SlicePoint` (ev_3fa78e20): `Ver` 2.0; `Start` `End` int; `Locked` `Deleted` bool; `Pos` int;
  `Strength` `DynSense` `Mean` float; `Type` string (`GetNameOfSliceType`).
  `SliceAnalysis` (ev_3845204b): `Ver` read (default 2.1); `Slices`, `ZX`, `Meta` children
  read only if `Ver ≥ 2.1`.
- `AudioFileMetaData` (ev_3a1b344c): `Ver` int 2; `Name` `Author` `Desc` children; `BPM`
  `Beats` `Bars` `Pitch` float.
- `CEngine`, `CPad`, `CLayer`, `CPattern`, `CTrack`, `CStep`, `CGroove`, `CScene` are the
  Guru import (`GuruFileImporter`, `GuruConversionJob`), not v1's own format.

## Defaults for attributes the loader leaves alone [decomp]

Where a loader table above says "keep", a missing attribute keeps the value the object was
constructed or reset with. These come from the constructors, `Initialize` and `ResetState`
bodies. 1/∛2 means `0x3f4b2ff5` (0.7937, 0 dB on Geist's gain curve).

| Object | Defaults | Evidence |
|---|---|---|
| mix (`MixableObject`) | `Gain` 0.7937, `Pan` 0.5, `Mute` false, `Solo` false | ev_7757985d |
| output (`AudioOutputObject`) | `OutputLocation` 0 = `Parent` | ev_5ad33fc9 |
| aux (`AuxSendingObject`) | `AuxSend0..3` 0 | ev_f087e40f |
| lock (`LockableObject`) | `LockState` false | ev_5ba9be32 |
| `Engine` | `Pitch` 0.5, `TempoNumerator` 1, `TempoDenominator` 1, `SwingTemplateDepth` 0, `EnginePadMode` 0 (`Normal`); `Engine::Initialize` also loads the factory swing template named `Default` | ev_68d18606, ev_c210c31c, ev_cdaeedeb, ev_5fe2883b |
| `EngineState` | `PatternPower` true, `CurrentPattern`/`CurrentPad`/`CurrentStep` 0, `FullLevels` 127, `FullLevelEnabled` false, `NoteEraseEnabled` false, `NoteRepeatEnabled` false, `NoteRepeatSynced` true, `P%dZR` 1, `P%dSP` 0 | ev_ab61573a |
| `PadState` | `Avis` false, `CurrentLayer` 0, `Atype` −1, `MTrack0..3` −1 | ev_7ceb4270 |
| `Pad` | `PadMode` 0 (`All layers`), `V2A` 1.0, `PadName` `Pad %d` (index + 1) | ev_13a0dc00, ev_b70b3fed, ev_40363409 |
| `Layer` | `CutMode` 1 (`Self`), `PlaybackMode` 0 (`One shot`), `IsReversed` false, `PhasePosition` 1.0, `EnvTracksPitch` false, `V2C` 0.5, `V2P` 0.5, `UsesTimestretch` false, `TSTracksBPM` false, `SourceBPM` 120, `DestinationBPM` 120, `TSMode` 0, both `EnvelopeType` 1 (ADSR) | ev_ab7a5d74 |
| `Layer` parameters (`GetDefaultLayerParameter`) | `LayerGain` 0.7937, `LayerPan` 0.5, `CoarseTune` 0.5, `FineTune` 0.5, `PreDelayTime` 0, `Cutoff` 1.0, `Resonance` 0, `FilterType` 0, `FilterDrive` 0.3, `FilterAlgorithm` 0.3, `FilterActive` 0 | ev_d1ec4c75 |
| `Layer` envelopes (`ConstructDefaultEnvelopes`, both) | Attack 0, Decay 0.02, Sustain 1, Hold −1, Release 0.001, the three curves 0.5 | ev_f6924262 |
| `LayerVelocitySplit` | `LowPoint` 0, `LowTurn` 0, `HighTurn` 127, `Highest` 127 | ev_84c92766 |
| `LayerModulationRouting` | `ModulationDestination` 0 (none), `ModulationDepth` 0.5 (centre = no modulation) | ev_59996834 |
| `StepPattern` | `SyncMode` 0 (`Slave`), `Resolution` 8 (`1/16`), `PatternSteps` 16, `MonoModeActive` false, `PatternName` `Pattern %d` | ev_fcca7ebf, ev_455f32b7, ev_ba5f04d9 |
| `Track` | `NumberOfSteps` 16, mix as above, every `<Lane>Rand/Scal/Comp` 0.5 | ev_879ae739 (the constructor body decompiles as bad data, ev_114428fa) |
| `Step` | `State` false, lanes at `GetDefaultValueOfStepGraphItem` (= the Step table above) | ev_3c8afa7a, ev_21e856d3 |
| `SwingTemplate` | `NumberOfSteps` 1, all `S%d` 0, name `Empty Template` after `ResetState` (the constructor sets `Default Template` first) | ev_379b41fb, ev_c3c41556 |
| `Effect` | `IsActive` false, `Mix` 0.5, `Level` 1.0, type 0, `Parameter0..5` 0, `PresetName` `Default` | ev_ed96afed, ev_8a46e236 |
| `EffectChain` | `IsActive` true, `ChainMix` 1.0, `ChainPresetName` `Default`, six empty effects | ev_5a12c4e5, ev_cbf18761 |
| `MasterChannel` | `Pitch` 0.5 | ev_db8ccdc4 |
| `GlobalAux` | `IsPreFade` = `Options "AuxesArePreFade"` (default false) | ev_4827433c |
| `Metronome` | `Mute` true, `OutputLocation` 1 (`Master`) | ev_4c3098ed |
| `TimeInfo` | `Tempo` 120, `TSN` 4, `TSD` 4, `Metro` false, `Quantize` false | ev_ff546833, ev_a274a319 |
| `SongSequencer` | `SongActive` false, `SongLoopEnabled` false, `SongLoopStart` 0, `SongLoopEnd` 16, `SongEditorZoom` 16, `SongEditorStart` 0 | ev_38453b65, ev_ae907c41 |
| `SongPart` | `mute` false, `starttime` 0, `length` 0, `patt` 9999 (none), `phase` 0 | ev_864aca4f |
| `SceneManager` / `Scene` / `SceneEngine` | `CurrentScene` 0, `EditorBase` 0; scenes cleared (`used` false); `patt` 0, `pattpower` false | ev_89622fd1, ev_b6016faa, ev_96d7dd23 |
| `Sampler` | `recordsource` 1, `inputgain` 1, `outputgain` 1, `startthreshold` 0.1, `stopthreshold` 0.005621 (`0x3bb827fa`), `inputthru` false, `beatsmode` true, `recbars` 1, `recbeats` 0, `recsecs` 2.0, `startmode` 0, `timedmode` 0 | ev_0be721ab |

`Scene::ClearSelfOnly` and the `SceneManager` constructor body were not decompiled; the scene
defaults above follow from `Scene::SerializeAsXML` clearing unused scenes. `Pattern %d` is the
format string present in the binary; `SetNameOfPatternToDefault` itself was not decompiled
(index base assumed 1, as for pads) [guess].

## Sampler, BindingMap, MIDIMap [decomp]

`Sampler` (`.geist` root `Sampler`, ev_3df21e0a): `version` int (written 2; missing → 1).
`mitmsource` string (ManInTheMiddle producer, out of scope) else `recordsource` string
(`GetRecordSourceFromName`). Only when `version ≥ 2`: `inputgain` `outputgain`
`startthreshold` `stopthreshold` float, `inputthru` `beatsmode` bool, `recbars` `recbeats` uint,
`recsecs` float, `startmode` enum `Immediate`/`MIDI`/`Host`/`Threshold`, `timedmode` enum
`Free`/`Timed`/`Loop` (`strStartModes`/`strTimedModes` [file]). Loading also writes these to the
`SamplerLast*` options.

`BindingMap` (`.geist` root `Bindings`, ev_e4e57dbe):
- `HostParameters`: child `VObjectArray` of 32 `VValue` (host automation values, missing → 0).
- For each of 32 host parameters: `HostBindingCount%d` int, then `Auto%dBinding%d` `Binding` children.
- For 16 MIDI channels × 128 CCs: `MIDI%dCC%dBindingCount` int, then `MIDI%dCC%dBinding%d`
  `Binding` children.

`Binding` (ev_1e06779d): `Min` `Max` float, `MIDI` bool, `MajorIndex` `MinorIndex` int,
`TargetMode` `EnterTargetMode` `EditorOnly` bool, `Path` and `Name` `FixedString` children, then
`MessageStuffer`, `MessagePopper` (child objects), a `Message` prototype and `Type`.
`Message` (ev_aa4127e9): `EngineIndex` `PadIndex` `LayerIndex` `PatternIndex` `TrackIndex`
`StepIndex` int, `MessageValue%d` children.

`MIDIMap` (ev_93038a78) writes only `Ver` int 2; `MIDIMapItem` (ev_b99eacf2): `Engine` `Pad`
`Pattern` `Scene` int, `DataPoint0..2` int.

## Effect types and the v1-only DSP [decomp]

`Effect::EffectTypeToString` (ev_16b78c41), `GetEffectTypeForType` (ev_eca51da3),
`GetNumberOfParametersForEffect` (ev_b3c0e875), `GetCategoryForEffect` (ev_6494cf3e),
`GetParameterNameForEffect` (ev_d5606ed0). DSP class from `Effect::AddAllSupportedEffects`
(0xf6aa0, ev_0be0368e), which registers `VDSPNodeAdaptor::AddStaticEffectType(EffectTypeToString(t),
class)` for t = 1..32; `Effect::SetEffectType` (ev_12d86655) selects node `t − 1`.

Categories (`GetCategoryForEffect`; group labels are my reading): 0 = types 14–16,
1 = 1, 2, 6–9 (gain/dynamics), 2 = 3–5 (EQ), 3 = 18–26 (filters), 5 = 28–32 (reverbs),
4 = everything else (10–13, 17, 27).

| t | Name | DSP class | Params (`Parameter0..`) |
|---|---|---|---|
| 0 | No Effect | — | — |
| 1 | Gain | **`VAmpM`** (v1-only) | Gain |
| 2 | Stereo Width | `VStereoMixer` | Gain, Width |
| 3 | Parametric EQ | `VEQSSLESeriesS` | Low Q, Low frequency, Low gain, High Q, High frequency, High gain |
| 4 | DJ EQ | `VEQSSLESeriesS` (same class) | Low gain, Mid gain, Mid frequency, High gain |
| 5 | Vari-EQ | **`VFilterEQ2`** (v1-only) | Mode, Frequency, Q, Gain |
| 6 | Channel Compressor | `VCompressorUrei1176S` | Input, Attack, Release, Ratio, Output |
| 7 | Bus Compressor | `VCompressorSSLMixbusS` | Threshold, Attack, Ratio, Release, Makeup, Analogue limit |
| 8 | Noise Gate | `VNoiseGateS` | Attack, Hold, Release, Threshold, Hysteresis, LPF |
| 9 | Envelope Shaper | **`VDSPTransientDesigner`** (v1-only) | Attack, Sustain, Gain, Sensitivity, Linked |
| 10 | Delay | `VDelayStereo` | Delay time, Feedback, Low cut, Hi cut, Swing, Sync |
| 11 | Flanger | `VFlangerS` | Frequency, Depth, Position, Feedback, Spread, Phase |
| 12 | Chorus | `VChorusS` | Frequency, Depth, Spread |
| 13 | Phaser | `VFilterPhaser8` | Drive, Mode, Pitch, Resonance, Frequency, Depth |
| 14 | Distortion | `VAudioShaperS` | Mode, Drive, Hi Cut, Low Cut, Tone, Clean |
| 15 | Bit Crusher | `VBitCrusherS` | Drive, Frequency, Bits, Hi Cut, Low Cut, Tone |
| 16 | Ring Mod | `VRingModS` | Drive, Type, Pitch |
| 17 | Frequency Shifter | `VFreqShifterS` | Pitch, Drive |
| 18 | Filter | `VFiltersSSLChanS` | HP active, HP frequency, LP active, LP frequency |
| 19 | Modulated Filter | `VFilterAutoS` | Mode, Cutoff, Resonance, Attack, Decay, Env depth |
| 20 | Comb Filter | `VFilterComb` | Drive, Mode, Pitch, Resonance |
| 21 | Three Oh! Three Filter | `VFilterTB303` | Drive, Mode, Pitch, Resonance |
| 22 | Hoover Filter | `VFilterJuno106` | Drive, Mode, Pitch, Resonance |
| 23 | Grandad Filter | `VFilterMoog4` | Drive, Mode, Pitch, Resonance |
| 24 | Little Fatty Filter | `VFilterMS20` | Drive, Mode, Pitch, Resonance |
| 25 | Child Genius Filter | `VFilterMoogProdigy` | Drive, Mode, Pitch, Resonance |
| 26 | Japan Filter | `VFilterSH101` | Drive, Mode, Pitch, Resonance |
| 27 | Audio Freezer | `VAudioFreeze` | Gate, Loop Length, Grain Length, Smooth, Scratch, Speed |
| 28–31 | Breverb Hall / Plate / Room / Inverse | `VDSPBreverbHall` / `Plate` / `Room` / `Inverse` | Reverb Time, Pre Delay, Diffusion, Size, Spread, Shape |
| 32 | Tin Can Verb | `VReverbS` | Decay, Size, Damping, Pinch, Squeeze |

`GetNumberOfParametersForEffect` agrees with these name lists, except that the reverbs share one
six-name list while the counts differ: Breverb Hall 6, Plate 5, Room 5, Inverse 3, Tin Can Verb 5.

v1-only DSP, per class [decomp, sym]:

- **`VAmpM`** (Gain, t = 1): mono voice. Gain parameter clamped to ±10, turned into a linear
  gain with a cubic 2^x approximation (`((x·0.0794 + 0.2245)·x + 0.6961)·x + 1` on the fraction,
  exponent bits for the integer part, −0.01637), negative results clamped to 0, then a one-pole
  smoother (coefficient from `SongInfo+0x2c`) per sample (ev_9b98b0fe).
- **`VFilterEQ2`** (Vari-EQ, t = 5): `VFilterEQCommon::SetCoeffs(mode clamped 0..8,
  freq = 2^(p1+1)·400·SongInfo[+0x24], Q, gain)` then a double-precision biquad with an
  alternating ±tiny anti-denormal term (ev_b33db815). Also used directly by the GUI curve
  (`EditorEffectVariEQ::ATC_Tick`, 0x341550).
- **`VDSPTransientDesigner`** (Envelope Shaper, t = 9): a node wrapper around a
  `TransientDesigner` object (`CreateNewTransientDesigner`, `TDImpl`): forwards changed
  parameters through its vtable (+0x40), processes (+0x60), and reports a meter value (+0x70)
  (ev_ed247fe9, ev_2726447b). Editor panel: `EditorEffectTransientDesigner`.

Geist2 has the stereo counterparts `VAmpS`, `VFilterEQ2S` and `VDSPTransientDesignerS` (plus the
shared `VFilterEQCommon` and `VDSPTransientDesignerConvertParam`) [sym, Dreist `re/out/symbols.txt`].
So the v1-only classes are mono versions of DSP Dreist already ports. What differs, recovered
2026-11-03 and implemented in `crates/einst-dsp` [decomp, file]:

- **Stereo handling.** `VDSPNodeAdaptor::ProcessStereo` 0x4f4060 (ev_69fac40b) →
  `Process22` 0x4f3840 (ev_964c5456): a node with one audio input and output runs as two
  instances, left through the first and right through the second. Each v1 effect therefore keeps
  separate state per channel.
- **Parameters.** `Effect::PushParameterValue` 0x2efc60 (ev_a735042f): Gain and Vari-EQ map
  `ParameterN` to port N; Envelope Shaper (v1 type 9) to port N + 1. `SetupInternalParameters`
  0x2f0250 (ev_5107cdbc) sets the Envelope Shaper's port 0 (WetMix) to 1. Port tables:
  `VAmpM::InitializePorts` 0x536b50 (ev_cfa257f1; "Amp" amp-log, 0 dB, −oo..36 dB),
  `VFilterEQ2::InitializePorts` 0x520370 (ev_6e577663; Mode 0..8, Freq −4..4.5
  frequency-pitch, Q 0.25 default 0.5..4 double-log, Gain ±15 dB),
  `VDSPTransientDesigner::InitializePorts` 0x31ddc0 (WetMix, Attack, Sustain, Gain 0..1,
  Sensitivity %, Linked switch; `In1`/`Out1`). These are the same tables as Geist2's stereo
  classes.
- **Fast 2^x** (inlined in both voices): `i = floor(x − 0.5)`, `f = x − i`,
  `p = ((f·0.079440236 + 0.22449434)·f + 0.69606566)·f + 1`, result = `p` with `i` added to
  its exponent bits (`shll $0x17; addl` at 0x536a58). Within 0.4 % of 2^x.
- **Gain (`VAmpM`)**, `Voice::ProcessBuffer` 0x5369d0 (ev_9b98b0fe): x = Amp port clamped to
  ±10; a = fast2^x − 0.016367754, floored at 0; the gain ramps linearly from its last value to a
  over the block (`SongInfo + 0x2c` = 1/block size). `Voice::Reset` 0x536b10 sets the gain to 0.
  Dreist's `VAmpS` port uses an exact `exp2` here.
- **Vari-EQ (`VFilterEQ2`)**, `Voice::ProcessBuffer` 0x51fab0 (ev_b33db815):
  mode = trunc(clamp(m, 0, 8) + 0.5); f/fs = fast2^(Freq + 1)·`SongInfo + 0x24` (1/fs)·400;
  then v1's `VFilterEQCommon::SetCoeffs` 0x521520 (ev_a7794e4b; the cases are behind a jump
  table at 0x521664: mode 0 → 0x521b99 … mode 8 → 0x5216b0, read from the disassembly):
  - w = 2π·clamp(f/fs, **0.001**, 0.45) (Geist2's: 1e−5..0.45);
  - Q clamped to 0.5..1000;
  - A = `powf(10, g·0.025)`, except Bell, which uses double `pow`;
  - RBJ-cookbook coefficients for Bell, Low Shelf, High Shelf, LPF, BPF, BPF 0 dB, HPF, Notch and
    Allpass, normalised by a0;
  - a mode change clears the coefficients and history.

  Per sample: y = ((x2·b2 + x1·b1) − y1·a1) − y2·a2 + x·b0, with history x2 ← x1, y2 ← y1,
  x1 ← (x − d) in float, y1 ← y, where d is an anti-denormal term that starts at 1e−8
  (`CreateVoice` 0x51fe80, ev_d61d4f58) and flips sign every sample. `Voice::Reset` 0x51f920
  (ev_9a6184d3) clears the key and history but not d.
- **Envelope Shaper (`VDSPTransientDesigner`)**: the voice (`Voice::Voice` 0x31e4a0,
  ev_c1277f4f; `ProcessBuffer` 0x31e370, ev_ed247fe9) pushes changed ports into the node's
  `TransientDesigner` (`TDImpl`) and processes one channel. v1's `TDImpl` has the same time
  constants (`TDChannel::BuildCoeffs` 0x31f560, ev_d7897614: e.g.
  exp(−1/(2π·sr·39000·100e−9)) = exp(−40.809/sr)), the same parameter scaling
  (`ConvertParameter01ToParameterValue` 0x31f1d0, ev_8c86f1ce: Attack ±15, Sustain ±24,
  Gain ±12 dB) and the same constants in `TDChannel::Process` 0x31fb40 (compared as a set, not
  line by line) as Geist2's, which Dreist ports. `TDImpl::Process` 0x320830 (ev_8c1c8c76)
  links channels only within one instance. So in v1, with one instance per channel, **Linked
  never links left and right**. The base voice `Reset` is empty, so a panic keeps the
  parameters.

The effect menu (`EditorEffectPanel::DoShowEffectTypeMenu`, ev_ad49b2ff) lists "No Effect",
then the six categories of `Effect::EffectCategoryToString` 0xf4e00 (ev_b6c62234: Distortion,
Dynamics, EQ, Filters, FX, Reverbs) as submenus, each with its types in type order.

## Conversion to Geist2 (`einst_formats::geist2`)

`Document::to_geist2` turns a v1 project, kit, pad, engine, pattern or bank into the tree Dreist's
`geist_formats` and `geist_engine` read. Dreist has no v1 legacy branch of its own, so the
converter emits what Geist2's own loader ends up with when it reads a v1 file. Geist2 2.1.2.13
(sha256 `395d2ccf…e46d`) branches checked with REA [decomp]:

- `Geist::SerializeFromData` 0xb9380 (ev_0f43b7e5): below root version 3000 it applies v1's
  `StateVersion` gate (missing → 2.1, < 2.0 → nothing loaded) and reads the same instance names
  (`FileManager`, `Scenes`, `Song`, `Engine%d`, `GlobalAux%d`, …). The converter checks the gate,
  drops `StateVersion` and writes root `version="3000"`.
- `Layer::Serialize` 0x71c10 (ev_5075e9a1): a missing `Ver` reads as 3300; otherwise
  `round(Ver·1000)`. v1.1.1.2 layers are 3200. The 11 v1 layer parameters already carry Geist2's
  stored names (`LayerGain` … `FilterActive`). Below 3300:
  - `V2C`, `V2P` → Velocity depth `2v − 1` on `Cutoff` / `CoarseTune` (`Cutoff_Velocity`,
    `CoarseTune_Velocity`), only when present;
  - `UsesTimestretch`, `TSTracksBPM`, `TSMode` → `TimestretchActive`, `TimestretchTracksBPM`,
    `TimestretchMode` as 0/1 (`Fast` = 1); `SourceBPM`/`DestinationBPM` are read and discarded;
  - `Envelope{e}Param{i}` → `AmpEnv*` (e = 0) / `FreeEnv*` (e = 1) in the same order
    (Attack, Decay, Sustain, Hold, Release, three curves), clamped to 0..1. Below 3000, Hold is
    set to −1 and the old default decay (A 0, D 0.5, S 1, R 0.001) becomes 0.02;
  - `ModRouting0/1` (amp envelope) and `ModRouting2/3` (free envelope): if depth ≠ 0.5 and the
    destination is set, that source's depth `2d − 1` (clamped ±1) goes on `Cutoff`, `Resonance`,
    `FilterDrive` (v1 "Drive") or `CoarseTune` (v1 "Pitch"); e.g. `Cutoff_AmpEnvelope`.
  - Below 3200, Geist2's cutoff/resonance remap (`ln(c⁴·400.36 + 0.545) + 2.079)·0.1202`,
    `r·1.3201 − 0.1111`) is the same formula as v1's own `< 3.2` remap.
  - `EnvelopeType%d` is not read (v1 forces ADSR anyway). `CutMode`/`PlaybackMode` labels are
    the same strings in both.
  The converter writes `version="3300"` and these Geist2 attributes, and drops the v1-only ones.
- `Step::Serialize` 0x3096d0 (ev_6246f185): no version branch; it reads only Geist2's 93 lane
  names. v1 lanes with the same names pass through. v1's `Ampenvdepth1/2` and `Freeenvdepth1/2`
  lanes have no Geist2 name and are dropped (with their track `Rand/Scal/Comp` modifiers).
- `Track::SerializeFromData` 0x153070 (ev_f1cbe2c5): `Ver` 2.2 (and `TrackVersion`) → `S%d`
  children, the same as v1. `Ver` 2.1 → an older `Steps` array or `S%d`.
- Effects and chains need no change: the factory `.fxpreset` files that differ still have the
  same `Parameter0..5` meanings for the effects compared (Parametric EQ, Delay, Bus Compressor,
  Distortion: identical values in both installs) [file].

Also dropped: the `Bindings` map (v1's `Binding` message objects are not traced) and every
`Ver`/`FileMarkerVersion`. Sample paths: `_RELATIVE_x` → `../x` (Dreist resolves relative names
against `<doc>.samples/` too, so `../` and bare names keep their v1 meaning).

Guesses and deliberate choices (to confirm):

- **Hold on v1 3.x layers: decided (owner, 2026-11-03), recompute like v1.** v1 saves Hold = −1
  ("compute from the sample") and its loader calls `ResetHoldTime` for a negative hold. Geist2
  clamps the value to 0 on these layers and does not reset it. The converter keeps a negative
  Hold as −1 (other values are clamped to 0..1), so Dreist's loader recomputes the hold with the
  sample's region, rate and the layer's pitch and envelope: v1's formula above. Residual
  difference: for a CoarseTune off the semitone grid, v1 rounds it and Dreist does not.
- **Output `Buss n` → `Aux n`.** Both are the four global aux buses; Geist2's name for them is
  taken from Dreist's `Route::from_name`, not from a decompiled v1→Geist2 mapping.
- **FX-active step lanes.** v1's default for `FXnActive` is 1.0, Geist2's is 0. A v1 step that
  omits the lane reads Geist2's default, as Geist2 would.
- **Legacy `Steps` array shape** (Track `Ver` < 2.1): its children are taken as the steps in order.
- The 11 shared layer parameter names are assumed to mean the same in both versions (same names,
  same defaults; `FilterType`/`FilterAlgorithm` value ranges are not checked).
- Root `version="3000"` is what Dreist's empty project uses; Geist2's own saves may write a
  higher one.

## Unknown / not done

- The layer virtual at vtable +0x78 used to reset a missing `Layer%d` (assumed reset/clear).
- `BindingMap+0xc3c0` (the flag that skips `Bindings` on load), presumably "keep current MIDI
  bindings"; the option behind it is not traced.
- `AudioFileManager` delegate at `+0x28` that decides which samples are saved.
- `Binding`'s `MessageStuffer`/`MessagePopper`/`Type` child objects are not traced, so v1
  bindings are not converted.
- Engine, Pad, StepPattern, EngineState, Scene and Song serializers of Geist2 were decompiled
  (ev_b1ae35fd Pad, ev_5511ff9a StepPattern, ev_68511a0f EngineState) but only read for the
  names the converter relies on; Geist2's real `Engine` serializer is behind a vtable thunk
  (ev_85e43971) and was not followed.
- No real v1 user files exist to check these against (the factory install has none).

## Tooling

The first Ghidra analysis of this 14 MB engine takes about 4 minutes. A direct REA MCP call from
the desktop Code tab times out on the client side before that and the timeout closes the target,
so this pass drove `rea mcp` over stdio from a small script (`re/scripts/rea_drive.py`): it opens the
binary once and runs queued job lists (`procedure_pseudo_code`, `xrefs`, `procedure_callers`,
`resolve_containing_procedure`) against the same Ghidra session. Same REA 6.7.0 + Ghidra 12.1.4
provider and env as Dreist (`GHIDRA_INSTALL_DIR`, `JAVA_HOME`,
`REA_GHIDRA_STARTUP_TIMEOUT_MS=3600000`). Pseudocode calls `VXMLFile` methods through the vtable;
offsets map to names via `vtable for XMLFile` (e.g. 0x1d0 `ReadAttribute(float&)`,
0x1f0 `SerializeAttribute(VString&, default)`, 0x228 `SerializeAttribute(float&)`,
0x2e8 `SerializeObject(VObject&)`, 0x320 `IsLoading`, 0x328 `IsSaving`).
