//! Haversine distance matrix provider (fallback when OSRM unavailable).
//!
//! Uses great-circle distance to estimate travel time.
//! Less accurate than OSRM (ignores roads) but always available.

use crate::traits::DistanceMatrixProvider;

/// Average driving speed assumption for time estimation.
const DEFAULT_SPEED_KMH: f64 = 40.0;

/// Earth radius in kilometers.
const EARTH_RADIUS_KM: f64 = 6371.0;

/// Haversine-based distance matrix provider.
///
/// Estimates travel time using straight-line distance and an assumed speed.
/// Useful as a fallback when OSRM is unavailable.
#[derive(Debug, Clone)]
pub struct HaversineMatrix {
    /// Assumed average driving speed in km/h.
    pub speed_kmh: f64,
}

impl Default for HaversineMatrix {
    fn default() -> Self {
        Self {
            speed_kmh: DEFAULT_SPEED_KMH,
        }
    }
}

impl HaversineMatrix {
    pub fn new(speed_kmh: f64) -> Self {
        Self { speed_kmh }
    }

    /// Calculate haversine distance between two points in kilometers.
    fn haversine_km(from: (f64, f64), to: (f64, f64)) -> f64 {
        let (lat1, lng1) = from;
        let (lat2, lng2) = to;

        let lat1_rad = lat1.to_radians();
        let lat2_rad = lat2.to_radians();
        let delta_lat = (lat2 - lat1).to_radians();
        let delta_lng = (lng2 - lng1).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (delta_lng / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().asin();

        EARTH_RADIUS_KM * c
    }

    /// Convert distance in km to travel time in seconds.
    fn km_to_seconds(&self, km: f64) -> i32 {
        let hours = km / self.speed_kmh;
        (hours * 3600.0).round() as i32
    }
}

impl DistanceMatrixProvider for HaversineMatrix {
    fn matrix_for(&self, locations: &[(f64, f64)]) -> Vec<Vec<i32>> {
        let n = locations.len();
        let mut matrix = vec![vec![0; n]; n];

        for (i, from) in locations.iter().enumerate() {
            for (j, to) in locations.iter().enumerate() {
                if i != j {
                    let km = Self::haversine_km(*from, *to);
                    matrix[i][j] = self.km_to_seconds(km);
                }
            }
        }

        matrix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_same_point() {
        let dist = HaversineMatrix::haversine_km((36.1, -115.1), (36.1, -115.1));
        assert!(dist < 0.001, "Same point should have ~0 distance");
    }

    #[test]
    fn test_haversine_known_distance() {
        // Las Vegas (36.17, -115.14) to Los Angeles (34.05, -118.24)
        // Actual distance ~370 km
        let dist = HaversineMatrix::haversine_km((36.17, -115.14), (34.05, -118.24));
        assert!(dist > 350.0 && dist < 400.0, "LV to LA should be ~370km, got {}", dist);
    }

    #[test]
    fn test_matrix_diagonal_is_zero() {
        let provider = HaversineMatrix::default();
        let locations = vec![(36.1, -115.1), (36.2, -115.2), (36.3, -115.3)];
        let matrix = provider.matrix_for(&locations);

        for i in 0..locations.len() {
            assert_eq!(matrix[i][i], 0, "Diagonal should be zero");
        }
    }

    #[test]
    fn test_matrix_symmetric() {
        let provider = HaversineMatrix::default();
        let locations = vec![(36.1, -115.1), (36.2, -115.2)];
        let matrix = provider.matrix_for(&locations);

        // Haversine is symmetric
        assert_eq!(matrix[0][1], matrix[1][0], "Matrix should be symmetric");
    }

    #[test]
    fn test_reasonable_travel_time() {
        let provider = HaversineMatrix::new(40.0); // 40 km/h
        // 10 km at 40 km/h = 0.25 hours = 900 seconds
        let seconds = provider.km_to_seconds(10.0);
        assert_eq!(seconds, 900);
    }
}
