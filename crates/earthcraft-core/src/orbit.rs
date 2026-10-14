//! Approximate two-body orbital subpoints; deliberately not an SGP4 propagator.
const GRAVITY_MU_KM3_PER_SEC2: f64 = 398_600.441_8;
const SECONDS_PER_DAY: f64 = 86400.0;
const MAX_EPOCH_AGE_SECS: f64 = 14.0 * SECONDS_PER_DAY;
const KEPLER_ITERATIONS: usize = 8;
const UNIX_EPOCH_JULIAN_DAY: f64 = 2_440_587.5;
const J2000_JULIAN_DAY: f64 = 2_451_545.0;
const DAYS_PER_JULIAN_CENTURY: f64 = 36_525.0;
const GMST_J2000_DEG: f64 = 280.460_618_37;
const GMST_DEG_PER_DAY: f64 = 360.985_647_366_29;
const GMST_QUADRATIC_DEG: f64 = 0.000_387_933;
const GMST_CUBIC_DIVISOR: f64 = 38_710_000.0;
#[derive(Clone, Debug)]
pub struct OrbitalElements {
    pub inclination_deg: f64,
    pub raan_deg: f64,
    pub eccentricity: f64,
    pub argument_perigee_deg: f64,
    pub mean_anomaly_deg: f64,
    pub mean_motion_rev_per_day: f64,
    pub epoch_unix_secs: f64,
}
impl OrbitalElements {
    /// Approximate subpoint from low-precision two-body propagation (no J2/SGP4).
    pub fn ground_position(&self, unix: f64) -> Option<(f64, f64)> {
        if ![
            unix,
            self.epoch_unix_secs,
            self.inclination_deg,
            self.raan_deg,
            self.eccentricity,
            self.argument_perigee_deg,
            self.mean_anomaly_deg,
            self.mean_motion_rev_per_day,
        ]
        .into_iter()
        .all(f64::is_finite)
        {
            return None;
        }
        if (unix - self.epoch_unix_secs).abs() > MAX_EPOCH_AGE_SECS {
            return None;
        }
        let n = self.mean_motion_rev_per_day * std::f64::consts::TAU / SECONDS_PER_DAY;
        if n <= 0.0 {
            return None;
        }
        let a = (GRAVITY_MU_KM3_PER_SEC2 / (n * n)).cbrt();
        let e = self.eccentricity;
        if !(0.0..1.0).contains(&e) {
            return None;
        }
        let m = (self.mean_anomaly_deg.to_radians() + n * (unix - self.epoch_unix_secs))
            .rem_euclid(std::f64::consts::TAU);
        let mut anomaly = m;
        for _ in 0..KEPLER_ITERATIONS {
            anomaly -= (anomaly - e * anomaly.sin() - m) / (1.0 - e * anomaly.cos());
        }
        let r = a * (1.0 - e * anomaly.cos());
        let true_anomaly = ((1.0 - e * e).sqrt() * anomaly.sin()).atan2(anomaly.cos() - e);
        let u = self.argument_perigee_deg.to_radians() + true_anomaly;
        let (inclination_radians, raan_radians) = (
            self.inclination_deg.to_radians(),
            self.raan_deg.to_radians(),
        );
        let (si, ci) = inclination_radians.sin_cos();
        let (su, cu) = u.sin_cos();
        let (so, co) = raan_radians.sin_cos();
        let x = r * (co * cu - so * su * ci);
        let y = r * (so * cu + co * su * ci);
        let z = r * su * si;
        // Greenwich mean sidereal angle using Julian-date approximation.
        let jd = unix / SECONDS_PER_DAY + UNIX_EPOCH_JULIAN_DAY;
        let t = (jd - J2000_JULIAN_DAY) / DAYS_PER_JULIAN_CENTURY;
        let gmst = (GMST_J2000_DEG
            + GMST_DEG_PER_DAY * (jd - J2000_JULIAN_DAY)
            + GMST_QUADRATIC_DEG * t * t
            - t * t * t / GMST_CUBIC_DIVISOR)
            .to_radians();
        let lat = z.atan2((x * x + y * y).sqrt()).to_degrees();
        let lon = ((y.atan2(x) - gmst).to_degrees() + 180.0).rem_euclid(360.0) - 180.0;
        crate::coordinates::validate_coordinates(lat, lon).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn orbit() -> OrbitalElements {
        OrbitalElements {
            inclination_deg: 90.0,
            raan_deg: 0.0,
            eccentricity: 0.001,
            argument_perigee_deg: 0.0,
            mean_anomaly_deg: 0.0,
            mean_motion_rev_per_day: 15.0,
            epoch_unix_secs: 1_700_000_000.0,
        }
    }
    #[test]
    fn propagation_preserves_equator_polar_motion_and_epoch_window() {
        let orbit = orbit();
        assert!(
            orbit
                .ground_position(orbit.epoch_unix_secs)
                .unwrap()
                .0
                .abs()
                < 0.001
        );
        let quarter_period = SECONDS_PER_DAY / orbit.mean_motion_rev_per_day / 4.0;
        assert!(
            orbit
                .ground_position(orbit.epoch_unix_secs + quarter_period)
                .unwrap()
                .0
                > 89.0
        );
        assert!(
            orbit
                .ground_position(orbit.epoch_unix_secs + MAX_EPOCH_AGE_SECS)
                .is_some()
        );
        assert!(
            orbit
                .ground_position(orbit.epoch_unix_secs + MAX_EPOCH_AGE_SECS + 1.0)
                .is_none()
        );
    }
    #[test]
    fn invalid_elements_and_nonfinite_time_do_not_publish_nan_coordinates() {
        let orbit = orbit();
        assert!(orbit.ground_position(f64::NAN).is_none());
        assert!(
            OrbitalElements {
                inclination_deg: f64::INFINITY,
                ..orbit.clone()
            }
            .ground_position(orbit.epoch_unix_secs)
            .is_none()
        );
        assert!(
            OrbitalElements {
                mean_motion_rev_per_day: 0.0,
                ..orbit.clone()
            }
            .ground_position(orbit.epoch_unix_secs)
            .is_none()
        );
        assert!(
            OrbitalElements {
                eccentricity: 1.0,
                ..orbit.clone()
            }
            .ground_position(orbit.epoch_unix_secs)
            .is_none()
        );
    }
}
