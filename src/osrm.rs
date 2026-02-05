//! OSRM HTTP adapter for distance matrices and route geometry.

use serde::Deserialize;

use crate::traits::DistanceMatrixProvider;

#[derive(Debug, Clone)]
pub struct OsrmConfig {
    pub base_url: String,
    pub profile: String,
    pub timeout_secs: u64,
}

impl Default for OsrmConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:5000".to_string(),
            profile: "car".to_string(),
            timeout_secs: 10,
        }
    }
}

/// Route geometry response from OSRM
#[derive(Debug, Clone)]
pub struct RouteGeometry {
    /// Encoded polyline for the entire route (Google Polyline Algorithm format)
    pub encoded_polyline: String,
    /// Total distance in meters
    pub distance_meters: i32,
    /// Total duration in seconds
    pub duration_seconds: i32,
    /// Per-leg breakdown (between consecutive waypoints)
    pub legs: Vec<LegGeometry>,
}

/// Geometry for a single leg (segment between two consecutive waypoints)
#[derive(Debug, Clone)]
pub struct LegGeometry {
    /// Encoded polyline for this leg
    pub encoded_polyline: String,
    /// Distance of this leg in meters
    pub distance_meters: i32,
    /// Duration of this leg in seconds
    pub duration_seconds: i32,
}

/// Error type for OSRM route requests
#[derive(Debug)]
pub enum OsrmRouteError {
    /// HTTP request failed
    RequestFailed(String),
    /// OSRM returned an error status
    OsrmError(String),
    /// Failed to parse response
    ParseError(String),
    /// No route found between waypoints
    NoRoute,
}

impl std::fmt::Display for OsrmRouteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OsrmRouteError::RequestFailed(msg) => write!(f, "OSRM request failed: {}", msg),
            OsrmRouteError::OsrmError(msg) => write!(f, "OSRM error: {}", msg),
            OsrmRouteError::ParseError(msg) => write!(f, "Failed to parse OSRM response: {}", msg),
            OsrmRouteError::NoRoute => write!(f, "No route found between waypoints"),
        }
    }
}

impl std::error::Error for OsrmRouteError {}

#[derive(Debug, Clone)]
pub struct OsrmClient {
    config: OsrmConfig,
    client: reqwest::blocking::Client,
}

impl OsrmClient {
    pub fn new(config: OsrmConfig) -> Result<Self, reqwest::Error> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()?;

        Ok(Self { config, client })
    }

    /// Fetch route geometry between ordered waypoints.
    ///
    /// # Arguments
    /// * `waypoints` - Ordered list of (lat, lng) coordinates
    ///
    /// # Returns
    /// Route geometry including encoded polyline for full route and per-leg breakdown.
    /// Returns error if OSRM request fails or no route is found.
    pub fn get_route_geometry(
        &self,
        waypoints: &[(f64, f64)],
    ) -> Result<RouteGeometry, OsrmRouteError> {
        if waypoints.len() < 2 {
            return Err(OsrmRouteError::NoRoute);
        }

        // Build coordinates string: lng1,lat1;lng2,lat2;...
        let coords = waypoints
            .iter()
            .map(|(lat, lng)| format!("{:.6},{:.6}", lng, lat))
            .collect::<Vec<_>>()
            .join(";");

        // Request route with full geometry and per-step annotations
        // overview=full gives us the complete route polyline
        // steps=true with geometries=polyline gives us per-leg polylines
        let url = format!(
            "{}/route/v1/{}/{}?overview=full&geometries=polyline&steps=true",
            self.config.base_url, self.config.profile, coords
        );

        let response = self
            .client
            .get(&url)
            .send()
            .map_err(|e| OsrmRouteError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(OsrmRouteError::RequestFailed(format!(
                "HTTP {}",
                response.status()
            )));
        }

        let body: OsrmRouteResponse = response
            .json()
            .map_err(|e| OsrmRouteError::ParseError(e.to_string()))?;

        // Check OSRM status
        if body.code != "Ok" {
            return Err(OsrmRouteError::OsrmError(body.code));
        }

        // Get the first (best) route
        let route = body.routes.into_iter().next().ok_or(OsrmRouteError::NoRoute)?;

        // Build leg geometries from the route legs
        let legs = route
            .legs
            .into_iter()
            .map(|leg| {
                // Combine step polylines for this leg, or use a fallback
                let leg_polyline = if leg.steps.is_empty() {
                    // No steps available, we'll need to handle this case
                    String::new()
                } else {
                    // Concatenate step geometries or decode/re-encode
                    // For simplicity, we'll use the first step's geometry as approximation
                    // A more accurate approach would decode all steps and merge
                    combine_step_geometries(&leg.steps)
                };

                LegGeometry {
                    encoded_polyline: leg_polyline,
                    distance_meters: leg.distance.round() as i32,
                    duration_seconds: leg.duration.round() as i32,
                }
            })
            .collect();

        Ok(RouteGeometry {
            encoded_polyline: route.geometry,
            distance_meters: route.distance.round() as i32,
            duration_seconds: route.duration.round() as i32,
            legs,
        })
    }
}

