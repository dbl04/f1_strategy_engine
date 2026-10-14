//! Synthetic fallback data generators for offline development and testing.
//!
//! When the OpenF1 API is unreachable or returns empty results, these generators
//! produce deterministic 22-car, 53-lap session data matching the 2026 F1 grid.

use std::time::{SystemTime, UNIX_EPOCH};

use super::models::{
    DriverGpsSample, OpenF1Driver, OpenF1Interval, OpenF1Lap, OpenF1Location, OpenF1Position,
    OpenF1RaceControl, OpenF1Session, OpenF1Stint, OpenF1Weather,
};
use super::utils::get_fallback_track_points;

/// Generates the full 2026 F1 driver grid with accurate team assignments and driver numbers.
pub fn generate_2026_fallback_drivers(session_key: u64) -> Vec<OpenF1Driver> {
    let list = [
        (1, "L NORRIS", "Lando NORRIS", "NOR", "McLaren", "F47600"),
        (
            3,
            "M VERSTAPPEN",
            "Max VERSTAPPEN",
            "VER",
            "Red Bull Racing",
            "4781D7",
        ),
        (
            5,
            "G BORTOLETO",
            "Gabriel BORTOLETO",
            "BOR",
            "Audi",
            "F50537",
        ),
        (
            6,
            "I HADJAR",
            "Isack HADJAR",
            "HAD",
            "Red Bull Racing",
            "4781D7",
        ),
        (10, "P GASLY", "Pierre GASLY", "GAS", "Alpine", "00A1E8"),
        (11, "S PEREZ", "Sergio PEREZ", "PER", "Cadillac", "909090"),
        (
            12,
            "K ANTONELLI",
            "Kimi ANTONELLI",
            "ANT",
            "Mercedes",
            "00D7B6",
        ),
        (
            14,
            "F ALONSO",
            "Fernando ALONSO",
            "ALO",
            "Aston Martin",
            "229971",
        ),
        (
            16,
            "C LECLERC",
            "Charles LECLERC",
            "LEC",
            "Ferrari",
            "ED1131",
        ),
        (
            18,
            "L STROLL",
            "Lance STROLL",
            "STR",
            "Aston Martin",
            "229971",
        ),
        (
            23,
            "A ALBON",
            "Alexander ALBON",
            "ALB",
            "Williams",
            "1868DB",
        ),
        (
            27,
            "N HULKENBERG",
            "Nico HULKENBERG",
            "HUL",
            "Audi",
            "F50537",
        ),
        (
            30,
            "L LAWSON",
            "Liam LAWSON",
            "LAW",
            "Racing Bulls",
            "6C98FF",
        ),
        (
            31,
            "E OCON",
            "Esteban OCON",
            "OCO",
            "Haas F1 Team",
            "9C9FA2",
        ),
        (
            41,
            "A LINDBLAD",
            "Arvid LINDBLAD",
            "LIN",
            "Racing Bulls",
            "6C98FF",
        ),
        (
            43,
            "F COLAPINTO",
            "Franco COLAPINTO",
            "COL",
            "Alpine",
            "00A1E8",
        ),
        (
            44,
            "L HAMILTON",
            "Lewis HAMILTON",
            "HAM",
            "Ferrari",
            "ED1131",
        ),
        (55, "C SAINZ", "Carlos SAINZ", "SAI", "Williams", "1868DB"),
        (
            63,
            "G RUSSELL",
            "George RUSSELL",
            "RUS",
            "Mercedes",
            "00D7B6",
        ),
        (
            77,
            "V BOTTAS",
            "Valtteri BOTTAS",
            "BOT",
            "Cadillac",
            "909090",
        ),
        (81, "O PIASTRI", "Oscar PIASTRI", "PIA", "McLaren", "F47600"),
        (
            87,
            "O BEARMAN",
            "Oliver BEARMAN",
            "BEA",
            "Haas F1 Team",
            "9C9FA2",
        ),
    ];

    list.iter()
        .map(|(no, bname, fname, tla, team, col)| OpenF1Driver {
            session_key,
            meeting_key: None,
            driver_number: *no,
            broadcast_name: Some(bname.to_string()),
            full_name: Some(fname.to_string()),
            name_acronym: Some(tla.to_string()),
            team_name: Some(team.to_string()),
            team_colour: Some(col.to_string()),
            first_name: None,
            last_name: None,
            headshot_url: None,
            country_code: None,
        })
        .collect()
}

