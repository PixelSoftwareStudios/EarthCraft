//! One filesystem worker, with coalesced settings and language writes.
use super::Settings;
use earthcraft_core::language::LanguagePreference;
use std::{
    path::PathBuf,
    sync::{Arc, Condvar, Mutex},
};

#[derive(Default)]
struct Pending {
    settings: Option<Settings>,
    lang: Option<LanguagePreference>,
    kml_paths: Option<Vec<PathBuf>>,
    stopped: bool,
    error: Option<String>,
}
#[derive(Default)]
struct Shared {
    pending: Mutex<Pending>,
    wake: Condvar,
}
pub struct SettingsWriter {
    shared: Arc<Shared>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl SettingsWriter {
    pub fn new(repaint: impl Fn() + Send + 'static) -> std::io::Result<Self> {
        Self::at(super::cache_dir(), repaint)
    }
    fn at(root: PathBuf, repaint: impl Fn() + Send + 'static) -> std::io::Result<Self> {
        let shared = Arc::new(Shared::default());
        let worker = shared.clone();
        let thread = std::thread::Builder::new()
            .name("earthcraft-settings".into())
            .spawn(move || {
                loop {
                    let (settings, lang, kml_paths) = {
                        let mut state = worker.pending.lock().unwrap();
                        while state.settings.is_none()
                            && state.lang.is_none()
                            && state.kml_paths.is_none()
                            && !state.stopped
                        {
                            state = worker.wake.wait(state).unwrap();
                        }
                        if state.settings.is_none()
                            && state.lang.is_none()
                            && state.kml_paths.is_none()
                            && state.stopped
                        {
                            return;
                        }
                        (
                            state.settings.take(),
                            state.lang.take(),
                            state.kml_paths.take(),
                        )
                    };
                    let result = (|| -> Result<(), crate::Error> {
                        std::fs::create_dir_all(&root)?;
                        if let Some(settings) = settings {
                            std::fs::write(
                                root.join("settings.json"),
                                serde_json::to_vec(&settings)?,
                            )?;
                        }
                        if let Some(paths) = kml_paths {
                            std::fs::write(
                                root.join("open-kml.json"),
                                serde_json::to_vec(&paths)?,
                            )?;
                        }
                        if let Some(lang) = lang {
                            std::fs::write(root.join("lang.txt"), lang.code())?;
                        }
                        Ok(())
                    })();
                    if let Err(error) = result {
                        worker.pending.lock().unwrap().error = Some(error.to_string());
                    }
                    repaint();
                }
            })?;
        Ok(Self {
            shared,
            thread: Some(thread),
        })
    }
    pub fn settings(&self, settings: &Settings) {
        self.shared.pending.lock().unwrap().settings = Some(settings.clone());
        self.shared.wake.notify_one();
    }
    /// Persist only source paths; never write the open documents themselves.
    pub fn kml_paths(&self, paths: &[PathBuf]) {
        self.shared.pending.lock().unwrap().kml_paths = Some(paths.to_vec());
        self.shared.wake.notify_one();
    }
    pub fn language(&self, lang: LanguagePreference) {
        self.shared.pending.lock().unwrap().lang = Some(lang);
        self.shared.wake.notify_one();
    }
    pub fn take_error(&self) -> Option<String> {
        self.shared.pending.lock().unwrap().error.take()
    }
    /// Final shutdown only: wait for the latest writes and report failures.
    pub fn finish(&mut self) -> Result<(), crate::Error> {
        self.shared.pending.lock().unwrap().stopped = true;
        self.shared.wake.notify_one();
        if let Some(thread) = self.thread.take() {
            thread
                .join()
                .map_err(|_| crate::Error::from("Settings writer panicked during shutdown"))?;
        }
        match self.take_error() {
            Some(error) => Err(crate::Error::from(error)),
            None => Ok(()),
        }
    }
}
impl Drop for SettingsWriter {
    fn drop(&mut self) {
        // Flush on final application shutdown only; normal UI updates never wait.
        if let Err(error) = self.finish() {
            eprintln!("Settings persistence failed during shutdown: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use earthcraft_core::language::Lang;
    #[test]
    fn language_preferences_preserve_legacy_overrides_and_roundtrip_system_choice() {
        let root = std::env::temp_dir().join(format!("earthcraft-language-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("lang.txt");
        assert_eq!(
            super::super::load_language_preference_at(&path),
            LanguagePreference::System
        );
        for (legacy, expected) in [("en\n", Lang::En), (" pt-BR ", Lang::PtBr)] {
            std::fs::write(&path, legacy).unwrap();
            let preference = super::super::load_language_preference_at(&path);
            assert_eq!(preference, LanguagePreference::Explicit(expected));
            assert_eq!(preference.resolve("en-US"), expected);
            assert_eq!(preference.resolve("pt-BR"), expected);
        }
        for preference in LanguagePreference::ALL {
            let mut writer = SettingsWriter::at(root.clone(), || {}).unwrap();
            writer.language(preference);
            writer.finish().unwrap();
            assert_eq!(std::fs::read_to_string(&path).unwrap(), preference.code());
            assert_eq!(super::super::load_language_preference_at(&path), preference);
        }
        let mut writer = SettingsWriter::at(root.clone(), || {}).unwrap();
        writer.language(LanguagePreference::System);
        writer.finish().unwrap();
        let preference = super::super::load_language_preference_at(&path);
        assert_eq!(preference.resolve("pt-BR"), Lang::PtBr);
        assert_eq!(preference.resolve("en-US"), Lang::En);
        std::fs::write(&path, "unsupported").unwrap();
        assert_eq!(
            super::super::load_language_preference_at(&path),
            LanguagePreference::System
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn explicit_shutdown_reports_final_write_failures() {
        let root = std::env::temp_dir().join(format!(
            "earthcraft-settings-failure-{}",
            std::process::id()
        ));
        std::fs::write(&root, "not a directory").unwrap();
        let mut writer = SettingsWriter::at(root.clone(), || {}).unwrap();
        writer.settings(&Settings::default());
        let result = writer.finish();
        std::fs::remove_file(root).unwrap();
        assert!(result.is_err());
    }
    #[test]
    fn shutdown_flushes_latest_settings_and_language_without_touching_other_files() {
        struct Fixture(PathBuf);
        impl Drop for Fixture {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let fixture = Fixture(
            std::env::temp_dir().join(format!("earthcraft-settings-{}", std::process::id())),
        );
        std::fs::create_dir_all(&fixture.0).unwrap();
        std::fs::write(fixture.0.join("keep.kml"), "keep").unwrap();
        let writer = SettingsWriter::at(fixture.0.clone(), || {}).unwrap();
        for value in 0..100 {
            writer.settings(&Settings {
                kml_dir: value.to_string(),
                ..Settings::default()
            });
            writer.language(LanguagePreference::Explicit(if value % 2 == 0 {
                Lang::En
            } else {
                Lang::PtBr
            }));
            writer.kml_paths(&[fixture.0.join(format!("{value}.kml"))]);
        }
        drop(writer);
        let json: serde_json::Value =
            serde_json::from_slice(&std::fs::read(fixture.0.join("settings.json")).unwrap())
                .unwrap();
        assert_eq!(json["kml_dir"], "99");
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("lang.txt")).unwrap(),
            "pt-BR"
        );
        let session_file = fixture.0.join("open-kml.json");
        let paths: Vec<PathBuf> =
            serde_json::from_slice(&std::fs::read(&session_file).unwrap()).unwrap();
        assert_eq!(paths, vec![fixture.0.join("99.kml")]);
        let writer = SettingsWriter::at(fixture.0.clone(), || {}).unwrap();
        writer.settings(&Settings::default());
        drop(writer);
        let retained: Vec<PathBuf> =
            serde_json::from_slice(&std::fs::read(&session_file).unwrap()).unwrap();
        assert_eq!(paths, retained, "Restore Defaults retains the open sources");
        let writer = SettingsWriter::at(fixture.0.clone(), || {}).unwrap();
        writer.kml_paths(&[]);
        drop(writer);
        assert_eq!(std::fs::read_to_string(session_file).unwrap(), "[]");
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("keep.kml")).unwrap(),
            "keep"
        );
    }
}
