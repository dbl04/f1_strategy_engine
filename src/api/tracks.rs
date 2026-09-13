//! In-memory GeoJSON track geometry loader and provider endpoints.

use axum::{
    extract::Path,
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};

/// Returns embedded GeoJSON circuit geometry for a given track identifier or location string.
///
/// Embeds circuit vector files from `assets/tracks/` directly into memory at compile time.
pub fn get_track_geometry(track_id: &str) -> Option<&'static str> {
    let clean = track_id.to_lowercase().replace(['-', '_'], " ");
    let t = clean.trim();
    match t {
        "monza" | "it 1922" | "it1922" | "italy" | "autodromo nazionale monza" => {
            Some(include_str!("../../assets/tracks/it-1922.geojson"))
        }
        "silverstone"
        | "gb 1948"
        | "gb1948"
        | "uk"
        | "britain"
        | "great britain"
        | "silverstone circuit" => Some(include_str!("../../assets/tracks/gb-1948.geojson")),
        "spa"
        | "be 1925"
        | "be1925"
        | "spa francorchamps"
        | "belgium"
        | "circuit de spa francorchamps" => {
            Some(include_str!("../../assets/tracks/be-1925.geojson"))
        }
        "monaco" | "mc 1929" | "mc1929" | "circuit de monaco" | "monte carlo" => {
            Some(include_str!("../../assets/tracks/mc-1929.geojson"))
        }
        "bahrain" | "bh 2002" | "bh2002" | "sakhir" | "bahrain international circuit" => {
            Some(include_str!("../../assets/tracks/bh-2002.geojson"))
        }
        "suzuka" | "jp 1962" | "jp1962" | "japan" | "suzuka international racing course" => {
            Some(include_str!("../../assets/tracks/jp-1962.geojson"))
        }
        "cota" | "us 2012" | "us2012" | "austin" | "circuit of the americas" => {
            Some(include_str!("../../assets/tracks/us-2012.geojson"))
        }
        "singapore" | "sg 2008" | "sg2008" | "marina bay" | "marina bay street circuit" => {
            Some(include_str!("../../assets/tracks/sg-2008.geojson"))
        }
        "melbourne"
        | "au 1953"
        | "au1953"
        | "albert park"
        | "albert park circuit"
        | "australia" => Some(include_str!("../../assets/tracks/au-1953.geojson")),
        "shanghai" | "cn 2004" | "cn2004" | "china" | "shanghai international circuit" => {
            Some(include_str!("../../assets/tracks/cn-2004.geojson"))
        }
        "jeddah" | "sa 2021" | "sa2021" | "saudi arabia" | "jeddah corniche circuit" => {
            Some(include_str!("../../assets/tracks/sa-2021.geojson"))
        }
        "miami" | "us 2022" | "us2022" | "miami international autodrome" => {
            Some(include_str!("../../assets/tracks/us-2022.geojson"))
        }
        "montreal" | "ca 1978" | "ca1978" | "canada" | "circuit gilles villeneuve" => {
            Some(include_str!("../../assets/tracks/ca-1978.geojson"))
        }
        "barcelona"
        | "es 1991"
        | "es1991"
        | "spain"
        | "catalunya"
        | "circuit de barcelona catalunya" => {
            Some(include_str!("../../assets/tracks/es-1991.geojson"))
        }
        "spielberg" | "at 1969" | "at1969" | "austria" | "red bull ring" => {
            Some(include_str!("../../assets/tracks/at-1969.geojson"))
        }
        "budapest" | "hu 1986" | "hu1986" | "hungary" | "hungaroring" => {
            Some(include_str!("../../assets/tracks/hu-1986.geojson"))
        }
        "zandvoort" | "nl 1948" | "nl1948" | "netherlands" | "dutch" | "circuit zandvoort" => {
            Some(include_str!("../../assets/tracks/nl-1948.geojson"))
        }
        "madrid" | "es 2026" | "es2026" | "circuito de madring" => {
            Some(include_str!("../../assets/tracks/es-2026.geojson"))
        }
        "baku" | "az 2016" | "az2016" | "azerbaijan" | "baku city circuit" => {
            Some(include_str!("../../assets/tracks/az-2016.geojson"))
        }
        "mexico" | "mexico city" | "mx 1962" | "mx1962" | "autodromo hermanos rodriguez" => {
            Some(include_str!("../../assets/tracks/mx-1962.geojson"))
        }
        "sao paulo" | "interlagos" | "br 1940" | "br1940" | "brazil" => {
            Some(include_str!("../../assets/tracks/br-1940.geojson"))
        }
        "las vegas" | "vegas" | "us 2023" | "us2023" => {
            Some(include_str!("../../assets/tracks/us-2023.geojson"))
        }
        "lusail" | "losail" | "qa 2004" | "qa2004" | "qatar" => {
            Some(include_str!("../../assets/tracks/qa-2004.geojson"))
        }
        "yas marina" | "abu dhabi" | "ae 2009" | "ae2009" | "uae" => {
            Some(include_str!("../../assets/tracks/ae-2009.geojson"))
        }
        _ => None,
    }
}

/// Handler serving in-memory GeoJSON vector map string at `GET /api/circuit-geometry/:track_id`.
#[utoipa::path(
    get,
    path = "/api/circuit-geometry/{track_id}",
    params(
        ("track_id" = String, Path, description = "Circuit identifier or location name (e.g. 'monza', 'silverstone', 'Melbourne')")
    ),
    responses(
        (status = 200, description = "Raw GeoJSON track circuit vector geometry", body = String),
        (status = 404, description = "Track geometry not found")
    )
)]
pub async fn get_circuit_geometry_by_id(Path(track_id): Path<String>) -> impl IntoResponse {
    if let Some(geojson) = get_track_geometry(&track_id) {
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
        (StatusCode::OK, headers, geojson).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Circuit GeoJSON geometry not found").into_response()
    }
}
