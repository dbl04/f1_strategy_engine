use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize)]
struct Lap {
    date_start: Option<String>,
    date_end: Option<String>,
    lap_duration: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LocationPoint {
    x: f64,
    y: f64,
    #[serde(default)]
    z: Option<f64>,
    #[serde(default)]
    date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NormalizedPoint {
    x: f64,
    y: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session_key = 9161;
    let driver_number = 63;
    let lap_number = 8;

    println!(
        "Fetching lap window for session {}, driver {}, lap {}...",
        session_key, driver_number, lap_number
    );

    let client = reqwest::blocking::Client::new();
    let lap_url = format!(
        "https://api.openf1.org/v1/laps?session_key={}&driver_number={}&lap_number={}",
        session_key, driver_number, lap_number
    );

    let laps: Vec<Lap> = client.get(&lap_url).send()?.json()?;
    if laps.is_empty() {
        return Err("No lap data found for the given query".into());
    }

    let current_lap = &laps[0];
    let date_start = current_lap
        .date_start
        .as_ref()
        .ok_or("date_start missing from lap data")?;

    let date_end = match &current_lap.date_end {
        Some(end) => end.clone(),
        None => {
            let next_lap_url = format!(
                "https://api.openf1.org/v1/laps?session_key={}&driver_number={}&lap_number={}",
                session_key,
                driver_number,
                lap_number + 1
            );
            let next_laps: Vec<Lap> = client.get(&next_lap_url).send()?.json()?;
            if !next_laps.is_empty() && next_laps[0].date_start.is_some() {
                next_laps[0].date_start.as_ref().unwrap().clone()
            } else if let Some(duration) = current_lap.lap_duration {
                let parsed = chrono::DateTime::parse_from_rfc3339(date_start)?;
                let end_time = parsed + chrono::Duration::milliseconds((duration * 1000.0) as i64);
                end_time.to_rfc3339()
            } else {
                return Err("Unable to determine date_end for lap window".into());
            }
        }
    };

    println!("Lap Window: {} to {}", date_start, date_end);

    // 2. Fetch Cartesian Locations
    let location_url = format!(
        "https://api.openf1.org/v1/location?session_key={}&driver_number={}&date>{date_start}&date<{date_end}",
        session_key, driver_number
    );

    println!("Fetching location data from API...");
    let raw_locations_text = client.get(&location_url).send()?.text()?;
    let raw_locations: Vec<LocationPoint> = serde_json::from_str(&raw_locations_text)?;

    // 4. Data Save (Raw)
    fs::create_dir_all("assets/debug")?;
    fs::write("assets/debug/raw_spike_location.json", &raw_locations_text)?;
    println!("Saved raw location data to assets/debug/raw_spike_location.json");

    let total_raw_points = raw_locations.len();

    // 3. Geometric Normalization & SVG Generation
    let clean_points: Vec<&LocationPoint> = raw_locations
        .iter()
        .filter(|pt| !(pt.x == 0.0 && pt.y == 0.0))
        .collect();

    let total_clean_points = clean_points.len();
    if total_clean_points == 0 {
        return Err("No clean points available after filtering zero coordinates".into());
    }

    let min_x = clean_points
        .iter()
        .map(|pt| pt.x)
        .fold(f64::INFINITY, f64::min);
    let max_x = clean_points
        .iter()
        .map(|pt| pt.x)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_y = clean_points
        .iter()
        .map(|pt| pt.y)
        .fold(f64::INFINITY, f64::min);
    let max_y = clean_points
        .iter()
        .map(|pt| pt.y)
        .fold(f64::NEG_INFINITY, f64::max);

    let width = max_x - min_x;
    let height = max_y - min_y;
    let max_dim = width.max(height);

    let aspect_ratio = if height != 0.0 { width / height } else { 0.0 };

    let normalized_points: Vec<NormalizedPoint> = clean_points
        .iter()
        .map(|pt| NormalizedPoint {
            x: if max_dim != 0.0 {
                (pt.x - min_x) / max_dim
            } else {
                0.0
            },
            y: if max_dim != 0.0 {
                (pt.y - min_y) / max_dim
            } else {
                0.0
            },
        })
        .collect();

    // Data Save (Clean)
    let clean_json = serde_json::to_string_pretty(&normalized_points)?;
    fs::write("assets/debug/clean_spike_location.json", clean_json)?;
    println!("Saved clean location data to assets/debug/clean_spike_location.json");

    // Generate Visual SVG
    let svg_width = 800.0;
    let svg_height = 800.0;
    let padding = 40.0;
    let draw_size = svg_width - 2.0 * padding;

    let norm_w = if max_dim != 0.0 { width / max_dim } else { 1.0 };
    let norm_h = if max_dim != 0.0 {
        height / max_dim
    } else {
        1.0
    };

    let offset_x = padding + (draw_size - norm_w * draw_size) / 2.0;
    let offset_y = padding + (draw_size - norm_h * draw_size) / 2.0;

    let mut path_d = String::new();
    for (i, pt) in normalized_points.iter().enumerate() {
        let svg_x = offset_x + pt.x * draw_size;
        let svg_y = offset_y + (norm_h - pt.y) * draw_size;
        if i == 0 {
            path_d.push_str(&format!("M {:.2},{:.2}", svg_x, svg_y));
        } else {
            path_d.push_str(&format!(" L {:.2},{:.2}", svg_x, svg_y));
        }
    }

    let svg_content = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">
  <rect width="100%" height="100%" fill="#ffffff"/>
  <path d="{path}" fill="none" stroke="#000000" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
</svg>"##,
        w = svg_width,
        h = svg_height,
        path = path_d
    );

    fs::write("assets/debug/spike_track_preview.svg", svg_content)?;
    println!("Saved track preview SVG to assets/debug/spike_track_preview.svg");

    // Console Assertions
    println!("\n================ Track Validation Spike Summary ================");
    println!("Total raw points fetched: {}", total_raw_points);
    println!("Total clean points used:  {}", total_clean_points);
    println!(
        "Bounding Box X:           min = {:.2}, max = {:.2}",
        min_x, max_x
    );
    println!(
        "Bounding Box Y:           min = {:.2}, max = {:.2}",
        min_y, max_y
    );
    println!(
        "Circuit Aspect Ratio:     {:.4} (width / height)",
        aspect_ratio
    );
    println!("=================================================================\n");

    Ok(())
}
