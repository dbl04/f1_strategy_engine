//! OpenF1 HTTP Client targeting `https://api.openf1.org/v1`.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::AppError;

use super::fallback::{
    generate_2026_fallback_drivers, generate_2026_fallback_sessions,
    generate_fallback_driver_locations, generate_fallback_gps_telemetry,
    generate_fallback_intervals, generate_fallback_laps, generate_fallback_positions,
    generate_fallback_race_control, generate_fallback_stints, generate_fallback_weather,
};
use super::models::{
    DriverGpsSample, LiveDriverState, LiveStateResponse, NormalizedTrackPoint, OpenF1Driver,
    OpenF1Interval, OpenF1Lap, OpenF1Location, OpenF1Position, OpenF1RaceControl, OpenF1Session,
    OpenF1Stint, OpenF1Weather,
};
use super::utils::{get_fallback_track_points, parse_iso_to_epoch};

/// OpenF1 HTTP Client targeting `https://api.openf1.org/v1`.
#[derive(Debug, Clone, Default)]
pub struct OpenF1Client;

impl OpenF1Client {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_sessions(&self, year: u32) -> Result<Vec<OpenF1Session>, AppError> {
        let url = format!(
            "https://api.openf1.org/v1/sessions?year={}&session_name=Race",
            year
        );
        let resp = reqwest::get(&url).await;

        let mut sessions: Vec<OpenF1Session> = match resp {
            Ok(res) => res.json::<Vec<OpenF1Session>>().await.unwrap_or_default(),
            Err(_) => vec![],
        };

        if sessions.is_empty() && year == 2026 {
            sessions = generate_2026_fallback_sessions();
        }

        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Filter out cancelled races and races starting in the future (> current datetime)
        sessions.retain(|s| {
            if s.is_cancelled == Some(true) {
                return false;
            }
            if let Some(start_secs) = s.date_start.as_deref().and_then(parse_iso_to_epoch) {
                return start_secs <= now_secs;
            }
            true
        });

        sessions.sort_by(|a, b| {
            let d_a = a.date_start.as_deref().unwrap_or("");
            let d_b = b.date_start.as_deref().unwrap_or("");
            d_a.cmp(d_b).then_with(|| a.session_key.cmp(&b.session_key))
        });

        for s in &mut sessions {
            let start = s.date_start.as_deref().and_then(parse_iso_to_epoch);
            let end = s.date_end.as_deref().and_then(parse_iso_to_epoch);

            let is_live = match (start, end) {
                (Some(st), Some(en)) => now_secs >= st && now_secs <= en,
                _ => false,
            };
            s.is_live = Some(is_live);
        }

        Ok(sessions)
    }

    pub async fn get_drivers(&self, session_key: u64) -> Result<Vec<OpenF1Driver>, AppError> {
        let url = format!(
            "https://api.openf1.org/v1/drivers?session_key={}",
            session_key
        );
        let resp = reqwest::get(&url).await;
        let drivers = match resp {
            Ok(res) => res.json::<Vec<OpenF1Driver>>().await.unwrap_or_default(),
            Err(_) => vec![],
        };
        if drivers.is_empty() {
            return Ok(generate_2026_fallback_drivers(session_key));
        }
        Ok(drivers)
    }

    pub async fn get_laps(
        &self,
        session_key: u64,
        driver_number: u32,
    ) -> Result<Vec<OpenF1Lap>, AppError> {
        let url = format!(
            "https://api.openf1.org/v1/laps?session_key={}&driver_number={}",
            session_key, driver_number
        );
        let resp = reqwest::get(&url)
            .await
            .map_err(|e| AppError::OverpassError(format!("OpenF1 API request failed: {}", e)))?;

        let laps = resp
            .json::<Vec<OpenF1Lap>>()
            .await
            .map_err(|e| AppError::OverpassError(format!("Failed to parse OpenF1 laps: {}", e)))?;

        Ok(laps)
    }

