//! The CLAP state extension as a host drives it, through the plugin's own
//! `clap_entry` in-process: corrupt state fails the load instead of aborting
//! the host, and the automation parameters survive a save and reload (the
//! host is asked to rescan them).

use std::ffi::{CStr, c_char, c_void};
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicU32, Ordering};

use clap_sys::events::{CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_PARAM_VALUE, clap_event_header, clap_event_param_value, clap_input_events, clap_output_events};
use clap_sys::ext::params::{CLAP_EXT_PARAMS, CLAP_PARAM_RESCAN_VALUES, clap_host_params, clap_param_clear_flags, clap_param_info, clap_param_rescan_flags, clap_plugin_params};
use clap_sys::ext::state::{CLAP_EXT_STATE, clap_plugin_state};
use clap_sys::factory::plugin_factory::{CLAP_PLUGIN_FACTORY_ID, clap_plugin_factory};
use clap_sys::host::clap_host;
use clap_sys::id::clap_id;
use clap_sys::plugin::clap_plugin;
use clap_sys::stream::{clap_istream, clap_ostream};
use clap_sys::version::CLAP_VERSION;
use dreist_host::clap_sys;

unsafe extern "C" fn rescan(host: *const clap_host, flags: clap_param_rescan_flags) {
    if flags & CLAP_PARAM_RESCAN_VALUES != 0 {
        unsafe { (*((*host).host_data as *const AtomicU32)).fetch_add(1, Ordering::SeqCst) };
    }
}
unsafe extern "C" fn clear(_: *const clap_host, _: clap_id, _: clap_param_clear_flags) {}
unsafe extern "C" fn request_flush(_: *const clap_host) {}
static HOST_PARAMS: clap_host_params = clap_host_params { rescan: Some(rescan), clear: Some(clear), request_flush: Some(request_flush) };

unsafe extern "C" fn get_extension(_: *const clap_host, id: *const c_char) -> *const c_void {
    if unsafe { CStr::from_ptr(id) } == CLAP_EXT_PARAMS { &HOST_PARAMS as *const _ as *const c_void } else { null() }
}
unsafe extern "C" fn noop(_: *const clap_host) {}

/// A created and initialised plugin instance with its own host, destroyed
/// on drop.
struct Instance {
    plugin: *const clap_plugin,
    _host: Box<clap_host>,
    /// Value rescans the plugin asked its host for.
    rescans: Box<AtomicU32>,
}

