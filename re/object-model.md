# Geist v1 object model, recovered from save/load code (x86_64)

Each section is one save/load function. It lists the XML attribute and child names it uses,
in first-use order (duplicates removed). `<X::Y>` marks a call to another serializer.
`%d` names are numbered children (Pad0..Pad15, Layer0..Layer7 and so on). This is a static reading of the code;
types and default values still need checking against real files.

## `AudioFileManager::SerializeAsXML(VFoundation::VXMLFile&)`

`NumberOfSamples`, `InMemorySamplerFile.sampler`, `AudioFile%dPath`, `XMLFile`, `Ver`, `VXMLFile`

## `AudioFileMarkers::SerializeAsXML(VFoundation::VXMLFile&)`

`FileMarkerVersion`, `StartSample`, `EndSample`, `SampleLength`

## `AudioFileMetaData::SerializeAsXML(VFoundation::VXMLFile&)`

`Ver`, `Name`, `Author`, `Desc`, `BPM`, `Beats`, `Bars`, `Pitch`

## `AudioOutputObject::SerializeOutputState(VFoundation::VXMLFile&)`

`OutputLocation`, `XMLFile`, `VXMLFile`

## `AuxSendingObject::SerializeAuxSends(VFoundation::VXMLFile&)`

`AuxSend%d`, `SerializeAuxSends`

## `Effect::SerializeAsXML(VFoundation::VXMLFile&)`

`EffectVersion`, `IsActive`, `Mix`, `Level`, `PresetName`, `Parameter%d`, `EffectType`

## `EffectChain::SerializeAsXML(VFoundation::VXMLFile&)`

`FXVer`, `Effect%d`, `IsActive`, `ChainPresetName`, `ChainMix`

## `Engine::SerializeAsXML(VFoundation::VXMLFile&)`

`Ver`, `<MixableObject::SerializeMixState>`, `<AudioOutputObject::SerializeOutputState>`, `EffectChain`, `<AuxSendingObject::SerializeAuxSends>`, `SwingTemplate`, `EngineState`, `Pitch`, `TempoNumerator`, `TempoDenominator`, `SwingTemplateDepth`, `StepPattern%d`, `Pad%d`, `EnginePadMode`

## `EngineState::SerializeAsXML(VFoundation::VXMLFile&)`

`Ver`, `PatternPower`, `CurrentPattern`, `CurrentPad`, `CurrentStep`, `FullLevels`, `FullLevelEnabled`, `NoteEraseEnabled`, `NoteRepeatEnabled`, `NoteRepeatSynced`, `PadState%d`, `P%dZR`, `P%dSP`

## `GlobalAux::SerializeAsXML(VFoundation::VXMLFile&)`

`Ver`, `<MixableObject::SerializeMixState>`, `<AudioOutputObject::SerializeOutputState>`, `EffectChain`, `IsPreFade`

## `Grid::LoadEntireState(XMLFile&, bool)`

`StateVersion`, `FileManager`, `MasterChannel`, `Bindings`, `Scenes`, `Song`, `TimeInfo`, `Sampler`, `Metronome`, `CurrentEngine`, `PreviewVolume`, `PreviewOutput`, `SendMIDINotes`, `SendMIDICCs`, `Engine%d`, `GlobalAux%d`

## `Grid::LoadEntireStateFromFile(VFoundation::VString const&, bool)`

`root`, `AllocatorRestoreOutputs`, `LoadEntireStateFromFile`

## `Grid::SaveEntireState(XMLFile&, bool)`

`StateVersion`, `FileManager`, `MasterChannel`, `Bindings`, `Scenes`, `Song`, `TimeInfo`, `Sampler`, `Metronome`, `SendMIDINotes`, `SendMIDICCs`, `CurrentEngine`, `PreviewVolume`, `PreviewOutput`, `Engine%d`, `GlobalAux%d`

## `Grid::SaveEntireStateToFile(VFoundation::VString const&, bool)`

`root`, `SaveSamplesWithChunks`, `SaveEntireStateToFile`

## `Grid::SerializeBindingMap(VFoundation::VString const&, bool)`

`root`, `AllocatorRestoreOutputs`, `SaveSamplesWithChunks`, `<VXMLFile::SerializeObject>`, `SerializeBindingMap`