    pub async fn get_all_session_laps(&self, session_key: u64) -> Result<Vec<OpenF1Lap>, AppError> {
        let url = format!("https://api.openf1.org/v1/laps?session_key={}", session_key);
        let resp = reqwest::get(&url).await;
        let laps = match resp {
            Ok(res) => res.json::<Vec<OpenF1Lap>>().await.unwrap_or_default(),
            Err(_) => vec![],
        };
        if laps.is_empty() {
            return Ok(generate_fallback_laps(session_key));
        }
        Ok(laps)
    }

    pub async fn get_locations(
        &self,
        session_key: u64,
        date_gt: Option<&str>,
    ) -> Result<Vec<OpenF1Location>, AppError> {
        let mut url = format!(
            "https://api.openf1.org/v1/location?session_key={}",
            session_key
        );
        if let Some(gt) = date_gt {
            url.push_str(&format!("&date>={}", gt));
        }

        let resp = reqwest::get(&url).await;
        let locations = match resp {
            Ok(res) => res.json::<Vec<OpenF1Location>>().await.unwrap_or_default(),
            Err(_) => vec![],
        };

        if locations.is_empty() {
            return Ok(generate_fallback_driver_locations(session_key));
        }

        Ok(locations)
    }

    pub async fn get_positions(&self, session_key: u64) -> Result<Vec<OpenF1Position>, AppError> {
        let url = format!(
            "https://api.openf1.org/v1/position?session_key={}",
            session_key
        );
        let resp = reqwest::get(&url).await;
        let pos = match resp {
            Ok(res) => res.json::<Vec<OpenF1Position>>().await.unwrap_or_default(),
            Err(_) => vec![],
        };
        if pos.is_empty() {
            return Ok(generate_fallback_positions(session_key));
        }
        Ok(pos)
    }

    pub async fn get_intervals(&self, session_key: u64) -> Result<Vec<OpenF1Interval>, AppError> {
        let url = format!(
            "https://api.openf1.org/v1/intervals?session_key={}",
            session_key
        );
        let resp = reqwest::get(&url).await;
        let res = match resp {
            Ok(r) => r.json::<Vec<OpenF1Interval>>().await.unwrap_or_default(),
            Err(_) => vec![],
        };
        if res.is_empty() {
            return Ok(generate_fallback_intervals(session_key));
        }
        Ok(res)
    }

    pub async fn get_stints(&self, session_key: u64) -> Result<Vec<OpenF1Stint>, AppError> {
        let url = format!(
            "https://api.openf1.org/v1/stints?session_key={}",
            session_key
        );
        let resp = reqwest::get(&url).await;
        let res = match resp {
            Ok(r) => r.json::<Vec<OpenF1Stint>>().await.unwrap_or_default(),
            Err(_) => vec![],
        };
        if res.is_empty() {
            return Ok(generate_fallback_stints(session_key));
        }
        Ok(res)
    }

    pub async fn get_race_control(
        &self,
        session_key: u64,
    ) -> Result<Vec<OpenF1RaceControl>, AppError> {
        let url = format!(
            "https://api.openf1.org/v1/race_control?session_key={}",
            session_key
        );
        let resp = reqwest::get(&url).await;
        let res = match resp {
            Ok(r) => r.json::<Vec<OpenF1RaceControl>>().await.unwrap_or_default(),
            Err(_) => vec![],
        };
        if res.is_empty() {
            return Ok(generate_fallback_race_control(session_key));
        }
        Ok(res)
    }

    pub async fn get_weather(&self, session_key: u64) -> Result<Vec<OpenF1Weather>, AppError> {
        let url = format!(
            "https://api.openf1.org/v1/weather?session_key={}",
            session_key
        );
        let resp = reqwest::get(&url).await;
        let res = match resp {
            Ok(r) => r.json::<Vec<OpenF1Weather>>().await.unwrap_or_default(),
            Err(_) => vec![],
        };
        if res.is_empty() {
            return Ok(generate_fallback_weather(session_key));
        }
        Ok(res)
    }

