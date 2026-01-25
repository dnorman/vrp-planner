//! Core domain traits for the routing planner.
//!
//! These are intentionally minimal and domain-agnostic. Concrete apps should
//! implement them for their own data models.

use std::hash::Hash;

/// Unique identifier for planner entities.
pub trait Id: Clone + Eq + Hash {}

impl<T> Id for T where T: Clone + Eq + Hash {}

/// A visit is a single service occurrence to be routed.
pub trait Visit {
    type Id: Id;
    type VisitorId: Id;

    fn id(&self) -> &Self::Id;

    /// Scheduled date (unix timestamp, date only).
    fn scheduled_date(&self) -> Option<i64>;

    /// Estimated service duration in minutes.
    fn estimated_duration_minutes(&self) -> i32;

    /// Committed window start/end (seconds from midnight).
    fn committed_window(&self) -> Option<(i32, i32)>;

    /// Target time preference (seconds from midnight).
    fn target_time(&self) -> Option<i32>;

    /// Pin type for routing constraints.
    fn pin_type(&self) -> VisitPinType;

    /// Pinned visitor (if any).
    fn pinned_visitor(&self) -> Option<&Self::VisitorId>;

    /// Pinned date (unix timestamp, date only).
    fn pinned_date(&self) -> Option<i64>;

    /// Required capability identifiers for this visit.
    /// Visitor must have ALL of these (superset match).
    fn required_capabilities(&self) -> &[String];

    /// Location coordinates (lat, lng).
    fn location(&self) -> (f64, f64);

    /// Current visitor assignment (for stability penalty).
    /// If the visit is currently assigned to a visitor, reassigning it
    /// to a different visitor incurs a soft penalty.
    fn current_visitor_id(&self) -> Option<&Self::VisitorId> {
        None
    }
}

/// The worker/vehicle that performs visits.
pub trait Visitor {
    type Id: Id;

    fn id(&self) -> &Self::Id;

    /// Start location (lat, lng). If None, solver may use a depot default.
    fn start_location(&self) -> Option<(f64, f64)>;

    /// End location (lat, lng). If None, solver may assume end = start.
    fn end_location(&self) -> Option<(f64, f64)>;

    /// Capability identifiers for this visitor.
    fn capabilities(&self) -> &[String];
}

/// A route plan is a container for a visitor on a specific date.
pub trait RoutePlan {
    type Id: Id;
    type VisitorId: Id;

    fn id(&self) -> &Self::Id;
    fn visitor_id(&self) -> &Self::VisitorId;
    fn service_date(&self) -> i64;
}

/// Provides availability (start/end) for a visitor on a given date.
pub trait AvailabilityProvider {
    type VisitorId: Id;

    fn availability_for(&self, visitor_id: &Self::VisitorId, date: i64) -> Option<(i32, i32)>;
}

/// Provides a distance/time matrix for a set of locations.
///
/// The matrix is indexed by the provided location order.
pub trait DistanceMatrixProvider {
    fn matrix_for(&self, locations: &[(f64, f64)]) -> Vec<Vec<i32>>;
}

/// Pin type for routing constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisitPinType {
    None,
    Visitor,
    Date,
    VisitorAndDate,
}

/// Reason why a visit could not be assigned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnassignedReason {
    /// Visit is pinned to a date that doesn't match the service date.
    WrongDate,
    /// Visit is pinned to a visitor but no pinned_visitor was provided.
    MissingPinnedVisitor,
    /// No visitor has the required capabilities.
    NoCapableVisitor,
    /// No feasible time window could be found (availability or committed window conflict).
    NoFeasibleWindow,
}
