//! The plugin as a host plays it, through its own `clap_entry` in-process:
//! with the host transport playing, a project's patterns sound; a MIDI note
//! on a pad sounds while the transport is stopped.
//!
//! The project comes from `EINST_TEST_PROJECT` (an `.einst` or v1 file with
//! its samples); without one the tests skip.

use std::ffi::{c_char, c_void};
use std::ptr::{null, null_mut};

use clap_sys::audio_buffer::clap_audio_buffer;
use clap_sys::events::{
    CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_MIDI, CLAP_TRANSPORT_HAS_BEATS_TIMELINE, CLAP_TRANSPORT_HAS_TEMPO, CLAP_TRANSPORT_HAS_TIME_SIGNATURE, CLAP_TRANSPORT_IS_PLAYING, clap_event_header,
    clap_event_midi, clap_event_transport, clap_input_events, clap_output_events,
};
use clap_sys::ext::audio_ports::{CLAP_EXT_AUDIO_PORTS, clap_audio_port_info, clap_plugin_audio_ports};
use clap_sys::factory::plugin_factory::{CLAP_PLUGIN_FACTORY_ID, clap_plugin_factory};
use clap_sys::fixedpoint::CLAP_BEATTIME_FACTOR;
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use clap_sys::process::clap_process;
use clap_sys::version::CLAP_VERSION;
use dreist_host::clap_sys;

unsafe extern "C" fn get_extension(_: *const clap_host, _: *const c_char) -> *const c_void {
    null()
}
unsafe extern "C" fn noop(_: *const clap_host) {}

const SR: f64 = 44100.0;
const BLOCK: u32 = 256;

struct Events(Vec<clap_event_midi>);

unsafe extern "C" fn ev_size(list: *const clap_input_events) -> u32 {
    unsafe { (*((*list).ctx as *const Events)).0.len() as u32 }
}
unsafe extern "C" fn ev_get(list: *const clap_input_events, i: u32) -> *const clap_event_header {
    unsafe { &(&(*((*list).ctx as *const Events)).0)[i as usize].header }
}
unsafe extern "C" fn out_push(_: *const clap_output_events, _: *const clap_event_header) -> bool {
    true
}

fn midi(data: [u8; 3]) -> clap_event_midi {
    clap_event_midi { header: clap_event_header { size: size_of::<clap_event_midi>() as u32, time: 0, space_id: CLAP_CORE_EVENT_SPACE_ID, type_: CLAP_EVENT_MIDI, flags: 0 }, port_index: 0, data }
}

struct Instance {
    plugin: *const clap_plugin,
    _host: Box<clap_host>,
    /// Channels per output port.
    ports: Vec<u32>,
    /// Sub outputs passed without buffers, as a VST3 host does for
    /// inactive busses.
    null_aux: bool,
}

impl Instance {
    fn new(null_aux: bool) -> Self {
        unsafe {
            let entry = &einst_plugin::clap_entry;
            assert!((entry.init.unwrap())(c"".as_ptr()));
            let factory = (entry.get_factory.unwrap())(CLAP_PLUGIN_FACTORY_ID.as_ptr()) as *const clap_plugin_factory;
            let desc = ((*factory).get_plugin_descriptor.unwrap())(factory, 0);
            let host = Box::new(clap_host {
                clap_version: CLAP_VERSION,
                host_data: null_mut(),
                name: c"test".as_ptr(),
                vendor: c"test".as_ptr(),
                url: c"".as_ptr(),
                version: c"1".as_ptr(),
                get_extension: Some(get_extension),
                request_restart: Some(noop),
                request_process: Some(noop),
                request_callback: Some(noop),
            });
            let plugin = ((*factory).create_plugin.unwrap())(factory, &*host, (*desc).id);
            assert!(!plugin.is_null() && ((*plugin).init.unwrap())(plugin));
            let ap = ((*plugin).get_extension.unwrap())(plugin, CLAP_EXT_AUDIO_PORTS.as_ptr()) as *const clap_plugin_audio_ports;
            let n = ((*ap).count.unwrap())(plugin, false);
            let ports = (0..n)
                .map(|i| {
                    let mut info: clap_audio_port_info = std::mem::zeroed();
                    assert!(((*ap).get.unwrap())(plugin, i, false, &mut info));
                    info.channel_count
                })
                .collect();
            assert!(((*plugin).activate.unwrap())(plugin, SR, 1, BLOCK));
            assert!(((*plugin).start_processing.unwrap())(plugin));
            Instance { plugin, _host: host, ports, null_aux }
        }
    }

