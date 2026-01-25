//! OSRM HTTP adapter for distance matrices.

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

#[derive(Debug, Deserialize)]
struct OsrmTableResponse {
    durations: Option<Vec<Vec<f64>>>,
}