    pub async fn get_baked_track(
        &self,
        session_key: u64,
    ) -> Result<Vec<NormalizedTrackPoint>, AppError> {
        let mut raw_locations: Vec<OpenF1Location> = Vec::new();

        // Attempt to fetch a reference lap from OpenF1
        for d_no in [1, 3, 4, 16, 44, 63, 81, 55, 10, 14] {
            let laps = self.get_laps(session_key, d_no).await.unwrap_or_default();
            let found_lap = laps.iter().find(|l| {
                l.date_start.is_some() && l.lap_duration.map(|d| d > 40.0).unwrap_or(false)
            });
            let Some(valid_lap) = found_lap else {
                continue;
            };
            let Some(ref start) = valid_lap.date_start else {
                continue;
            };

            let clean_start = if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(start) {
                parsed.format("%Y-%m-%dT%H:%M:%S").to_string()
            } else {
                start.chars().take(19).collect()
            };

            let clean_end = match valid_lap.lap_duration {
                Some(dur) => {
                    if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(start) {
                        let end_time =
                            parsed + chrono::Duration::milliseconds((dur * 1000.0) as i64);
                        Some(end_time.format("%Y-%m-%dT%H:%M:%S").to_string())
                    } else {
                        None
                    }
                }
                None => None,
            };

            let mut url = format!(
                "https://api.openf1.org/v1/location?session_key={}&driver_number={}&date>={}",
                session_key, d_no, clean_start
            );
            if let Some(ref end) = clean_end {
                url.push_str(&format!("&date<={}", end));
            }
            if let Ok(resp) = reqwest::get(&url).await {
                let locs = resp.json::<Vec<OpenF1Location>>().await.unwrap_or_default();
                if locs.len() > 50 {
                    raw_locations = locs;
                    break;
                }
            }
        }

        let clean_locs: Vec<&OpenF1Location> = raw_locations
            .iter()
            .filter(|l| !(l.x == 0.0 && l.y == 0.0))
            .collect();

        if clean_locs.is_empty() {
            return Ok(get_fallback_track_points());
        }

        let min_x = clean_locs.iter().map(|l| l.x).fold(f64::INFINITY, f64::min);
        let max_x = clean_locs
            .iter()
            .map(|l| l.x)
            .fold(f64::NEG_INFINITY, f64::max);
        let min_y = clean_locs.iter().map(|l| l.y).fold(f64::INFINITY, f64::min);
        let max_y = clean_locs
            .iter()
            .map(|l| l.y)
            .fold(f64::NEG_INFINITY, f64::max);

        let width = max_x - min_x;
        let height = max_y - min_y;
        let scale = width.max(height);
        let scale = if scale > 0.0 { scale } else { 1.0 };

        let normalized = clean_locs
            .iter()
            .map(|l| NormalizedTrackPoint {
                x: (l.x - min_x) / scale,
                y: (l.y - min_y) / scale,
            })
            .collect();

        Ok(normalized)
    }