impl DistanceMatrixProvider for OsrmClient {
    fn matrix_for(&self, locations: &[(f64, f64)]) -> Vec<Vec<i32>> {
        if locations.is_empty() {
            return Vec::new();
        }

        let coords = locations
            .iter()
            .map(|(lat, lng)| format!("{:.6},{:.6}", lng, lat))
            .collect::<Vec<_>>()
            .join(";");

        let url = format!(
            "{}/table/v1/{}/{}?annotations=duration",
            self.config.base_url, self.config.profile, coords
        );

        let response = self
            .client
            .get(url)
            .send()
            .and_then(|resp| resp.error_for_status())
            .and_then(|resp| resp.json::<OsrmTableResponse>());

        match response {
            Ok(body) => body
                .durations
                .unwrap_or_default()
                .into_iter()
                .map(|row| row.into_iter().map(|value| value.round() as i32).collect())
                .collect(),
            Err(_) => Vec::new(),
        }
    }
}

/// Combine step geometries into a single polyline for the leg.
///
/// This decodes each step's polyline, concatenates the points, and re-encodes.
fn combine_step_geometries(steps: &[OsrmRouteStep]) -> String {
    let mut all_points: Vec<(f64, f64)> = Vec::new();

    for step in steps {
        let points = decode_polyline(&step.geometry);
        if all_points.is_empty() {
            all_points.extend(points);
        } else {
            // Skip first point as it's the same as the last point of previous step
            all_points.extend(points.into_iter().skip(1));
        }
    }

    encode_polyline(&all_points)
}

/// Decode a Google Polyline Algorithm encoded string into coordinates.
fn decode_polyline(encoded: &str) -> Vec<(f64, f64)> {
    let mut points = Vec::new();
    let mut lat = 0i64;
    let mut lng = 0i64;
    let mut index = 0;
    let chars: Vec<char> = encoded.chars().collect();

    while index < chars.len() {
        // Decode latitude
        let mut shift = 0;
        let mut result = 0i64;
        loop {
            if index >= chars.len() {
                break;
            }
            let b = (chars[index] as i64) - 63;
            index += 1;
            result |= (b & 0x1f) << shift;
            shift += 5;
            if b < 0x20 {
                break;
            }
        }
        lat += if (result & 1) != 0 {
            !(result >> 1)
        } else {
            result >> 1
        };

        // Decode longitude
        shift = 0;
        result = 0;
        loop {
            if index >= chars.len() {
                break;
            }
            let b = (chars[index] as i64) - 63;
            index += 1;
            result |= (b & 0x1f) << shift;
            shift += 5;
            if b < 0x20 {
                break;
            }
        }
        lng += if (result & 1) != 0 {
            !(result >> 1)
        } else {
            result >> 1
        };

        points.push((lat as f64 / 1e5, lng as f64 / 1e5));
    }

    points
}

/// Encode coordinates into a Google Polyline Algorithm string.
fn encode_polyline(points: &[(f64, f64)]) -> String {
    let mut encoded = String::new();
    let mut prev_lat = 0i64;
    let mut prev_lng = 0i64;

    for &(lat, lng) in points {
        let lat_e5 = (lat * 1e5).round() as i64;
        let lng_e5 = (lng * 1e5).round() as i64;

        encode_value(lat_e5 - prev_lat, &mut encoded);
        encode_value(lng_e5 - prev_lng, &mut encoded);

        prev_lat = lat_e5;
        prev_lng = lng_e5;
    }

    encoded
}

fn encode_value(mut value: i64, output: &mut String) {
    // Two's complement for negative values
    if value < 0 {
        value = !value;
        value <<= 1;
        value |= 1;
    } else {
        value <<= 1;
    }

    while value >= 0x20 {
        let chunk = ((value & 0x1f) | 0x20) as u8 + 63;
        output.push(chunk as char);
        value >>= 5;
    }

    output.push((value as u8 + 63) as char);
}

// -----------------------------------------------------------------------------
// OSRM Response Types
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct OsrmTableResponse {
    durations: Option<Vec<Vec<f64>>>,
}

#[derive(Debug, Deserialize)]
struct OsrmRouteResponse {
    code: String,
    #[serde(default)]
    routes: Vec<OsrmRoute>,
}

#[derive(Debug, Deserialize)]
struct OsrmRoute {
    /// Encoded polyline for full route
    geometry: String,
    /// Total distance in meters
    distance: f64,
    /// Total duration in seconds
    duration: f64,
    /// Per-leg breakdown
    legs: Vec<OsrmRouteLeg>,
}

#[derive(Debug, Deserialize)]
struct OsrmRouteLeg {
    /// Distance in meters
    distance: f64,
    /// Duration in seconds
    duration: f64,
    /// Steps within this leg (for per-step geometry)
    #[serde(default)]
    steps: Vec<OsrmRouteStep>,
}

#[derive(Debug, Deserialize)]
struct OsrmRouteStep {
    /// Encoded polyline for this step
    geometry: String,
}