/// Generates synthetic GPS telemetry samples for all 22 drivers over 120 seconds.
pub fn generate_fallback_gps_telemetry(_session_key: u64) -> Vec<DriverGpsSample> {
    let track_points = get_fallback_track_points();
    if track_points.is_empty() {
        return vec![];
    }
    let total_pts = track_points.len();
    let drivers = [
        1, 3, 5, 6, 10, 11, 12, 14, 16, 18, 23, 27, 30, 31, 41, 43, 44, 55, 63, 77, 81, 87,
    ];

    let mut samples = Vec::new();
    for sec in 0..120 {
        let t = sec as f64;
        for (idx, &d_no) in drivers.iter().enumerate() {
            let offset_fraction = (idx as f64) * 0.015;
            let lap_time = 88.0 + (idx as f64) * 0.35;
            let progress = (((t / lap_time) - offset_fraction) % 1.0 + 1.0) % 1.0;
            let pt_idx = ((progress * (total_pts - 1) as f64).floor() as usize).min(total_pts - 1);
            let pt = &track_points[pt_idx];
            samples.push(DriverGpsSample {
                driver_number: d_no,
                t,
                x: pt.x,
                y: pt.y,
            });
        }
    }
    samples
}

/// Generates synthetic lap telemetry for all 22 drivers across a 53-lap race.
pub fn generate_fallback_laps(_session_key: u64) -> Vec<OpenF1Lap> {
    let drivers = [
        1, 3, 5, 6, 10, 11, 12, 14, 16, 18, 23, 27, 30, 31, 41, 43, 44, 55, 63, 77, 81, 87,
    ];
    let mut laps = Vec::with_capacity(drivers.len() * 53);
    for (idx, &d_no) in drivers.iter().enumerate() {
        let base_lap = 88.0 + (idx as f64) * 0.35;
        let pit_lap = 20 + (idx % 6) as u32;
        for lap_num in 1..=53 {
            let is_pit_in = lap_num == pit_lap;
            let is_pit_out = lap_num == pit_lap + 1;
            let stint = if lap_num <= pit_lap { 1 } else { 2 };
            let compound = if stint == 1 {
                if idx % 2 == 0 { "MEDIUM" } else { "HARD" }
            } else if idx % 2 == 0 {
                "HARD"
            } else {
                "MEDIUM"
            };

            let duration = if is_pit_in {
                base_lap + 22.5
            } else if is_pit_out {
                base_lap + 3.0
            } else {
                base_lap + ((lap_num as f64 * 0.05).sin() * 0.4)
            };

            laps.push(OpenF1Lap {
                lap_number: lap_num,
                driver_number: d_no,
                lap_duration: Some(duration),
                duration_sector_1: Some(duration * 0.33),
                duration_sector_2: Some(duration * 0.35),
                duration_sector_3: Some(duration * 0.32),
                is_pit_out_lap: Some(is_pit_out),
                stint: Some(stint),
                compound: Some(compound.to_string()),
                date_start: Some(format!(
                    "2026-03-15T05:{:02}:{:02}+00:00",
                    (lap_num * 88 / 60) % 60,
                    (lap_num * 88) % 60
                )),
            });
        }
    }
    laps
}

/// Generates synthetic position standings for all 22 drivers.
pub fn generate_fallback_positions(session_key: u64) -> Vec<OpenF1Position> {
    let drivers = [
        1, 3, 5, 6, 10, 11, 12, 14, 16, 18, 23, 27, 30, 31, 41, 43, 44, 55, 63, 77, 81, 87,
    ];
    drivers
        .iter()
        .enumerate()
        .map(|(idx, &no)| OpenF1Position {
            session_key,
            driver_number: no,
            position: (idx + 1) as u32,
            date: Some("2026-03-15T05:00:00+00:00".to_string()),
        })
        .collect()
}

/// Generates synthetic interval/gap data for all 22 drivers.
pub fn generate_fallback_intervals(session_key: u64) -> Vec<OpenF1Interval> {
    let drivers = [
        1, 3, 5, 6, 10, 11, 12, 14, 16, 18, 23, 27, 30, 31, 41, 43, 44, 55, 63, 77, 81, 87,
    ];
    drivers
        .iter()
        .enumerate()
        .map(|(idx, &no)| OpenF1Interval {
            session_key,
            driver_number: no,
            gap_to_leader: Some((idx as f64) * 2.14),
            interval: Some(if idx == 0 { 0.0 } else { 2.14 }),
            date: Some("2026-03-15T05:00:00+00:00".to_string()),
        })
        .collect()
}