## `Grid::SerializeCurrentEngine(VFoundation::VString const&)`

`root`, `AllocatorRestoreOutputs`, `SaveSamplesWithChunks`, `<VXMLFile::SerializeObject>`, `SerializeCurrentEngine`

## `Grid::SerializeCurrentKit(VFoundation::VString const&)`

`root`, `AllocatorRestoreOutputs`, `SaveSamplesWithChunks`, `Pad%d`, `SerializeCurrentKit`

## `Grid::SerializeCurrentPad(VFoundation::VString const&)`

`root`, `AllocatorRestoreOutputs`, `SaveSamplesWithChunks`, `<VXMLFile::SerializeObject>`, `SerializeCurrentPad`

## `Grid::SerializeCurrentPattern(VFoundation::VString const&)`

`root`, `AllocatorRestoreOutputs`, `SaveSamplesWithChunks`, `<VXMLFile::SerializeObject>`, `SerializeCurrentPattern`

## `Grid::SerializeCurrentPatternBank(VFoundation::VString const&)`

`root`, `AllocatorRestoreOutputs`, `SaveSamplesWithChunks`, `StepPattern%d`, `<VXMLFile::SerializeObject>`, `SerializeCurrentPatternBank`

## `Grid::SerializeCurrentState(VFoundation::VString const&)`

`SerializeCurrentState`

## `Grid::SerializeEngineFiletoEngine(VFoundation::VString const&, Engine*)`

`root`, `SerializeEngineFiletoEngine`, `AllocatorRestoreOutputs`, `SaveSamplesWithChunks`, `<VXMLFile::SerializeObject>`

## `Grid::SerializeKitFileToEngine(VFoundation::VString const&, Engine*)`

`root`, `AllocatorRestoreOutputs`, `SaveSamplesWithChunks`, `Pad%d`, `SerializeKitFileToEngine`

## `Grid::SerializePadFileToPad(VFoundation::VString const&, Pad*)`

`root`, `SerializePadFileToPad`, `AllocatorRestoreOutputs`, `SaveSamplesWithChunks`, `<VXMLFile::SerializeObject>`

## `Grid::SerializePatternAsMIDI(VFoundation::VString const&, int, int)`

`SerializePatternAsMIDI`

## `InsertablePattern::SerializeAsXML(VFoundation::VXMLFile&)`

`PatternVersion`, `PatternName`, `Element%d`, `Element0`

## `Layer::SerializeAsXML(VFoundation::VXMLFile&)`

`Ver`, `<LockableObject::SerializeLockState>`, `<MixableObject::SerializeMixState>`, `<AudioOutputObject::SerializeOutputState>`, `EffectChain`, `<AuxSendingObject::SerializeAuxSends>`, `VelocitySplit`, `StartEndMarkers`, `LoopMarkers`, `LayerName`, `IsReversed`, `PhasePosition`, `UsesTimestretch`, `TSTracksBPM`, `SourceBPM`, `DestinationBPM`, `V2C`, `V2P`, `EnvTracksPitch`, `Envelope%dParam%d`, `ModRouting%d`, `EnvelopeType%d`, `CutMode`, `PlaybackMode`, `TSMode`, `AudioFileName`, `XMLFile`, `VXMLFile`

## `LayerModulationRouting::SerializeAsXML(VFoundation::VXMLFile&)`

`Ver`, `ModulationDepth`, `ModulationDestination`

## `LayerVelocitySplit::SerializeAsXML(VFoundation::VXMLFile&)`

`Ver`, `LowPoint`, `LowTurn`, `HighTurn`, `Highest`

## `LockableObject::SerializeLockState(VFoundation::VXMLFile&)`

`LockState`

## `MasterChannel::SerializeAsXML(VFoundation::VXMLFile&)`

`Ver`, `<MixableObject::SerializeMixState>`, `<AudioOutputObject::SerializeOutputState>`, `EffectChain`, `Pitch`

## `Metronome::SerializeAsXML(VFoundation::VXMLFile&)`

`Ver`, `<MixableObject::SerializeMixState>`, `<AudioOutputObject::SerializeOutputState>`

## `MixableObject::SerializeMixState(VFoundation::VXMLFile&)`

