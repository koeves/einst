//! What the audio thread and the host feed the editor: meters, playback,
//! load progress, sampler takes, scene / retro / song-recording events, MIDI
//! in, automated parameters and binding messages.

use crate::*;

impl Editor {
    /// One meter tick from the engine's peak levels (the caller clears
    /// them afterwards, as the original clears all meters).
    pub fn update_meters(&mut self, levels: &dreist_engine::Meters) {
        // Only the changed meters are repainted.
        for r in meters::update(&mut self.app.root, &self.model, levels) {
            self.app.invalidate_rect(r);
        }
        self.update_gr_meters(levels);
        if self.header.tick_indicators(levels.midi_received, levels.voice_stolen) {
            let before = self.app.marks();
            self.header.sync(&mut self.app, &self.model, self.learn.active);
            self.app.invalidate_changes(&before);
        }
    }

    /// [re:c5d9d9] / [re:47461f]: a compressor panel's meter (`DasMeter`)
    /// shows the effect's control output ([re:414b35]); while the reading
    /// stays the same the meter rises by 0.01 a tick, up to 1. The last
    /// reading and the shown value live on the meter (`grRaw`).
    fn update_gr_meters(&mut self, levels: &dreist_engine::Meters) {
        let mut dirty = Vec::new();
        for mx in &self.mixers {
            let Some((t, slots)) = mx.fx_panel().and_then(|f| f.shown()) else { continue };
            let at = t.index().map(|v| v as i8);
            for (k, path) in slots.iter().enumerate() {
                let raw = levels.fx_output(at, k);
                let Some(m) = self.app.ctrl(path).and_then(|c| c.path_of("DasMeter")).map(|p| [&path[..], &p[..]].concat()) else { continue };
                let Some(c) = self.app.ctrl_mut(&m) else { continue };
                let last = c.attr("grRaw").and_then(|v| v.parse::<f32>().ok());
                let v = match (raw, last) {
                    (Some(r), l) if l.is_none_or(|l| (r - l).abs() >= 1e-6 || r.is_nan()) => {
                        c.set_attr("grRaw", r.to_string());
                        r
                    }
                    (_, Some(_)) => (c.value + 0.01).min(1.0),
                    _ => continue,
                };
                if (c.value - v).abs() >= 1e-6 {
                    c.value = v;
                    dirty.push(m);
                }
            }
        }
        for m in dirty {
            let r = self.app.abs_rect(&m);
            self.app.invalidate_rect(r);
        }
    }

    /// Transport state from the engine. A play / stop changes the header
    /// (full redraw); a new step or song position only moves the pattern
    /// grid's and song body's playheads, so only those repaint.
    pub fn set_playback(&mut self, playing: bool, steps: [Option<u32>; 8], song_pos: Option<f64>) {
        if self.model.playing != playing {
            self.model.playing = playing;
            self.app.dirty = true;
        }
        let grid = self.model.steps != steps;
        let song = self.model.song_pos != song_pos;
        let (old_pos, new_pos) = (self.model.song_pos, song_pos);
        self.model.steps = steps;
        self.model.song_pos = song_pos;
        if !(grid || song) {
            return;
        }
        // Keep the position in view: the pattern page with option
        // `PatternEditorFollow`, the song page with its Follow button.
        if playing {
            let mut scrolled = grid && self.model.follow_playback && self.pattern.as_ref().is_some_and(|p| p.follow(&self.app, &mut self.model));
            // The song panel only shows the position with the song active.
            let song_active = self.model.doc.children.iter().find(|c| c.class == "SongSequencer").and_then(|n| n.bool("SongActive")).unwrap_or(false);
            if let (true, true, Some(pos), Some(sg)) = (song, song_active, song_pos, self.song.as_mut().filter(|sg| sg.follow_on)) {
                scrolled |= sg.follow(pos);
            }
            if scrolled {
                self.sync();
                self.app.dirty = true;
                return;
            }
        }
        // The song body: the old and new position lines only.
        let lines: Option<Vec<f32>> = self.song.as_ref().and_then(|sg| [old_pos, new_pos].iter().flatten().map(|&p| sg.playhead_x(p)).collect());
        let mut rects = Vec::new();
        fn walk(c: &Ctrl, ox: i32, oy: i32, grid: bool, song: bool, lines: &Option<Vec<f32>>, out: &mut Vec<dreist_gui::Rect>) {
            if !c.visible {
                return;
            }
            let (x, y) = (ox + c.rect.x, oy + c.rect.y);
            if grid && c.overload == "PatternGrid" {
                out.push(dreist_gui::Rect::new(x, y, c.rect.w, c.rect.h));
            }
            if song && c.overload == "SongBody" {
                match lines {
                    Some(ls) => out.extend(ls.iter().map(|lx| dreist_gui::Rect::new(x + lx.floor() as i32 - 2, y, 5, c.rect.h))),
                    None => out.push(dreist_gui::Rect::new(x, y, c.rect.w, c.rect.h)),
                }
            }
            for k in &c.children {
                walk(k, x, y, grid, song, lines, out);
            }
        }
        walk(&self.app.root, -self.app.root.rect.x, -self.app.root.rect.y, grid, song, &lines, &mut rects);
        for r in rects {
            self.app.invalidate_rect(r);
        }
    }