    /// One block; returns the main output's peak.
    fn block(&self, steady: i64, playing: bool, beats: f64, events: Vec<clap_event_midi>) -> f32 {
        let n = BLOCK as usize;
        let mut input = [vec![0f32; n], vec![0f32; n]];
        let mut in_ptrs: Vec<*mut f32> = input.iter_mut().map(|c| c.as_mut_ptr()).collect();
        let in_buf = clap_audio_buffer { data32: in_ptrs.as_mut_ptr(), data64: null_mut(), channel_count: 2, latency: 0, constant_mask: 0 };
        let mut outs: Vec<Vec<Vec<f32>>> = self.ports.iter().map(|&c| vec![vec![0f32; n]; c as usize]).collect();
        let mut out_ptrs: Vec<Vec<*mut f32>> = outs.iter_mut().map(|p| p.iter_mut().map(|c| c.as_mut_ptr()).collect()).collect();
        let mut out_bufs: Vec<clap_audio_buffer> = out_ptrs
            .iter_mut()
            .zip(&self.ports)
            .enumerate()
            .map(|(i, (p, &c))| clap_audio_buffer { data32: if i > 0 && self.null_aux { null_mut() } else { p.as_mut_ptr() }, data64: null_mut(), channel_count: c, latency: 0, constant_mask: 0 })
            .collect();
        let transport = clap_event_transport {
            header: clap_event_header { size: size_of::<clap_event_transport>() as u32, time: 0, space_id: CLAP_CORE_EVENT_SPACE_ID, type_: 0, flags: 0 },
            flags: CLAP_TRANSPORT_HAS_TEMPO | CLAP_TRANSPORT_HAS_BEATS_TIMELINE | CLAP_TRANSPORT_HAS_TIME_SIGNATURE | if playing { CLAP_TRANSPORT_IS_PLAYING } else { 0 },
            song_pos_beats: (beats * CLAP_BEATTIME_FACTOR as f64) as i64,
            song_pos_seconds: 0,
            tempo: 120.0,
            tempo_inc: 0.0,
            loop_start_beats: 0,
            loop_end_beats: 0,
            loop_start_seconds: 0,
            loop_end_seconds: 0,
            bar_start: 0,
            bar_number: 0,
            tsig_num: 4,
            tsig_denom: 4,
        };
        let evs = Events(events);
        let in_events = clap_input_events { ctx: &evs as *const Events as *mut c_void, size: Some(ev_size), get: Some(ev_get) };
        let out_events = clap_output_events { ctx: null_mut(), try_push: Some(out_push) };
        let process = clap_process {
            steady_time: steady,
            frames_count: BLOCK,
            transport: &transport,
            audio_inputs: &in_buf,
            audio_outputs: out_bufs.as_mut_ptr(),
            audio_inputs_count: 1,
            audio_outputs_count: out_bufs.len() as u32,
            in_events: &in_events,
            out_events: &out_events,
        };
        unsafe { ((*self.plugin).process.unwrap())(self.plugin, &process) };
        outs[0].iter().flat_map(|c| c.iter()).fold(0f32, |m, v| m.max(v.abs()))
    }

    /// Runs `seconds` of blocks; returns the main output's peak.
    fn run(&self, seconds: f64, playing: bool, mut events: impl FnMut(usize) -> Vec<clap_event_midi>) -> f32 {
        let blocks = (seconds * SR / BLOCK as f64) as usize;
        let mut peak = 0f32;
        for b in 0..blocks {
            let beats = b as f64 * BLOCK as f64 / SR * 2.0;
            peak = peak.max(self.block((b * BLOCK as usize) as i64, playing, beats, events(b)));
            // The engine is built off the audio thread: give it time.
            if b < 40 {
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
        }
        peak
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            ((*self.plugin).stop_processing.unwrap())(self.plugin);
            ((*self.plugin).deactivate.unwrap())(self.plugin);
            ((*self.plugin).destroy.unwrap())(self.plugin);
        }
    }
}

fn project() -> Option<String> {
    let p = std::env::var("EINST_TEST_PROJECT").ok()?;
    // SAFETY: set before the plugin's threads start; both tests set the same value.
    unsafe { std::env::set_var("EINST_PROJECT", &p) };
    Some(p)
}

fn plays(null_aux: bool) {
    let Some(p) = project() else {
        eprintln!("EINST_TEST_PROJECT not set: skipped");
        return;
    };
    let inst = Instance::new(null_aux);
    let idle = inst.run(1.0, false, |_| Vec::new());
    // Host transport playing: the project's patterns sound.
    let played = inst.run(4.0, true, |_| Vec::new());
    // Transport stopped: a note on every pad note (36..52) sounds.
    let noted = inst.run(2.0, false, |b| if b == 4 { (36u8..52).map(|k| midi([0x90, k, 110])).collect() } else { Vec::new() });
    eprintln!("{p} (null aux {null_aux}): idle {idle:.4} played {played:.4} noted {noted:.4}");
    assert!(played > 0.01, "no output with the host transport playing (peak {played})");
    assert!(noted > 0.01, "no output from MIDI notes (peak {noted})");
}

#[test]
fn host_transport_plays_the_project_and_notes_sound() {
    plays(false);
}

/// Live leaves the sub outputs inactive and passes them without buffers;
/// the main output must still sound (the wrapper used to skip the block).
#[test]
fn inactive_sub_outputs_do_not_silence_the_main_output() {
    plays(true);
}
