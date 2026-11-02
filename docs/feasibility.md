# Geist v1.1.1.2 (macOS): can it be brought back to life?

Analysis date: 2026-11-02. Source: `~/Downloads/FXpansion.Geist.v1.1.1.2.MacOSX-Xdb.rar`.
Everything was unpacked in an isolated cloud sandbox. Nothing was executed, and the
licensing code was not examined.

## Short answer

Yes, and it is in better shape than most 2012-era plugins.

1. **Running the original is probably possible today.** The binaries already contain a
   64-bit Intel (x86_64) build. Every system function they call still exists in modern
   macOS, so it should load natively on an Intel Mac and through Rosetta 2 on Apple Silicon.
   This only lasts as long as Rosetta does (see the risks below).
2. **Rescuing your content is very feasible.** Geist's own files (kits, patterns, projects,
   effect presets) are plain XML written by TinyXML, with samples kept as normal WAV/AIFF
   files that the XML refers to by path. The binary still has its C++ symbol names, so
   the serializer for each object type can be read off directly.
3. **A native Apple Silicon rebuild is possible but big.** The sampler, slicer, step
   sequencer, scenes and song mode are well understood. The 30-odd effects are the hard part
   if you want them to sound identical.

## What's in the package

| Part | Contents |
|---|---|
| Engine `Geist.component` / `Geist.vst` / `Geist.dpm` (RTAS) | **The same 29 MB binary for all three** (identical MD5). Universal i386 + x86_64. VST2 entry `VSTPluginMain`. The AU is FXpansion's VST-to-AU wrapper (`CVST2AUPluginEntry`). |
| `GeistCocoaView.bundle` | Small Cocoa AU editor view, i386 + x86_64 |
| `Geist.app` (standalone) | 160 KB "StubLoader" that loads the VST from `/Library/Audio/Plug-ins/VST/Geist.vst` (Info.plist key `FxTargetBundle`) |
| `ManInTheMiddle.bundle` | Producer/consumer bridge that passes audio between instances, i386 + x86_64 |
| Presets package | 260 `.fxpreset` and 48 `.fxchain` effect presets, 35 pattern templates, 4 swing templates, metronome WAVs, 102 skin files with 643 PNGs, 4 PDF manuals |
| Not included | **No factory kits or sample library.** These were a separate content install. |

Build facts: the 10.5 SDK (AppKit 949), GCC (`libgcc_s` unwinder, statically linked C++
runtime), no code signature, no PowerPC code, no QuickTime. Third-party code: libsndfile
1.0.4, TinyXML, Propellerhead REX SDK 1.7 (loads the external *REX Shared Library* at
runtime for `.rx2`/`.rcy` files).

## Compatibility of the 64-bit build

I checked every imported symbol of the four x86_64 binaries against the x86_64 symbol lists
of the macOS 11.3 SDK (script: `tools/checksyms.py`).

| Binary | Imports | Missing |
|---|---|---|
| Engine (AU/VST/RTAS) | 494 | 0 (*) |
| GeistCocoaView | 13 | 0 |
| Standalone stub | 78 | 0 |
| ManInTheMiddle | 170 | 0 |

(*) The six vDSP FFT functions are imported through the old top-level path
`/System/Library/Frameworks/vecLib.framework`. The functions exist in
Accelerate's vecLib. If that legacy path won't load on a current macOS, one
`install_name_tool -change` fixes it.

The deprecated APIs it uses all still exist for x86_64: Carbon event-loop timers, Component
Manager, Multiprocessing Services (MP semaphores), `FSFindFolder`/`FSResolveAliasFile`,
`Gestalt`, Internet Config (`ICLaunchURL`), ColorSyncLegacy (`CMGetSystemProfile`) and Process
Manager. The GUI is NSView/CoreGraphics/CoreText based, not QuickDraw.

### Risks for running the original

- **AU registration is old-style.** The `.component` Info.plist has no `AudioComponents`
  entry; registration relies on a `thng` resource in `Geist.rsrc`. Current hosts and `auval`
  may not find it. Workarounds: use the VST2 or the standalone app, or add a small shim.
