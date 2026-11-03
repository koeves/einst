//! Preferences (`Options::Read*/Write*`). The original keeps them in the
//! `com.FXpansion.Geist2` defaults domain (shared with licensing); the
//! port stores its own `key=value` file so the installed Geist2's settings
//! are never touched.

use std::collections::BTreeMap;
use std::path::PathBuf;

/// The options file (`key=value` lines). Copies are loaded and saved by
/// many panels; `save` writes only the keys this copy set, over what the
/// file holds then, so an older copy never reverts another's change.
#[derive(Clone, Debug, Default)]
pub struct Options {
    values: BTreeMap<String, String>,
    path: Option<PathBuf>,
    /// Keys set since the load / last save.
    changed: std::collections::BTreeSet<String>,
}

impl PartialEq for Options {
    fn eq(&self, o: &Options) -> bool {
        self.values == o.values && self.path == o.path
    }
}

// Unit tests never touch the user's settings: each test thread has its own
// in-memory options file.
#[cfg(test)]
thread_local! {
    static TEST_FILE: std::cell::RefCell<BTreeMap<String, String>> = Default::default();
}

fn read_file(path: &Option<PathBuf>) -> BTreeMap<String, String> {
    #[cfg(test)]
    {
        let _ = path;
        TEST_FILE.with(|f| f.borrow().clone())
    }
    #[cfg(not(test))]
    {
        // Read once per change of the file (size and modification time):
        // tooltips and panels look options up on every pointer move.
        let Some(p) = path.as_ref() else { return BTreeMap::new() };
        let stamp = std::fs::metadata(p).ok().map(|m| (m.len(), m.modified().ok()));
        let mut cache = FILE_CACHE.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some((cp, cs, values)) = cache.as_ref() {
            if cp == p && *cs == stamp {
                return values.clone();
            }
        }
        let mut values = BTreeMap::new();
        if let Ok(text) = std::fs::read_to_string(p) {
            for line in text.lines() {
                if let Some((k, v)) = line.split_once('=') {
                    values.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }
        *cache = Some((p.clone(), stamp, values.clone()));
        values
    }
}

/// The options file as last read: its path, (size, modification time) and values.
#[cfg(not(test))]
type FileStamp = Option<(u64, Option<std::time::SystemTime>)>;
#[cfg(not(test))]
static FILE_CACHE: std::sync::Mutex<Option<(PathBuf, FileStamp, BTreeMap<String, String>)>> = std::sync::Mutex::new(None);

/// Einst's settings folder (`~/Library/Application Support/Einst`; Dreist's
/// is `Geist-rs`).
/// `Paths::GetUserLocalDocumentsPath` 0x1bdd50: option
/// `LocalDocumentsPath`, default `~/Documents/FXpansion/Geist2`. The user
/// Content, Effects and Effect Chains folders live under it.
pub fn documents_dir() -> PathBuf {
    PathBuf::from(Options::load().string("LocalDocumentsPath", &default_documents()))
}

/// `Paths::GetDefaultUserDocumentsFolder`.
pub fn default_documents() -> String {
    geist_formats::paths::home().join("Documents/FXpansion/Geist2").to_string_lossy().into_owned()
}

/// Engine settings that come from the options (`VoiceManager` reads
/// `ForcedLayersFallThrough` when it is built).
pub fn render_options() -> geist_engine::RenderOptions {
    render_options_from(&Options::load())
}

/// [`render_options`] from loaded options.
pub fn render_options_from(o: &Options) -> geist_engine::RenderOptions {
    geist_engine::RenderOptions {
        forced_layers_fall_through: o.bool("ForcedLayersFallThrough", false),
        auxes_pre_fade: o.bool("AuxesArePreFade", false),
        preroll_mode: crate::header::preroll_index(&o.string("PrerollTime", "PrerollOff")) as u32,
        preroll_only_rec: o.bool("PrerollOnlyOnRecord", true),
        wait_midi: o.bool("WaitForMIDI", false),
        // `kOptionKeyMetroOnWhenRecording` is "PrerollOnlyOnRecord" too.
        metro_only_rec: o.bool("PrerollOnlyOnRecord", false),
        pad_follow: o.bool("PadSelectionFollowsMIDI", true),
        engine_follow: o.bool("EngineSelectionFollowsMIDI", false),
        predelay_ms: o.bool("PreDelayInMsec", true),
        ..Default::default()
    }
}

/// Option `MetronomeSound` (a folder of `Resources/Metronomes`).
pub fn metronome_sound() -> String {
    Options::load().string("MetronomeSound", "Default")
}

pub fn settings_dir() -> Option<PathBuf> {
    // Unit tests get a scratch folder instead of the user's.
    if cfg!(test) {
        return Some(std::env::temp_dir().join(format!("einst-test-{}", std::process::id())));
    }
    // EINST_SETTINGS_DIR: a separate settings folder (tools, fuzzing).
    if let Some(d) = std::env::var_os("EINST_SETTINGS_DIR") {
        return Some(PathBuf::from(d));
    }
    if cfg!(windows) {
        return std::env::var_os("APPDATA").map(|d| PathBuf::from(d).join("Einst"));
    }
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support/Einst"))
}

impl Options {
    pub fn load() -> Options {
        // Unit tests never read or write the user's settings.
        let path = if cfg!(test) { None } else { settings_dir().map(|d| d.join("Options.txt")) };
        Options { values: read_file(&path), path, changed: Default::default() }
    }

    /// Write the keys set since the last save into the current file.
    pub fn save(&mut self) {
        let mut values = read_file(&self.path);
        for k in std::mem::take(&mut self.changed) {
            if let Some(v) = self.values.get(&k) {
                values.insert(k, v.clone());
            }
        }
        // Pick up the others' changes too.
        self.values = values.clone();
        #[cfg(test)]
        TEST_FILE.with(|f| *f.borrow_mut() = values);
        #[cfg(not(test))]
        {
            let Some(p) = &self.path else { return };
            if let Some(d) = p.parent() {
                let _ = std::fs::create_dir_all(d);
            }
            let text: String = values.iter().map(|(k, v)| format!("{k}={v}\n")).collect();
            let _ = geist_formats::file::write_atomic(p, text);
            *FILE_CACHE.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        }
    }

    pub fn bool(&self, key: &str, default: bool) -> bool {
        self.values.get(key).map(|v| v == "true" || v == "1").unwrap_or(default)
    }

    pub fn int(&self, key: &str, default: i64) -> i64 {
        self.values.get(key).and_then(|v| v.parse().ok()).unwrap_or(default)
    }

    pub fn string(&self, key: &str, default: &str) -> String {
        self.values.get(key).cloned().unwrap_or_else(|| default.to_string())
    }

    pub fn set(&mut self, key: &str, value: impl ToString) {
        self.values.insert(key.to_string(), value.to_string());
        self.changed.insert(key.to_string());
    }
}