/// Generates synthetic stint/compound data for all 22 drivers.
pub fn generate_fallback_stints(session_key: u64) -> Vec<OpenF1Stint> {
    let drivers = [
        1, 3, 5, 6, 10, 11, 12, 14, 16, 18, 23, 27, 30, 31, 41, 43, 44, 55, 63, 77, 81, 87,
    ];
    let mut stints = Vec::new();
    for (idx, &no) in drivers.iter().enumerate() {
        let has_pitted = idx % 2 == 0;
        let pit_lap = 18 + (idx % 6) as u32;

        // Stint 1
        stints.push(OpenF1Stint {
            session_key: Some(session_key),
            driver_number: Some(no),
            stint_number: Some(1),
            compound: Some(if no % 3 == 0 {
                "SOFT".to_string()
            } else {
                "MEDIUM".to_string()
            }),
            tyre_age_at_start: Some(if has_pitted { 0 } else { 24 }),
            lap_start: Some(1),
            lap_end: Some(if has_pitted { pit_lap } else { 53 }),
        });

        if has_pitted {
            // Stint 2 (currently active stint)
            stints.push(OpenF1Stint {
                session_key: Some(session_key),
                driver_number: Some(no),
                stint_number: Some(2),
                compound: Some("HARD".to_string()),
                tyre_age_at_start: Some(28 - pit_lap), // e.g. 5-10 laps old
                lap_start: Some(pit_lap + 1),
                lap_end: Some(53),
            });
        }
    }
    stints
}

/// Generates synthetic driver location data using fallback track points.
pub fn generate_fallback_driver_locations(session_key: u64) -> Vec<OpenF1Location> {
    let drivers = [
        1, 3, 5, 6, 10, 11, 12, 14, 16, 18, 23, 27, 30, 31, 41, 43, 44, 55, 63, 77, 81, 87,
    ];
    let now_millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0);

    let track_points = get_fallback_track_points();
    if track_points.is_empty() {
        return vec![];
    }
    let total_pts = track_points.len();

    drivers
        .iter()
        .enumerate()
        .map(|(idx, &driver_number)| {
            let spacing = total_pts as f64 / drivers.len() as f64;
            let progress_offset = (now_millis * 0.005) as usize;
            let pt_idx = ((idx as f64 * spacing) as usize + progress_offset) % total_pts;
            let pt = &track_points[pt_idx];

            OpenF1Location {
                session_key,
                driver_number,
                x: pt.x,
                y: pt.y,
                z: Some(10.0),
                date: Some("2026-03-15T05:15:00+00:00".to_string()),
            }
        })
        .collect()
}