- **Host architecture.** On Apple Silicon the host must run as Intel (Rosetta) to load an
  x86_64 plugin in-process, or use a bridge. The standalone app avoids this issue.
- **Rosetta's lifespan.** Apple has said full Rosetta 2 support continues only through
  macOS 27, with a reduced subset after that. So this route buys time; it is not permanent.
  (This is from Apple's 2025 announcement as I remember it. Check it against current Apple docs.)
- **Licensing.** The engine is protected by FX License Manager. I did not look into it and
  won't work around it. If you own a license, whether it can still be activated is a question
  for the current rights holder. This archive is labelled as a cracked scene release, so
  the engine binary may differ from the retail one. That doesn't affect the file-format
  work below.
- None of this has been tested on real hardware yet. A test run on your Mac is the next step
  if you want this route.

## Geist's own file formats

File extensions registered by the engine: `.geist` (project), `.geistkit`, `.geistpad`,
`.geistengine`, `.geistpattern`, `.geistbank`, `.fxpreset`, `.fxchain`. It also imports
Guru files (`Guru Kit`, `Guru Pattern`, `Guru Pattern Bank`), which it converts on load.

All of them go through `VFoundation::VXMLFile` (TinyXML). There are 223 `SerializeAsXML`
methods, each with its own version constant: `Pad`, `PadState`, `Engine`, `EngineState`,
`Layer*`, `LayerVelocitySplit`, `LayerModulationRouting`, `SlicePoint`, `SliceAnalysis`,
`StepPattern`, `Step`, `InsertablePattern`, `SwingTemplate`, `Song`, `SongSequencer`,
`SceneEngine`, `SceneManager`, `MasterChannel`, `GlobalAux`, `Effect`, `EffectChain`,
`AudioFileManager`, `AudioFileMarkers`, `AudioFileMetaData`, `BindingMap`, `MIDIMapItem`,
`Metronome`, and others.

The factory presets confirm the style:

```xml
<Effect name="Effect" EffectVersion="2" IsActive="true" Mix="1" Level="1"
        Parameter0="0" Parameter1="0.499832" ... EffectType="DJ EQ">
  <FixedString name="PresetName" string="Low Kill" />
</Effect>
```

Samples are referenced by path. The options include "Save relative sample paths",
`SaveSamplesWithChunks` (samples inside the host session chunk), "Save samples with presets"
and "Collect files and export as favourite". This means most user content is XML plus
ordinary audio files, which is the best case for recovering it.

## Options, from cheapest to most work

| # | Option | Effort | Lasting? | Notes |
|---|---|---|---|---|
| A | Run the original x86_64 build (Intel Mac, or Rosetta on Apple Silicon) | Hours to test | Until Rosetta goes | Use the standalone app or VST2 in an Intel host first; the AU may need a registration shim |
| B | **Content converter**: read `.geist`/`.geistkit`/`.geistpattern` XML and export WAV + kit maps (SFZ / Decent Sampler / Ableton Drum Rack) + MIDI patterns | Days to a couple of weeks | Yes | Protects your work whatever happens to A. Needs some of your real files to test against |
| C | Native rebuild (JUCE or similar; arm64 AU/VST3/CLAP; standalone) that loads Geist files | Months | Yes | Sampler, slicer, sequencer and scenes are straightforward. Effects can be close approximations, or reverse-engineered from the disassembly for exact sound. A clean-room reimplementation for interoperability is fine; FXpansion's skins/GUI art should not be copied |
| D | Translate the x86_64 binary to arm64 | Not realistic | | |

**Recommendation:** do B first, since it keeps your kits and patterns usable in any modern
tool. Try A in parallel as a quick test. Decide on C after B, once it's clear how much of
Geist's behaviour you actually miss.

## What I need from you to go further

- A few of your own Geist files (`.geist` projects, `.geistkit` kits, `.geistpattern` files and
  the samples they use). The package has no example kits, so the converter can't be checked
  against real data without them.
- Notes from the *dreist* project, if you want the same approach and structure.
- Whether to run option A as a test on your Mac.