    /// Loader progress of the current rebuild (fraction, 1.5 = done) for
    /// the browser's loading bar ([re:0a1fc4]
    ///  [re:91a0d5] → 0x143).
    pub fn load_progress(&mut self, f: f32) {
        if self.browser.as_mut().is_some_and(|b| b.load_progress(f)) {
            self.app.dirty = true;
        }
    }

    /// The tempo the engine runs at (the host's, tap tempo): into the
    /// document, whose [re:5cc709] is the engine's own state in the original
    /// (the header shows it, saves and rebuilds keep it); not an undo step.
    pub fn engine_tempo(&mut self, bpm: f64) {
        let bpm = bpm.clamp(20.0, 999.0);
        if (bpm - self.model.tempo()).abs() > 1e-9 {
            self.model.doc.child_mut_or_insert("TimeInfo", "TimeInfo").set("Tempo", bpm);
            self.header.sync(&mut self.app, &self.model, self.learn.active);
        }
    }

    /// Whether the host reports a loop (the header's Loop button is
    /// disabled meanwhile).
    pub fn set_host_loop(&mut self, on: bool) {
        if self.model.host_loop != on {
            self.model.host_loop = on;
            self.header.sync(&mut self.app, &self.model, self.learn.active);
            self.app.dirty = true;
        }
    }

    /// Modulators retriggered by pad notes (bit = transmod source index):
    /// flash their source buttons.
    pub fn modulators_triggered(&mut self, mask: u32) {
        for s in 0..32 {
            if mask & (1 << s) != 0 {
                self.transmod.flash(s);
            }
        }
    }

    /// A sampler take finished: it joins the sampler page's take list
    /// and becomes the current take; its index.
    pub fn sampler_take(&mut self, left: &[f32], right: &[f32], sr: f32) -> Option<usize> {
        if left.is_empty() {
            return None;
        }
        // Take stop [re:d17293]: the display shows the finished take.
        let i = self.sampler.add_take(left, right, sr);
        self.model.sampling = None;
        self.sync();
        Some(i)
    }

    /// Live take display from the audio thread ([re:c67e70] /
    /// [re:ba3087]): `reset` starts a new take.
    pub fn sampler_live(&mut self, reset: bool, buckets: &[(f32, f32)]) {
        if !reset && buckets.is_empty() {
            return;
        }
        let e = self.model.engine;
        let sel = (e, self.model.pad[e], self.model.layer[e]);
        if reset || self.model.sampling.is_none() {
            self.model.sampling = Some(model::Sampling { buckets: Vec::new(), frames: 0, selection: sel });
        }
        if let Some(s) = &mut self.model.sampling {
            s.buckets.extend_from_slice(buckets);
            s.frames += buckets.len() * model::LIVE_BUCKET;
        }
        self.app.dirty = true;
    }

