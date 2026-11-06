# Changelog

## [0.1.1] - 2026-11-06

First release: the original v1 (1.1.1.2) rebuilt natively for Apple Silicon (VST3, CLAP,
standalone) as a thin app on dreist-commons v0.1.1. History was cleaned up before this
release: `main` holds the initial import and one merge per release (`git log --first-parent
main`), the work is on `dev`, and old commit subjects were reworded to Conventional Commits.
GitHub Actions run only on release tags.

### What was ported

- v1 files: projects, kits, pads, engines and patterns convert onto the v2 document on load,
  with v1's object defaults and layer holds; v1's MIDI and host bindings; all of v1's factory
  effect presets, chains, pattern and swing templates load. Einst saves its own `.einst` family.
- Effects: v1's own Gain, Vari-EQ and Envelope Shaper, and v1's differences in the effects it
  shares with v2 (compressors, filters, Flanger, Chorus, Phaser, Delay, Audio Freezer, Ring
  Mod and more), including v1's fast 2^x approximation.
- Voice and sequencer: v1's pitch and filter modulation model, envelope end and loop rules,
  step lanes, pan and send summing, reverse and end fade, timestretch, routing-depth lanes and
  scene snapshots.
- Editor: v1's layout and look, generated from v1's skin layouts; every page, dialog and menu
  checked against the original and its manual (pads and layers, pattern editor with graph
  lanes, scenes, song, mixers, sampler, slicer, browser, MIDI mapping, export, preferences).
- Known differences are logged in the private reverse-engineering ledger; Launchpad support
  is deliberately left out.

### Features

