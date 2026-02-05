//! vrp-planner core traits
//!
//! Domain-agnostic interfaces for routing visits into route plans.

pub mod traits;
pub mod solver;
pub mod osrm;
pub mod osrm_data;
pub mod haversine;
pub mod polyline;
