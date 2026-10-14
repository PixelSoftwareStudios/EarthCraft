//! CelesTrak OMM element sets converted to approximate *predicted* ground tracks.
//! This is a Kepler two-body educational visualization, not SGP4 or live telemetry.
mod decode;
use decode::parse_omm;
use earthcraft_core::math::now_unix;
use std::{
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};
const URLS: [&str; 3] = [
    "https://celestrak.org/NORAD/elements/gp.php?GROUP=STATIONS&FORMAT=JSON",
    "https://celestrak.org/NORAD/elements/gp.php?GROUP=GPS-OPS&FORMAT=JSON",
    "https://celestrak.org/NORAD/elements/gp.php?GROUP=WEATHER&FORMAT=JSON",
];
const REFRESH: Duration = Duration::from_secs(6 * 3600);
const RETRY: Duration = Duration::from_secs(10 * 60);
#[derive(Clone, Debug)]
pub struct Satellite {
    pub name: String,
    pub norad: String,
    pub orbit: earthcraft_core::orbit::OrbitalElements,
}
impl Satellite {
    pub fn ground_position(&self, unix: f64) -> Option<(f64, f64)> {
        self.orbit.ground_position(unix)
    }
}
#[derive(Default)]
pub struct LiveSatellites {
    satellites: Arc<[Satellite]>,
    loaded: Option<Instant>,
    attempted: Option<Instant>,
    error: Option<crate::Error>,
    rx: Option<mpsc::Receiver<Result<Arc<[Satellite]>, crate::Error>>>,
}
impl LiveSatellites {
    pub fn update(
        &mut self,
        runtime: &crate::runtime::ServiceRuntime,
        enabled: bool,
        repaint: impl Fn() + Send + Sync + 'static,
    ) {
        if let Some(rx) = &self.rx {
            match rx.try_recv() {
                Ok(Ok(s)) => {
                    self.satellites = s;
                    self.loaded = Some(Instant::now());
                    self.error = None;
                    self.rx = None;
                }
                Ok(Err(e)) => {
                    self.error = Some(e);
                    self.rx = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.error = Some("CelesTrak disconnected".into());
                    self.rx = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        if enabled
            && self.rx.is_none()
            && self
                .attempted
                .is_none_or(|t| t.elapsed() >= if self.error.is_some() { RETRY } else { REFRESH })
        {
            let (tx, rx) = mpsc::channel();
            self.rx = Some(rx);
            self.attempted = Some(Instant::now());
            runtime.spawn(async move {
                let _ = tx.send(download().await);
                repaint();
            });
        }
    }
    pub fn snapshot(&self) -> &[Satellite] {
        &self.satellites
    }
    pub fn shared_snapshot(&self) -> Option<Arc<[Satellite]>> {
        (!self.satellites.is_empty()).then(|| self.satellites.clone())
    }
    pub fn status(&self, lang: earthcraft_core::language::Lang) -> String {
        if let Some(error) = &self.error {
            return format!("CelesTrak: {error}");
        }
        if self.loaded.is_none() {
            return lang.text("satellites-loading-status").into();
        }
        lang.format(
            "satellites-loaded-status",
            &[("count", self.satellites.len().into())],
        )
    }
}
async fn download() -> Result<Arc<[Satellite]>, crate::Error> {
    let client = crate::http::client(crate::http::Policy::Aircraft)?;
    let mut all = Vec::new();
    let mut errors = Vec::new();
    for url in URLS {
        match crate::http::body(client.get(url), 4_000_000).await {
            Ok(bytes) => match crate::http::parse_json(bytes).await {
                Ok(v) => all.extend(
                    crate::cpu::run(crate::cpu::Service::Features, move || {
                        Ok(parse_omm(&v, now_unix()))
                    })
                    .await?,
                ),
                Err(e) => errors.push(e.to_string()),
            },
            Err(e) => errors.push(e.to_string()),
        }
    }
    crate::cpu::run(crate::cpu::Service::Features, move || {
        all.sort_unstable_by(|a, b: &Satellite| a.norad.cmp(&b.norad));
        all.dedup_by(|a, b| a.norad == b.norad);
        if all.is_empty() {
            return Err(format!("CelesTrak unavailable: {}", errors.join("; ")).into());
        }
        Ok(all.into())
    })
    .await
}