    pub async fn get_session_gps_telemetry(
        &self,
        session_key: u64,
    ) -> Result<Vec<DriverGpsSample>, AppError> {
        let url = format!(
            "https://api.openf1.org/v1/location?session_key={}",
            session_key
        );
        let resp = reqwest::get(&url).await;
        let mut locations = match resp {
            Ok(res) => res.json::<Vec<OpenF1Location>>().await.unwrap_or_default(),
            Err(_) => vec![],
        };

        locations.retain(|l| !(l.x == 0.0 && l.y == 0.0) && l.date.is_some());

        if locations.is_empty() {
            return Ok(generate_fallback_gps_telemetry(session_key));
        }

        // Sort chronologically by date
        locations.sort_by(|a, b| a.date.cmp(&b.date));

        // Find bounding box for normalization
        let min_x = locations.iter().map(|l| l.x).fold(f64::INFINITY, f64::min);
        let max_x = locations
            .iter()
            .map(|l| l.x)
            .fold(f64::NEG_INFINITY, f64::max);
        let min_y = locations.iter().map(|l| l.y).fold(f64::INFINITY, f64::min);
        let max_y = locations
            .iter()
            .map(|l| l.y)
            .fold(f64::NEG_INFINITY, f64::max);

        let width = max_x - min_x;
        let height = max_y - min_y;
        let scale = width.max(height);
        let scale = if scale > 0.0 { scale } else { 1.0 };

        // Parse base timestamp
        let first_date_str = locations[0].date.as_deref().unwrap_or_default();
        let base_timestamp = chrono::DateTime::parse_from_rfc3339(first_date_str)
            .map(|dt| dt.timestamp_millis() as f64 / 1000.0)
            .unwrap_or(0.0);

        // Downsample to ~1.5 Hz per driver (minimum 0.6s between consecutive points)
        let mut last_t_by_driver: std::collections::HashMap<u32, f64> =
            std::collections::HashMap::new();
        let mut downsampled: Vec<DriverGpsSample> = Vec::with_capacity(locations.len() / 2);

        for loc in locations {
            let Some(ref d_str) = loc.date else { continue };
            let t = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(d_str) {
                let sec = (dt.timestamp_millis() as f64 / 1000.0) - base_timestamp;
                if sec < 0.0 { 0.0 } else { sec }
            } else {
                continue;
            };

            let last_t = last_t_by_driver
                .get(&loc.driver_number)
                .copied()
                .unwrap_or(-1.0);
            if t - last_t >= 0.6 {
                last_t_by_driver.insert(loc.driver_number, t);
                downsampled.push(DriverGpsSample {
                    driver_number: loc.driver_number,
                    t: (t * 10.0).round() / 10.0,
                    x: ((loc.x - min_x) / scale * 10000.0).round() / 10000.0,
                    y: ((loc.y - min_y) / scale * 10000.0).round() / 10000.0,
                });
            }
        }

        if downsampled.is_empty() {
            return Ok(generate_fallback_gps_telemetry(session_key));
        }

        Ok(downsampled)
    }