- **einst-formats:** Add einst-formats and the v1 reverse-engineering notes
- **einst-formats:** Convert v1 projects, kits, pads, engines and patterns to the v2 document
- **einst-plugin:** Add the Einst plugin shell, xtask and build script
- **einst-editor:** Add einst-editor, Einst's copy of Dreist's editor
- **einst-dsp:** v1's own Gain, Vari-EQ and Envelope Shaper
- **einst-dsp:** v1's Channel and Bus Compressor, Modulated Filter and Breverb
- **einst:** Install v1's rules; voice filters take v1's fast 2^x
- **einst:** Install v1's envelope end and loop rules
- **einst:** v1's voice model for pitch, pitch envelope and filter modulation
- **formats:** Convert v1 MIDI and host bindings
- **ui:** V1-layout UI set generated from v1's skin layouts
- **ui:** V1 set uses the skin engine's level strips, pan sliders, meters, plots and tables
- **ui:** V1 set matches v1's look: dark chassis and tabs, silver panel, einst. wordmark
- **ui:** V1 controls from measured styles: gradients, bevels, grooves, pointers
- **editor:** Einst's editor runs on the v1 layout (breaking)
- **ui:** V1 look round 3: radial knobs, tab glow, brushed panel, mixer and browser detail
- **editor:** Wire the v1 Scene, Pad/Layers, mixer, Sampler and slicer pages
- **ui:** Content styles for the code-drawn LCD parts, v1 layout fixes
- **ui:** Authored device dialogs, MIDI console and newer effect panels
- **editor:** Code-drawn LCD content reads the theme's content styles
- **editor:** Device dialogs, MIDI console and every effect type's panel
- **editor:** Browser lists v1's content and draws its icons and badges
- **editor:** The pen tool's right click opens the track menu
- **einst-editor:** A layer row's Trigger plays that layer only
- **einst:** V1 step lanes at note start
- **einst:** V1 pan stage and send summing
- **einst:** V1 reverse and end fade
- **einst:** V1 timestretch ratio
- **einst-editor:** V1 Guru imports
- **einst-editor:** V1 load and save menus
- **einst:** Open v1 plugin state
- **einst:** Master display shows the engine's sample memory
- **einst-editor:** v1's browser list pane
- **einst:** V1 routings slot by slot with their routing-depth lanes
- **einst-editor:** Envelope rows edit a v1 layer's routing slots
- **einst-editor:** v1's View menu filters the browser's file types
- **einst-editor:** v1's Pad/Layers selector menus
- **einst-editor:** v1's envelope handle locks and Host parameters panel
- **einst-editor:** v1's pattern track menu
- **einst-editor:** v1's browser show / hide
- **einst-editor:** v1's output locations
- **einst-editor:** v1's sampler preview output menu
- **einst-editor:** v1's browser drop targets
- **einst-editor:** Drag sampler takes and slicer slices onto pads
- **einst-editor:** v1's wave display drag directions and invert options
- **einst-editor:** Effect preset menu as v1 lays it out
- **einst-editor:** v1's mapping column expand / collapse
- **einst-editor:** v1's overwrite question for samples layers use
- **einst-editor:** v1's question for samples changed on disk
- **einst-editor:** v1's pad and engine selector menus
- **einst-editor:** v1's graph lane and pattern key menus
- **einst-editor:** Sampler Export/Send menu as v1 lists it
- **einst-editor:** v1's song part menu tools
- **einst-editor:** v1's swing template menus
- **einst-editor:** Lane Insert button lists v1's shapes
- **einst-editor:** v1's metronome output menu
- **einst-editor:** v1's question for samples removed from disk
- **einst-editor:** Grid menu in v1's order
- **einst-editor:** v1's Reset question
- **einst-editor:** v1's undo stage limits
- **einst-editor:** v1's keyboard shortcuts and standalone menus
- **einst-editor:** Learn mode right clicks as v1
- **einst-editor:** Browser file menu as v1
- **einst-editor:** Knobs without a right-click menu, as v1
- **einst:** v1's default MIDI map
- **einst-editor:** v1's browser file clicks
- **einst-editor:** Alt-click in the browser loads the next free layer
- **einst-editor:** v1's pad clicks per pad mode
- **einst-editor:** v1's All pads / Row links and their keys
- **einst-editor:** v1's step count keys
- **einst-editor:** Multi tool right click as v1
- **einst-editor:** Drag step selections as v1
- **einst-editor:** v1's Play mode and Choke lists
- **einst-editor:** Layer wave audition and drag zoom as v1
- **einst-editor:** Shift-drag knobs finely, as v1
- **einst-editor:** Classify slices and allocate them by type as v1
- **einst-editor:** Slicer wave tooltip and saved slice types as v1
- **einst-editor:** EQ and filter panels draw their response as v1
- **einst-editor:** Compressor meters, tempo drag and editor details as v1
- **einst-editor:** Song editor tools behave as v1
- **einst-editor:** Drag effects along the slot bar to reorder them
- **einst-editor:** Drag dwell on engine selectors and pads as v1
- **einst-editor:** Slicer shows unused slices dimmed and slice types as v1
- **einst:** v1's timestretch amount curve and Find source tempo
- **einst:** v1's scene snapshot of empty patterns
- **einst-editor:** Pattern editor selection, zoom and graph menu as v1
- **einst-editor:** Layer mixer multi-select as v1
- **einst-editor:** Ctrl-click opens context menus as v1
- **einst-editor:** Automation Edit / Live modes as v1
- **einst-editor:** Alt-nudge moves the selected track as v1
- **einst-editor:** Browser Preview play button as v1
- **einst-editor:** Tempo modifier boxes drag as v1
- **einst-editor:** No-slice star on browser sample icons as v1
- **einst-editor:** A folder dropped on the browser opens it
- **einst-plugin:** Standalone MIDI clock sync as v1
- **einst-editor:** MIDI indicator shows a missing clock as v1
- **einst-editor:** Browser footer bindings as v1
- **einst-editor:** v1's multi graph view
- **einst-editor:** Graph menu marks lanes that hold data as v1
- **einst-editor:** Add and remove content packs as v1
- **einst-editor:** Save samples with presets as v1
- **einst-editor:** Remember editor tools and sampler settings as v1
- **einst-editor:** Browser footer and slicer states as v1
- **einst-editor:** Export prompt and browser location as v1
- **einst-editor:** Sampler modes and settings follow the project as v1
- **einst-editor:** Swing template import as v1
- **einst-editor:** Alt on engine Mute / Solo / Pattern power sets all engines
- **einst-editor:** A preset dropped on the window loads, as v1

### Fixes