`Gain`, `Pan`, `Mute`, `Solo`

## `Pad::SerializeAsXML(VFoundation::VXMLFile&)`

`Ver`, `<LockableObject::SerializeLockState>`, `<MixableObject::SerializeMixState>`, `<AudioOutputObject::SerializeOutputState>`, `EffectChain`, `<AuxSendingObject::SerializeAuxSends>`, `V2A`, `PadMode`, `PadName`, `Layer%d`

## `PadState::SerializeAsXML(VFoundation::VXMLFile&)`

`Ver`, `Avis`, `CurrentLayer`, `Atype`, `MTrack%d`

## `Scene::SerializeAsXML(VFoundation::VXMLFile&)`

`used`, `name`, `mode`, `start`, `Eng%d`

## `SceneEngine::SerializeAsXML(VFoundation::VXMLFile&)`

`patt`, `pattpower`

## `SceneManager::SerializeAsXML(VFoundation::VXMLFile&)`

`Scene%d`, `CurrentScene`, `EditorBase`

## `SliceAnalysis::SerializeAsXML(VFoundation::VXMLFile&)`

`Ver`, `Slices`, `ZX`, `Meta`

## `SlicePoint::SerializeAsXML(VFoundation::VXMLFile&)`

`Ver`, `Start`, `End`, `Locked`, `Deleted`, `Pos`, `Strength`, `DynSense`, `Mean`, `Type`

## `Song::SerializeAsXML(VFoundation::VXMLFile&)`

`Track%d`

## `SongPart::SerializeAsXML(VFoundation::VXMLFile&)`

`mute`, `starttime`, `length`, `patt`, `phase`

## `SongSequencer::SerializeAsXML(VFoundation::VXMLFile&)`

`SongActive`, `SongLoopEnabled`, `SongLoopStart`, `SongLoopEnd`, `SongEditorZoom`, `SongEditorStart`, `Song`

## `SongTrack::SerializeAsXML(VFoundation::VXMLFile&)`

`Part%d`

## `Step::SerializeAsXML(VFoundation::VXMLFile&)`

`Ver`, `State`

## `StepPattern::SerializeAsXML(VFoundation::VXMLFile&)`

`Ver`, `<LockableObject::SerializeLockState>`, `PatternName`, `PatternSteps`, `MonoModeActive`, `Track%d`, `SyncMode`, `Resolution`

## `SwingTemplate::SerializeAsXML(VFoundation::VXMLFile&)`

`Ver`, `SwingTemplateName`, `NumberOfSteps`, `S%d`, `Step%d`

## `AutoBorderedPanelStyle::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::CtrlPanelStyle::SerializeAsXML>`, `GradientPeakColour`, `GradientBaseColour`, `UnderlineColour`, `TitleColour`, `LinePos`

## `Binding::SerializeAsXML(VFoundation::VXMLFile&)`

`Min`, `Max`, `MIDI`, `MajorIndex`, `MinorIndex`, `TargetMode`, `EnterTargetMode`, `EditorOnly`, `Path`, `Name`, `MessageStuffer`, `MessagePopper`, `MessagePrototype`, `Type`, `Binding::StuffToValue0`, `Binding::GetValueFromValue0`, `Binding::StuffToValue1`, `Binding::GetValueFromValue1`, `Binding::GetValueFromValue2`, `Binding::StuffToValue2`, `Binding::GetValueFromValue0AsBool`, `Binding::StuffToValue0AsBool`

## `BindingMap::SerializeAsXML(VFoundation::VXMLFile&)`

`VValue`, `HostParameters`, `HostBindingCount%d`, `Auto%dBinding%d`, `MIDI%dCC%dBindingCount`, `MIDI%dCC%dBinding%d`

## `BrowserObject::SerializeAsXML(VFoundation::VXMLFile&)`

`colour`, `path`, `name`

## `BrowserSearchLocations::SerializeAsXML(VFoundation::VXMLFile&)`

`Location`, `Recursive`

## `BrowserSearchObject::SerializeAsXML(VFoundation::VXMLFile&)`

`MatchAllTerms`, `SearchOptions`, `SearchPaths`, `MatchedItems`

## `BrowserSearchOptions::SerializeAsXML(VFoundation::VXMLFile&)`