impl Instance {
    fn new() -> Self {
        unsafe {
            let entry = &einst_plugin::clap_entry;
            assert!((entry.init.unwrap())(c"".as_ptr()));
            let factory = (entry.get_factory.unwrap())(CLAP_PLUGIN_FACTORY_ID.as_ptr()) as *const clap_plugin_factory;
            let desc = ((*factory).get_plugin_descriptor.unwrap())(factory, 0);
            let rescans = Box::new(AtomicU32::new(0));
            let host = Box::new(clap_host {
                clap_version: CLAP_VERSION,
                host_data: &*rescans as *const AtomicU32 as *mut c_void,
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
            Instance { plugin, _host: host, rescans }
        }
    }

    fn ext<T>(&self, id: &CStr) -> &T {
        unsafe { &*(((*self.plugin).get_extension.unwrap())(self.plugin, id.as_ptr()) as *const T) }
    }

    fn params(&self) -> Vec<clap_id> {
        let ext: &clap_plugin_params = self.ext(CLAP_EXT_PARAMS);
        let n = unsafe { (ext.count.unwrap())(self.plugin) };
        (0..n)
            .map(|i| unsafe {
                let mut info: clap_param_info = std::mem::zeroed();
                assert!((ext.get_info.unwrap())(self.plugin, i, &mut info));
                info.id
            })
            .collect()
    }

    fn values(&self, ids: &[clap_id]) -> Vec<f64> {
        let ext: &clap_plugin_params = self.ext(CLAP_EXT_PARAMS);
        ids.iter()
            .map(|id| unsafe {
                let mut v = f64::NAN;
                assert!((ext.get_value.unwrap())(self.plugin, *id, &mut v));
                v
            })
            .collect()
    }

    /// Set parameters through `clap_plugin_params::flush` (inactive plugin).
    fn set(&self, values: &[(clap_id, f64)]) {
        let events: Vec<clap_event_param_value> = values
            .iter()
            .map(|&(param_id, value)| clap_event_param_value {
                header: clap_event_header { size: size_of::<clap_event_param_value>() as u32, time: 0, space_id: CLAP_CORE_EVENT_SPACE_ID, type_: CLAP_EVENT_PARAM_VALUE, flags: 0 },
                param_id,
                cookie: null_mut(),
                note_id: -1,
                port_index: -1,
                channel: -1,
                key: -1,
                value,
            })
            .collect();
        unsafe extern "C" fn size(l: *const clap_input_events) -> u32 {
            unsafe { (*((*l).ctx as *const Vec<clap_event_param_value>)).len() as u32 }
        }
        unsafe extern "C" fn get(l: *const clap_input_events, i: u32) -> *const clap_event_header {
            unsafe {
                let events = &*((*l).ctx as *const Vec<clap_event_param_value>);
                &events[i as usize].header
            }
        }
        unsafe extern "C" fn push(_: *const clap_output_events, _: *const clap_event_header) -> bool {
            true
        }
        let input = clap_input_events { ctx: &events as *const _ as *mut c_void, size: Some(size), get: Some(get) };
        let output = clap_output_events { ctx: null_mut(), try_push: Some(push) };
        let ext: &clap_plugin_params = self.ext(CLAP_EXT_PARAMS);
        unsafe { (ext.flush.unwrap())(self.plugin, &input, &output) };
    }

    fn save(&self) -> Vec<u8> {
        unsafe extern "C" fn write(s: *const clap_ostream, buf: *const c_void, n: u64) -> i64 {
            unsafe { (*((*s).ctx as *mut Vec<u8>)).extend_from_slice(std::slice::from_raw_parts(buf as *const u8, n as usize)) };
            n as i64
        }
        let mut out = Vec::new();
        let stream = clap_ostream { ctx: &mut out as *mut _ as *mut c_void, write: Some(write) };
        let ext: &clap_plugin_state = self.ext(CLAP_EXT_STATE);
        assert!(unsafe { (ext.save.unwrap())(self.plugin, &stream) });
        out
    }

    /// Load `data`, delivered at most `chunk` bytes per read.
    fn load(&self, data: &[u8], chunk: usize) -> bool {
        struct Reader<'a>(&'a [u8], usize);
        unsafe extern "C" fn read(s: *const clap_istream, buf: *mut c_void, n: u64) -> i64 {
            unsafe {
                let r = &mut *((*s).ctx as *mut Reader);
                let k = (n as usize).min(r.1).min(r.0.len());
                std::ptr::copy_nonoverlapping(r.0.as_ptr(), buf as *mut u8, k);
                r.0 = &r.0[k..];
                k as i64
            }
        }
        let mut reader = Reader(data, chunk);
        let stream = clap_istream { ctx: &mut reader as *mut _ as *mut c_void, read: Some(read) };
        let ext: &clap_plugin_state = self.ext(CLAP_EXT_STATE);
        unsafe { (ext.load.unwrap())(self.plugin, &stream) }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { ((*self.plugin).destroy.unwrap())(self.plugin) };
    }
}

/// Deterministic noise (xorshift).
fn noise(len: usize, mut seed: u64) -> Vec<u8> {
    (0..len)
        .map(|_| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed as u8
        })
        .collect()
}

#[test]
fn random_state_fails_to_load() {
    let p = Instance::new();
    for seed in 1..4 {
        assert!(!p.load(&noise(1 << 20, seed), usize::MAX), "random bytes (seed {seed}) must not load");
    }
    // A claimed length far past the end of the data.
    let mut huge = u64::MAX.to_le_bytes().to_vec();
    huge.extend_from_slice(br#"{"version":"0","params":{},"fields":{}}"#);
    assert!(!p.load(&huge, usize::MAX));
    // A truncated state.
    let saved = p.save();
    assert!(!p.load(&saved[..saved.len() - 1], usize::MAX));
    assert!(p.load(&saved, usize::MAX));
}

#[test]
fn automation_survives_state_reload() {
    let a = Instance::new();
    let ids = a.params();
    assert_eq!(ids.len(), einst_plugin::AUTOMATION_PARAMS);
    let set: Vec<(clap_id, f64)> = ids.iter().enumerate().map(|(i, &id)| (id, (i as f64 * 0.37 + 0.11).fract())).collect();
    a.set(&set);
    let expected = a.values(&ids);
    assert_ne!(expected, vec![0.0; ids.len()], "the flush set the values");
    let saved = a.save();
    drop(a);

    for chunk in [usize::MAX, 7] {
        let b = Instance::new();
        assert!(b.load(&saved, chunk));
        assert_eq!(b.rescans.load(Ordering::SeqCst), 1, "the host is asked to rescan the values");
        assert_eq!(b.values(&ids), expected);
        assert_eq!(b.save(), saved, "saving again gives the same bytes");
    }
}