- **einst-formats:** Recompute layer holds as v1 does
- **einst-dsp:** Stereo Width, Phaser, Frequency Shifter and Noise Gate as v1, checked against v2
- **einst:** About text describes Einst without the original's product name
- **einst-dsp:** Comb Filter takes v1's fast 2^x and summation order
- **einst-dsp:** Ladder, Japan, Little Fatty and Bit Crusher as v1
- **einst-dsp:** Distortion, Noise Gate, EQ, Filter, Delay and Tin Can as v1
- **einst-dsp:** Audio Freezer takes v1's loop, grain cap, jump and trigger
- **einst-dsp:** Ring Mod takes v1's band-limited oscillator and random start phase
- **einst:** The header logo is Einst's amber "E", not Dreist's "D"
- **einst:** About text on three lines instead of clipped
- **einst-editor:** Give each test thread its own settings folder
- **einst-plugin:** Vendor is Einst, not Dreist
- **einst-editor:** Clear pad resets the whole pad, name included
- **einst-editor:** Song lanes, browser column and track toggle as v1 draws them
- **einst-editor:** Pattern grid, LCD knobs, FX slot bar and pad LEDs as v1 draws them
- **einst-dsp:** Flanger, Chorus and Phaser as the original v1 computes them
- **einst-editor:** Divide makes locked slices as v1 does
- **einst-editor:** v1's Recent searches label in the search options
- **einst-editor:** Pad and pattern key drags as v1
- **einst-editor:** Engine selectors take engine files only, as v1
- **einst:** Slices go to v1's 16 pads
- **einst-editor:** Engine selector meters follow v1's level curve
- **einst-editor:** Retro parts go where they were played, as v1
- **einst-editor:** v1's binding types and file drops on 16 pads
- **einst-editor:** Clone using pitching as v1
- **einst-editor:** Pattern editor details checked against v1
- **einst-editor:** Pattern grid cursor per tool as v1
- **einst-editor:** No per-track length markers (v2 only)
- **einst-editor:** More pattern editor details checked against v1
- **einst-editor:** Browser details checked against v1
- **einst-editor:** Effect panel menu and evidence checked against v1
- **einst-editor:** Layer wave wheel and snapping as v1
- **einst-editor:** Wave wheels as v1 in the slicer and sampler
- **einst-editor:** Master strip resets only its gain as v1
- **einst-editor:** Pads checked against v1
- **einst-editor:** Search's audio length offers Is (sec) as v1
- **einst-editor:** Dialogs, preferences and export panel as v1
- **einst-editor:** Slicer session as v1
- **einst-editor:** Knob tooltips from v1's providers
- **einst-editor:** Transport metronome level and tooltips as v1
- **einst-editor:** Pad view Auto and Vel-Amp as v1
- **einst-editor:** Missing files dialog as v1's File Finder
- **einst-editor:** Learnable controls and binding types as v1
- **einst-editor:** Sampler Send to pad keeps the pad's other layers
- **einst-editor:** A pad click selects that pad alone as v1
- **einst-editor:** No pattern or song editing keys, as v1
- **einst-editor:** Sampler source list as v1
- **einst-editor:** Reset keeps locked bindings as v1
- **einst-editor:** Layer sample save dialog as v1
- **einst-editor:** Scene details table columns as v1
- **einst-editor:** A file dropped on the browser list shows its folder, as v1

### Performance

- **einst-editor:** Browser previews decode off the editor thread

### Refactoring

- Make Einst a thin app on dreist-commons
- **einst-dsp:** Use dreist-commons' exp2 hook and drop the copies that only swapped 2^x
- **editor:** Einst owns its editor, einst-editor

### Documentation