`SearchFor`, `SubOption`, `SearchType`, `File name`, `Audio length`, `File size`, `File type`

## `CEngine::SerializeAsXML(VFoundation::VXMLFile&)`

`Pad%d`, `Pattern%d`, `Mute`, `Solo`, `Volume`, `Pitch`, `Pan`, `Numerator`, `Denominator`, `CurrentPattern`, `CurrentPad`, `GrooveTime`, `Groove`

## `CGroove::SerializeAsXML(VFoundation::VXMLFile&)`

`Steps`, `Name`, `Step%d`, `Guru ST`

## `CGrooveStep::SerializeAsXML(VFoundation::VXMLFile&)`

`Mov0`

## `CLayer::SerializeAsXML(VFoundation::VXMLFile&)`

`Gain`, `Pan`, `Tune`, `Fine`, `Cutoff`, `Resonance`, `FilterType`, `AmpEnvAttack`, `AmpEnvHold`, `AmpEnvRelease`, `FxEnvAttack`, `FxEnvHold`, `FxEnvRelease`, `FxEnvCutoff`, `FxEnvPitch`, `Lock`, `LayerMute`, `LayerSolo`, `PhaseFlip`, `AuxSend1`, `AuxSend2`, `AuxSend3`, `StartSample`, `EndSample`, `Path`, `Name`

## `CPad::SerializeAsXML(VFoundation::VXMLFile&)`

`Layer%d`, `Mute`, `Solo`, `Level`, `Lock`, `CutBy`, `UsingVelocitySplit`

## `CPattern::SerializeAsXML(VFoundation::VXMLFile&)`

`Resolution`, `Sync`, `Name`, `Pages`, `Steps`, `Track%d`

## `CScene::SerializeAsXML(VFoundation::VXMLFile&)`

`EngPatt%d`

## `CStep::SerializeAsXML(VFoundation::VXMLFile&)`

`State`, `Vol`, `Pan`, `Rpt`, `Shi`, `FFr`, `Res`, `Pit`, `Fin`, `Scr`, `Frc`, `F10`

## `CTrack::SerializeAsXML(VFoundation::VXMLFile&)`

`Steps`, `Step%d`, `CStep`

## `ContentPackBrowserObject::SerializeAsXML(VFoundation::VXMLFile&)`

`<BrowserObject::SerializeAsXML>`, `contentTag`, `name`

## `Cursors::CursorHotspot::SerializeAsXML(VFoundation::VXMLFile&)`

`Hotspot`, `CursorName`

## `FixedString::SerializeAsXML(VFoundation::VXMLFile&)`

`string`

## `MIDIMap::SerializeAsXML(VFoundation::VXMLFile&)`

`Ver`

## `MIDIMapItem::SerializeAsXML(VFoundation::VXMLFile&)`

`Engine`, `Pad`, `Pattern`, `Scene`, `DataPoint%d`

## `Message::SerializeAsXML(VFoundation::VXMLFile&)`

`EngineIndex`, `PadIndex`, `LayerIndex`, `PatternIndex`, `TrackIndex`, `StepIndex`, `Type`, `MessageValue%d`, `VValue`

## `Sampler::SerializeAsXML(VFoundation::VXMLFile&)`

`version`, `mitmsource`, `recordsource`, `inputgain`, `outputgain`, `startthreshold`, `stopthreshold`, `inputthru`, `beatsmode`, `recbars`, `recbeats`, `recsecs`, `startmode`, `timedmode`, `SamplerLastStartMode`, `SamplerLastTimedMode`, `SamplerLastSyncMode`, `%f`, `SamplerLastSeconds`, `%d`, `SamplerLastBars`, `SamplerLastBeats`, `SamplerLastStartThreshold`, `SamplerLastStopThreshold`, `SamplerLastThruMode`

## `ShortlistBrowserObject::SerializeAsXML(VFoundation::VXMLFile&)`

`<BrowserObject::SerializeAsXML>`, `ChildShortlists`

## `SmartFader::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::Ctrl::SerializeAsXML>`

## `SmartKnob::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::CtrlKnob::SerializeAsXML>`

## `TimeInfo::SerializeAsXML(VFoundation::VXMLFile&)`

