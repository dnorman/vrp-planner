//! Polyline representation for route geometries.
//!
//! This module provides a type for working with polylines as decoded
//! coordinate sequences. Encoding/decoding happens at the boundary
//! (when receiving from OSRM or sending to frontend).

use serde::{Deserialize, Serialize};

/// A polyline representing a route geometry as decoded coordinates.
///
/// Stores latitude/longitude points directly for internal processing.
/// Encoding to/from the compact polyline format should happen at
/// API boundaries, not within the VRP planner core.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polyline {
    points: Vec<(f64, f64)>,
}

impl Polyline {
    /// Creates a new Polyline from decoded coordinate points.
    ///
    /// Each point is a (latitude, longitude) tuple.
    pub fn new(points: Vec<(f64, f64)>) -> Self {
        Self { points }
    }

    /// Returns a reference to the coordinate points.
    pub fn points(&self) -> &[(f64, f64)] {
        &self.points
    }

    /// Consumes the polyline and returns the owned coordinate points.
    pub fn into_points(self) -> Vec<(f64, f64)> {
        self.points
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_points() {
        let points = vec![(38.5, -120.2), (40.7, -120.95), (43.252, -126.453)];
        let polyline = Polyline::new(points.clone());
        assert_eq!(polyline.points(), &points[..]);
    }

    #[test]
    fn test_into_points() {
        let points = vec![(38.5, -120.2), (40.7, -120.95)];
        let polyline = Polyline::new(points.clone());
        let owned = polyline.into_points();
        assert_eq!(owned, points);
    }

    #[test]
    fn test_empty_polyline() {
        let polyline = Polyline::new(vec![]);
        assert!(polyline.points().is_empty());
    }

    #[test]
    fn test_clone() {
        let polyline = Polyline::new(vec![(1.0, 2.0), (3.0, 4.0)]);
        let cloned = polyline.clone();
        assert_eq!(polyline, cloned);
    }

    #[test]
    fn test_debug() {
        let polyline = Polyline::new(vec![(1.5, 2.5)]);
        let debug_str = format!("{:?}", polyline);
        assert!(debug_str.contains("Polyline"));
        assert!(debug_str.contains("1.5"));
        assert!(debug_str.contains("2.5"));
    }

    #[test]
    fn test_partial_eq() {
        let p1 = Polyline::new(vec![(1.0, 2.0)]);
        let p2 = Polyline::new(vec![(1.0, 2.0)]);
        let p3 = Polyline::new(vec![(1.0, 2.1)]);
        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
    }
}