    pub async fn get_live_state(&self, session_key: u64) -> Result<LiveStateResponse, AppError> {
        let positions = self.get_positions(session_key).await.unwrap_or_default();
        let intervals = self.get_intervals(session_key).await.unwrap_or_default();
        let stints = self.get_stints(session_key).await.unwrap_or_default();
        let locations = self
            .get_locations(session_key, None)
            .await
            .unwrap_or_default();
        let race_control = self.get_race_control(session_key).await.unwrap_or_default();
        let weather = self.get_weather(session_key).await.unwrap_or_default();

        let track_status = race_control
            .last()
            .and_then(|rc| rc.flag.clone())
            .unwrap_or_else(|| "GREEN".to_string());

        let (track_temp, air_temp) = weather
            .last()
            .map(|w| (w.track_temperature, w.air_temperature))
            .unwrap_or((38.4, 24.1));

        let clean_locs: Vec<&OpenF1Location> = locations
            .iter()
            .filter(|l| !(l.x == 0.0 && l.y == 0.0))
            .collect();

        let is_already_normalized = !clean_locs.is_empty()
            && clean_locs
                .iter()
                .all(|l| l.x >= 0.0 && l.x <= 1.0 && l.y >= 0.0 && l.y <= 1.0);

        let (min_x, min_y, scale) = if is_already_normalized {
            (0.0, 0.0, 1.0)
        } else if !clean_locs.is_empty() {
            let min_x = clean_locs.iter().map(|l| l.x).fold(f64::INFINITY, f64::min);
            let max_x = clean_locs
                .iter()
                .map(|l| l.x)
                .fold(f64::NEG_INFINITY, f64::max);
            let min_y = clean_locs.iter().map(|l| l.y).fold(f64::INFINITY, f64::min);
            let max_y = clean_locs
                .iter()
                .map(|l| l.y)
                .fold(f64::NEG_INFINITY, f64::max);
            let w = max_x - min_x;
            let h = max_y - min_y;
            let s = w.max(h);
            (min_x, min_y, if s > 0.0 { s } else { 1.0 })
        } else {
            (0.0, 0.0, 1.0)
        };

        let drivers_meta = self.get_drivers(session_key).await.unwrap_or_default();
        let mut drivers = Vec::new();
        for (idx, d_meta) in drivers_meta.into_iter().enumerate() {
            let d_no = d_meta.driver_number;
            let tla = d_meta.name_acronym.clone().unwrap_or_else(|| {
                d_meta
                    .broadcast_name
                    .as_deref()
                    .map(|b| {
                        b.chars()
                            .filter(|c| c.is_alphabetic())
                            .take(3)
                            .collect::<String>()
                            .to_uppercase()
                    })
                    .unwrap_or_else(|| format!("{}", d_no))
            });
            let team = d_meta
                .team_name
                .clone()
                .unwrap_or_else(|| "Formula 1".to_string());

            let pos = positions
                .iter()
                .find(|p| p.driver_number == d_no)
                .map(|p| p.position)
                .unwrap_or((idx + 1) as u32);
            let interval_obj = intervals.iter().find(|i| i.driver_number == d_no);
            let stint_obj = stints.iter().find(|s| s.driver_number == Some(d_no));
            let loc_obj = locations.iter().find(|l| l.driver_number == d_no);

            let gap = if pos == 1 {
                "LEADER".to_string()
            } else {
                interval_obj
                    .and_then(|i| i.gap_to_leader)
                    .map(|g| format!("+{:.3}s", g))
                    .unwrap_or_else(|| format!("+{:.3}s", (pos.saturating_sub(1) as f64) * 1.5))
            };

            let interval = if pos == 1 {
                "-".to_string()
            } else {
                interval_obj
                    .and_then(|i| i.interval)
                    .map(|inv| format!("+{:.3}s", inv))
                    .unwrap_or_else(|| "+0.500s".to_string())
            };

            let (x, y) = match loc_obj {
                Some(l) if !(l.x == 0.0 && l.y == 0.0) => {
                    ((l.x - min_x) / scale, (l.y - min_y) / scale)
                }
                _ => (0.0, 0.0),
            };

            let driver_stints: Vec<&OpenF1Stint> = stints
                .iter()
                .filter(|s| s.driver_number == Some(d_no))
                .collect();
            let pit_stop_count = driver_stints.len().saturating_sub(1) as u32;
            let current_stint = driver_stints.last().copied().or(stint_obj);

            let compound = current_stint
                .and_then(|s| s.compound.clone())
                .unwrap_or_else(|| "MEDIUM".to_string());
            let tyre_age = current_stint.and_then(|s| s.tyre_age_at_start).unwrap_or(0);
            let current_lap = current_stint
                .and_then(|s| s.lap_end.or(s.lap_start))
                .unwrap_or(24);

            let drs_active = interval_obj
                .and_then(|i| i.interval)
                .map(|inv| inv <= 1.0)
                .unwrap_or(false);

            let last_lap_time = Some(((88.5 + (pos as f64 * 0.18)) * 1000.0).round() / 1000.0);

            drivers.push(LiveDriverState {
                driver_number: d_no,
                tla,
                team,
                position: pos,
                gap,
                interval,
                x,
                y,
                compound,
                tyre_age,
                pit_stop_count,
                current_lap,
                last_lap_time,
                drs_active,
            });
        }

        drivers.sort_by_key(|d| d.position);

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let max_lap = drivers.iter().map(|d| d.current_lap).max().unwrap_or(24);
        let odi_res = crate::engine::calculate_dynamic_odi(0.40, max_lap, 53, 1, 0.0);

        Ok(LiveStateResponse {
            session_key,
            timestamp_ms: now_ms,
            track_status,
            track_temperature: track_temp,
            air_temperature: air_temp,
            dynamic_odi: Some(odi_res.dynamic_odi),
            odi_rating: Some(odi_res.difficulty_rating),
            drivers,
        })
    }
}