`Tempo`, `TSN`, `TSD`, `Metro`, `Quantize`

## `Track::SerializeAsXML(VFoundation::VXMLFile&)`

`Ver`, `TrackVersion`, `<LockableObject::SerializeLockState>`, `<MixableObject::SerializeMixState>`, `NumberOfSteps`, `S%d`, `Rand`, `Scal`, `Comp`, `Steps`

## `VApplication::VFont::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`, `bold`, `normal`

## `VApplication::VPreference::SerializeAsXML(VFoundation::VXMLFile&)`

`pref_run_time_only`, `pref_title`, `pref_value`, `pref_visibility`, `pref_user_level`, `pref_type`, `pref_user_data`

## `VApplication::VPreference::VFloatMinMax::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`, `pref_min`, `pref_max`

## `VApplication::VPreference::VLongMinMax::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`, `pref_min`, `pref_max`

## `VCtrl::Ctrl::SerializeAsXML(VFoundation::VXMLFile&, bool, bool)`

`<VObject::SerializeAsXML>`, `overloadClass`, `style`, `rect`, `id`, `displayName`, `displayDesc`, `zorder`, `name`, `Error creating a ctrl of class %s from XML skin definition`, `class`, `CtrlClassView`, `<VRect::SerializeAsXML>`, `Ctrl::SerializeAsXML - aha, an exception`

## `VCtrl::CtrlAutoScrollBox::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::Ctrl::SerializeAsXML>`, `itemList`

## `VCtrl::CtrlAutoScrollBoxItem::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`

## `VCtrl::CtrlButton::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::Ctrl::SerializeAsXML>`

## `VCtrl::CtrlButtonStyle::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::CtrlStyle::SerializeAsXML>`

## `VCtrl::CtrlEvent::SerializeAsXML(VFoundation::VXMLFile&)`

`<VEvent::SerializeAsXML>`, `modifiers`, `point`, `keyCode`, `key`

## `VCtrl::CtrlFaderStyle::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::CtrlSliderStyle::SerializeAsXML>`

## `VCtrl::CtrlImageView::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::Ctrl::SerializeAsXML>`, `image`

## `VCtrl::CtrlImageViewStyle::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::CtrlStyle::SerializeAsXML>`

## `VCtrl::CtrlIndicator::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::Ctrl::SerializeAsXML>`

## `VCtrl::CtrlIndicatorStyle::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::CtrlStyle::SerializeAsXML>`

## `VCtrl::CtrlKnobStyle::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::CtrlStyle::SerializeAsXML>`

## `VCtrl::CtrlLabel::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::Ctrl::SerializeAsXML>`, `text`

## `VCtrl::CtrlLabelStyle::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::CtrlStyle::SerializeAsXML>`

## `VCtrl::CtrlListBoxStyle::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::CtrlStyle::SerializeAsXML>`

## `VCtrl::CtrlMeterStyle::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::CtrlStyle::SerializeAsXML>`

## `VCtrl::CtrlOptionMenuStyle::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::CtrlStyle::SerializeAsXML>`, `ColHighlightText`, `ColDisabledText`, `ColSubMenuArrow`, `ColDivider`, `MenuScrollViewStyle`

## `VCtrl::CtrlPanSliderStyle::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::CtrlFaderStyle::SerializeAsXML>`

## `VCtrl::CtrlPanelStyle::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::CtrlStyle::SerializeAsXML>`

## `VCtrl::CtrlPlot::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::Ctrl::SerializeAsXML>`

## `VCtrl::CtrlPlotStyle::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::CtrlPanelStyle::SerializeAsXML>`

## `VCtrl::CtrlRootVstGui::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::Ctrl::SerializeAsXML>`

## `VCtrl::CtrlScrollViewStyle::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::CtrlPanelStyle::SerializeAsXML>`, `HSliderStyle`, `VSliderStyle`

## `VCtrl::CtrlSliderStyle::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::CtrlStyle::SerializeAsXML>`, `split`, `bottom`, `off`, `top`

## `VCtrl::CtrlStyle::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`

## `VCtrl::CtrlTableViewStyle::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::CtrlStyle::SerializeAsXML>`

## `VCtrl::CtrlTextBoxStyle::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::CtrlStyle::SerializeAsXML>`