- **re:** Document the v1 load path, serializers and effect DSP map
- **re:** Record v1 object defaults and the sampler, binding and MIDI maps
- Move the RE ledger to dreist-re and cite evidence by tag
- **readme:** How the three projects relate
- Contributing section, git conventions for agents
- **changelog:** 0.1.0 is unreleased until the remaining v1 work is done
- Worktree placement and cleanup after a task
- Coordinate shared-code changes between Einst and Dreist
- Read and update the commons coordination log
- **einst-editor:** Pattern grid drawing cites v1 evidence
- **einst-editor:** Browser code cites v1 evidence
- **einst-editor:** Transmod is v2 only
- **einst-editor:** Drag and drop cites v1 evidence
- **einst-editor:** Song editor cites v1 evidence
- **einst-editor:** File finder cites v1 evidence
- **einst-editor:** Guru import cites v1 evidence
- **einst-dsp:** Ladder filters cite v1 evidence
- **einst-editor:** MIDI map and zoom slider cite v1 evidence
- **einst-editor:** Model, project and device panels cite v1 evidence
- **einst-editor:** Scenes, retro, pad menu, meta data cite v1 evidence
- **einst:** Cite v1 evidence where v1 has the same function
- **einst:** Cite v1 evidence for effects, menus and file handling
- **einst-editor:** Keyboard shortcuts cite v1 evidence
- **readme:** Add screenshots of the demo songs

### Tests

- **editor:** Pattern-key drag highlight waits for its first tick on slow machines
- **editor:** M4 dialogs, effect panels, pen menu, slicer bar; goldens
- **einst:** Screen goldens use a scratch settings folder
- **einst-plugin:** Host the CLAP state extension in-process
- Editor host opens several plugin editors in one process
- **einst-editor:** Screen goldens save PNGs with SCREENS_DUMP
- **einst:** Screen goldens for the v1 song lanes and browser column
- **einst:** Screen goldens for the v1 pattern grid, knobs and FX slot bar
- **einst:** Screen goldens for v1's browser list pane
- **einst:** Engine goldens for v1's cubic read order
- **einst:** Per-effect golden renders of v1's 32 effects
- **einst:** Mapping screen golden with the Host parameters panel
- **einst-editor:** Mapping screen golden with the Host parameters panel
- Screen goldens with the browser show / hide button
- Main menu screen goldens with the v1 Grid menu order
- Update screen goldens for filter response plots
- Update engine goldens for v1's timestretch curve
- Update screen goldens for the pattern zoom buttons
- Update screen goldens without track length markers
- **einst:** Modal screen goldens for v1 dialog presentation
- **einst:** Slicer screen golden for v1 slicing locks
- **einst:** File finder screen golden for v1's layout
- **einst:** MIDI settings screen golden for the sync check box
- **einst:** Screen golden for the multi graph view
- Goldens for v1's default tools
- Goldens for v1's footer and slicer defaults
- Golden for the sampler's source box
- Golden for the sampler's default trigger mode
- Golden for the scene details table
- **einst-editor:** Songcheck example

### Build and CI

- Depend on dreist-commons at a pinned revision
- dreist-commons c468034 (the exp2 hook)
- dreist-commons 8a03d51 (porting fixes, voice-filter 2^x, rules)
- dreist-commons 449b54a (envelope/loop rules, voice model hook)
- Take dreist-commons as a submodule at commons, by path
- dreist-commons v0.1.0
- Conventional-commit check, git-cliff changelog, tag-triggered release
- dreist-commons 01309f2
- dreist-commons 2d43602
- dreist-commons 673bb4c (end fade from the triggering sample now the reference; goldens unchanged)
- dreist-commons 5559d30 (v2 end-fade trigger and clamp in the reference; goldens unchanged)
- dreist-commons 230f34e (vendored nih-plug with CLAP state fixes; goldens unchanged)
- dreist-commons fcf90a4 (editor crash fixes, background GUI mode, preview loader; goldens unchanged)
- dreist-commons af84117 (docs only; goldens unchanged)
- Run on main, tags and by hand only; work goes to dev
- dreist-commons 9c356c2 (dev; CI triggers, docs only; goldens unchanged)
- dreist-commons 9584edc (dev; sample memory and its readout text; goldens unchanged)
- dreist-commons 5462f7c (dev; v1 routing-depth lanes and routing slots; goldens unchanged)
- Run workflows only on release tags
- Changelog skips coordination log commits
- dreist-commons v0.1.1
- Build on the macos-15 runner

### Other

- Import the v1 1.1.1.2 factory resources
- **einst:** Screens example renders the editor to PNGs
- **einst-editor:** Slicer helper above the test module
- Bump commons (SampleCache::forget)
- Ignore the local test songs folder
