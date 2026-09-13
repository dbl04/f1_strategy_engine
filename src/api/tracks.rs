//! In-memory GeoJSON track geometry loader and provider endpoints.

use axum::{
    extract::Path,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};

/// Returns embedded GeoJSON circuit geometry for a given track identifier string.
///
/// Embeds circuit vector files from `assets/tracks/` directly into memory at compile time.
pub fn get_track_geometry(track_id: &str) -> Option<&'static str> {
    match track_id.to_lowercase().as_str() {
        "monza" | "it-1922" => Some(include_str!("../../assets/tracks/it-1922.geojson")),
        "silverstone" | "gb-1948" => Some(include_str!("../../assets/tracks/gb-1948.geojson")),
        "spa" | "be-1925" => Some(include_str!("../../assets/tracks/be-1925.geojson")),
        "monaco" | "mc-1929" => Some(include_str!("../../assets/tracks/mc-1929.geojson")),
        "bahrain" | "bh-2002" => Some(include_str!("../../assets/tracks/bh-2002.geojson")),
        "suzuka" | "jp-1962" => Some(include_str!("../../assets/tracks/jp-1962.geojson")),
        "cota" | "us-2012" => Some(include_str!("../../assets/tracks/us-2012.geojson")),
        "singapore" | "sg-2008" => Some(include_str!("../../assets/tracks/sg-2008.geojson")),
        "ae-2009" => Some(include_str!("../../assets/tracks/ae-2009.geojson")),
        "at-1969" => Some(include_str!("../../assets/tracks/at-1969.geojson")),
        "au-1953" => Some(include_str!("../../assets/tracks/au-1953.geojson")),
        "az-2016" => Some(include_str!("../../assets/tracks/az-2016.geojson")),
        "br-1940" => Some(include_str!("../../assets/tracks/br-1940.geojson")),
        "ca-1978" => Some(include_str!("../../assets/tracks/ca-1978.geojson")),
        "cn-2004" => Some(include_str!("../../assets/tracks/cn-2004.geojson")),
        "es-1991" => Some(include_str!("../../assets/tracks/es-1991.geojson")),
        "es-2026" => Some(include_str!("../../assets/tracks/es-2026.geojson")),
        "hu-1986" => Some(include_str!("../../assets/tracks/hu-1986.geojson")),
        "mx-1962" => Some(include_str!("../../assets/tracks/mx-1962.geojson")),
        "nl-1948" => Some(include_str!("../../assets/tracks/nl-1948.geojson")),
        "qa-2004" => Some(include_str!("../../assets/tracks/qa-2004.geojson")),
        "sa-2021" => Some(include_str!("../../assets/tracks/sa-2021.geojson")),
        "us-2022" => Some(include_str!("../../assets/tracks/us-2022.geojson")),
        "us-2023" => Some(include_str!("../../assets/tracks/us-2023.geojson")),
        _ => None,
    }
}

/// Handler serving in-memory GeoJSON vector map string at `GET /api/circuit-geometry/:track_id`.
#[utoipa::path(
    get,
    path = "/api/circuit-geometry/{track_id}",
    params(
        ("track_id" = String, Path, description = "Circuit identifier (e.g. 'monza', 'silverstone', 'spa', 'monaco')")
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