## `VCtrl::CtrlTreeView::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::Ctrl::SerializeAsXML>`

## `VCtrl::CtrlTreeViewStyle::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::CtrlStyle::SerializeAsXML>`

## `VCtrl::VProgressBar::SerializeAsXML(VFoundation::VXMLFile&)`

`<VCtrl::Ctrl::SerializeAsXML>`

## `VDSP::VDSPNode::SerializeAsXML(VFoundation::VXMLFile&)`

`<VDSP::VDSPPortArray::SerializeAsXML>`, `display_name`, `display_identifier`, `ID`

## `VDSP::VDSPPort::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`, `name`

## `VDSP::VDSPPortArray::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`, `poly`, `control_in`, `audio_in`, `event_in`, `control_out`, `audio_out`, `event_out`, `VDSPPortControlInProxy`

## `VDSP::VDSPPortControlIn::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`, `name`, `convert`, `umin`, `umax`, `uval`, `enum_val`, `val`, `max`, `min`

## `VDSP::VDSPPortControlInProxy::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`, `name`

## `VDSP::VDSPPortID::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`, `type`, `index`, `id`

## `VFoundation::VArchive::Serialize(VFoundation::IAIFile&)`

`<VObject::Serialize>`, `VArc`, `Info`, `<VObjectArrayTmplSerializeable<VFileInfo>::Serialize>`, `Data`, `<VObjectArrayTmplSerializeable<VData>::Serialize>`

## `VFoundation::VBitString::Serialize_YeOldeBFDStyle(VFoundation::IAIFile&)`

`<VData::Serialize_YeOldeBFDStyle>`

## `VFoundation::VColour::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`, `colour`, `name`

## `VFoundation::VDictionary::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`, `version`, `count`, `class`, `keyArray`, `valueArray`, `<VDictionaryPair::SerializeAsXML>`

## `VFoundation::VDictionaryPair::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`, `class`

## `VFoundation::VEvent::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`, `time`, `evID`

## `VFoundation::VFileInfo::SerializeAsXML(VFoundation::VXMLFile&)`

`isDirectory`, `isWritable`, `isReadable`, `isHidden`, `filename`, `lengthInBytes`, `modificationDate`

## `VFoundation::VFloatVectorMapXY::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`, `count`, `element`, `x`, `y`

## `VFoundation::VGraph::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`, `properties`, `nodes`, `edges`

## `VFoundation::VGraphEdge::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`

## `VFoundation::VGraphNode::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`, `properties`

## `VFoundation::VObjectArray::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`, `count`, `class`

## `VFoundation::VPDFConnection::SerializeAsXML(VFoundation::VXMLFile&)`

`connection_name`, `connection_page`

## `VFoundation::VPoint::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`, `x`, `y`

## `VFoundation::VRange::SerializeAsXML(VFoundation::VXMLFile&)`

`location`, `length`

## `VFoundation::VRect::Serialize(VFoundation::IAIFile&)`

`<VPoint::Serialize>`, `<VSize::Serialize>`

## `VFoundation::VRect::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`, `x`, `y`, `width`, `height`

## `VFoundation::VSize::SerializeAsXML(VFoundation::VXMLFile&)`

`width`, `height`

## `VFoundation::VString::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`, `string`

## `VFoundation::VTimeRange::SerializeAsXML(VFoundation::VXMLFile&)`

`startTime`, `duration`

## `VFoundation::VValue::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`, `type`, `value`

## `VMusic::VGrooveTemplate::SerializeAsXML(VFoundation::VXMLFile&)`

`<VObject::SerializeAsXML>`, `timemap`

## `VMusic::VNoteDurationLimiter::Serialize(VFoundation::IAIFile&)`

`<VMusic::VPatternProcessor::Serialize>`

## `VMusic::VNoteFilter::Serialize(VFoundation::IAIFile&)`

`<VMusic::VPatternProcessor::Serialize>`, `<VBitString::Serialize>`

## `VMusic::VNoteFilter::Serialize_YeOldeBFDStyle(VFoundation::IAIFile&)`

`<VMusic::VPatternProcessor::Serialize>`, `<VBitString::Serialize_YeOldeBFDStyle>`