/// Generates synthetic 2026 season calendar with all 24 Grand Prix sessions.
pub fn generate_2026_fallback_sessions() -> Vec<OpenF1Session> {
    let locations = [
        (
            "Melbourne",
            "Albert Park Circuit",
            "au-1953",
            "2026-03-15T05:00:00+00:00",
            "2026-03-15T07:00:00+00:00",
        ),
        (
            "Shanghai",
            "Shanghai International Circuit",
            "cn-2004",
            "2026-03-22T07:00:00+00:00",
            "2026-03-22T09:00:00+00:00",
        ),
        (
            "Suzuka",
            "Suzuka International Racing Course",
            "jp-1962",
            "2026-04-05T05:00:00+00:00",
            "2026-04-05T07:00:00+00:00",
        ),
        (
            "Sakhir",
            "Bahrain International Circuit",
            "bh-2002",
            "2026-04-19T15:00:00+00:00",
            "2026-04-19T17:00:00+00:00",
        ),
        (
            "Jeddah",
            "Jeddah Corniche Circuit",
            "sa-2021",
            "2026-04-26T17:00:00+00:00",
            "2026-04-26T19:00:00+00:00",
        ),
        (
            "Miami",
            "Miami International Autodrome",
            "us-2022",
            "2026-05-03T20:00:00+00:00",
            "2026-05-03T22:00:00+00:00",
        ),
        (
            "Montreal",
            "Circuit Gilles-Villeneuve",
            "ca-1978",
            "2026-05-17T18:00:00+00:00",
            "2026-05-17T20:00:00+00:00",
        ),
        (
            "Monaco",
            "Circuit de Monaco",
            "mc-1929",
            "2026-05-24T13:00:00+00:00",
            "2026-05-24T15:00:00+00:00",
        ),
        (
            "Barcelona",
            "Circuit de Barcelona-Catalunya",
            "es-1991",
            "2026-06-07T13:00:00+00:00",
            "2026-06-07T15:00:00+00:00",
        ),
        (
            "Spielberg",
            "Red Bull Ring",
            "at-1969",
            "2026-06-28T13:00:00+00:00",
            "2026-06-28T15:00:00+00:00",
        ),
        (
            "Silverstone",
            "Silverstone Circuit",
            "gb-1948",
            "2026-07-05T14:00:00+00:00",
            "2026-07-05T16:00:00+00:00",
        ),
        (
            "Spa Francorchamps",
            "Circuit de Spa-Francorchamps",
            "be-1925",
            "2026-07-26T13:00:00+00:00",
            "2026-07-26T15:00:00+00:00",
        ),
        (
            "Budapest",
            "Hungaroring",
            "hu-1986",
            "2026-08-02T13:00:00+00:00",
            "2026-08-02T15:00:00+00:00",
        ),
        (
            "Zandvoort",
            "Circuit Zandvoort",
            "nl-1948",
            "2026-08-30T13:00:00+00:00",
            "2026-08-30T15:00:00+00:00",
        ),
        (
            "Monza",
            "Autodromo Nazionale Monza",
            "it-1922",
            "2026-09-06T13:00:00+00:00",
            "2026-09-06T15:00:00+00:00",
        ),
        (
            "Madrid",
            "Circuito de Madring",
            "es-2026",
            "2026-09-20T13:00:00+00:00",
            "2026-09-20T15:00:00+00:00",
        ),
        (
            "Baku",
            "Baku City Circuit",
            "az-2016",
            "2026-10-04T11:00:00+00:00",
            "2026-10-04T13:00:00+00:00",
        ),
        (
            "Singapore",
            "Marina Bay Street Circuit",
            "sg-2008",
            "2026-10-11T12:00:00+00:00",
            "2026-10-11T14:00:00+00:00",
        ),
        (
            "Austin",
            "Circuit of the Americas",
            "us-2012",
            "2026-10-25T19:00:00+00:00",
            "2026-10-25T21:00:00+00:00",
        ),
        (
            "Mexico City",
            "Autódromo Hermanos Rodríguez",
            "mx-1962",
            "2026-11-01T20:00:00+00:00",
            "2026-11-01T22:00:00+00:00",
        ),
        (
            "Sao Paulo",
            "Autódromo José Carlos Pace - Interlagos",
            "br-1940",
            "2026-11-15T17:00:00+00:00",
            "2026-11-15T19:00:00+00:00",
        ),
        (
            "Las Vegas",
            "Las Vegas Street Circuit",
            "us-2023",
            "2026-11-22T06:00:00+00:00",
            "2026-11-22T08:00:00+00:00",
        ),
        (
            "Lusail",
            "Losail International Circuit",
            "qa-2004",
            "2026-11-29T17:00:00+00:00",
            "2026-11-29T19:00:00+00:00",
        ),
        (
            "Yas Marina",
            "Yas Marina Circuit",
            "ae-2009",
            "2026-12-06T13:00:00+00:00",
            "2026-12-06T15:00:00+00:00",
        ),
    ];

    let mut sessions = Vec::new();
    for (idx, (loc, _name, _id, start, end)) in locations.iter().enumerate() {
        let base_key = 9000 + (idx as u64 * 10);
        let session_types = ["Practice 1", "Practice 2", "Qualifying", "Race"];
        for (s_idx, s_name) in session_types.iter().enumerate() {
            sessions.push(OpenF1Session {
                session_key: base_key + s_idx as u64,
                session_name: Some(s_name.to_string()),
                session_type: Some(s_name.to_string()),
                date_start: Some(start.to_string()),
                date_end: Some(end.to_string()),
                gmt_offset: Some("+00:00".to_string()),
                circuit_short_name: Some(loc.to_string()),
                circuit_key: Some(100 + idx as u64),
                location: Some(loc.to_string()),
                country_key: Some(1 + idx as u64),
                country_name: Some(loc.to_string()),
                year: Some(2026),
                is_live: Some(false),
                is_cancelled: Some(false),
            });
        }
    }

    sessions
}

/// Generates a single fallback race control event (GREEN flag).
pub fn generate_fallback_race_control(session_key: u64) -> Vec<OpenF1RaceControl> {
    vec![OpenF1RaceControl {
        session_key,
        category: "Flag".to_string(),
        flag: Some("GREEN".to_string()),
        message: Some("TRACK CLEAR".to_string()),
        scope: Some("Track".to_string()),
        date: Some("2026-03-15T05:00:00+00:00".to_string()),
    }]
}

/// Generates a single fallback weather reading.
pub fn generate_fallback_weather(session_key: u64) -> Vec<OpenF1Weather> {
    vec![OpenF1Weather {
        session_key,
        air_temperature: 24.1,
        track_temperature: 38.4,
        humidity: 45.0,
        pressure: 1013.2,
        rainfall: 0,
        date: Some("2026-03-15T05:00:00+00:00".to_string()),
    }]
}