    /// Scene notifications from the engine (cued / active slots).
    pub fn scene_events(&mut self, events: &[dreist_engine::scene::SceneEvent]) {
        if events.is_empty() {
            return;
        }
        if let Some(sc) = &mut self.scenes {
            for e in events {
                sc.event(*e);
            }
        }
        self.sync();
    }

    /// Pattern retro input from the engine.
    pub fn retro_events(&mut self, events: &[dreist_engine::songrec::RetroEvent]) {
        for ev in events {
            self.retro.event(ev, self.model.engine);
        }
    }

    /// Steps the engine wrote while recording: copy them into the editor's
    /// document (the original's engine posts the step changes to the editor).
    pub fn step_writes(&mut self, writes: &[dreist_engine::StepWrite]) {
        if writes.is_empty() {
            return;
        }
        for w in writes {
            dreist_engine::engine::doc::write_step(&mut self.model.doc, w.engine, w.pattern, w.track, w.step, &w.data);
        }
        self.sync();
    }

    /// Song recorder messages from the engine.
    pub fn song_events(&mut self, events: &[dreist_engine::songrec::SongEvent]) {
        if events.is_empty() {
            return;
        }
        let limit = self.undo_limit();
        let Some(sg) = &mut self.song else { return };
        let mut out = Vec::new();
        for ev in events {
            if sg.song_event(&self.model, ev, &mut out) {
                // "Record Song Parts" undo point.
                self.history.push("Record Song Parts".into(), self.model.doc.clone(), limit);
            }
            for c in out.drain(..) {
                self.model.apply(&c);
                self.outbox.push(c);
            }
        }
        self.sync();
    }

    /// Incoming MIDI for the MIDI console.
    pub fn midi_in(&mut self, msgs: &[[u8; 3]]) {
        let mode = self.modals.options.int("MIDINumbering", 1).clamp(0, 2) as u8;
        if self.midiconsole.push(msgs, mode) {
            self.app.dirty = true;
        }
    }

    pub fn take_commands(&mut self) -> Vec<Command> {
        if let Some(b) = self.browser.as_mut() {
            if b.poll_preview(&mut self.outbox) {
                self.app.dirty = true;
            }
        }
        if std::mem::take(&mut self.model.pattern_grown) {
            let r = self.model.rebuild_command();
            self.outbox.push(r);
        }
        std::mem::take(&mut self.outbox)
    }

    /// Messages from editor-only bindings (`EditorOnly`; the panels'
    /// message handlers, see `editbind`).
    /// Bound parameters the engine moved (MIDI CC / host automation): into
    /// the document and onto the controls (the original's editor reads the
    /// shared model), not undo steps.
    pub fn automation(&mut self, msgs: &[dreist_engine::binding::Message]) {
        for m in msgs {
            self.model.apply(&Command::Message(m.clone()));
        }
        self.sync();
    }

    pub fn binding_messages(&mut self, msgs: &[dreist_engine::binding::Message]) {
        for m in msgs {
            self.binding_message(m);
        }
        self.sync();
    }

    /// A MIDI CC arrived (count, channel, controller): in learn mode with a
    /// target, bind it.
    pub fn midi_cc(&mut self, count: u32, ch: u8, cc: u8) {
        if count == self.learn.last_cc {
            return;
        }
        self.learn.last_cc = count;
        if let Some(m) = self.midimap.as_mut() {
            m.incoming(&self.app, &self.model, ch as usize, cc as usize);
        }
        if self.learn.active && self.learn.target.is_some() {
            self.learn_bind(true, ch as usize, cc as usize);
            self.learn.target = None;
        }
    }
}
