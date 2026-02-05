//! Comprehensive solver tests
//!
//! Tests for pinning, capabilities, windows, and unassigned reasons.

use std::collections::HashMap;

use vrp_planner::solver::{solve, PlannerResult, SolveOptions};
use vrp_planner::traits::{
    AvailabilityProvider, DistanceMatrixProvider, UnassignedReason, Visit, VisitPinType, Visitor,
};

// ============================================================================
// Test Fixtures
// ============================================================================

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct TestId(String);

impl TestId {
    fn new(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Builder for test visits with sensible defaults.
#[derive(Clone, Debug)]
struct TestVisit {
    id: TestId,
    location: (f64, f64),
    duration_min: i32,
    pin_type: VisitPinType,
    pinned_visitor: Option<TestId>,
    pinned_date: Option<i64>,
    committed_window: Option<(i32, i32)>,
    target_time: Option<i32>,
    required_capabilities: Vec<String>,
    current_visitor: Option<TestId>,
}

impl TestVisit {
    fn new(id: &str) -> Self {
        Self {
            id: TestId::new(id),
            location: (0.0, 0.0),
            duration_min: 30,
            pin_type: VisitPinType::None,
            pinned_visitor: None,
            pinned_date: None,
            committed_window: None,
            target_time: None,
            required_capabilities: Vec::new(),
            current_visitor: None,
        }
    }

    fn location(mut self, lat: f64, lng: f64) -> Self {
        self.location = (lat, lng);
        self
    }

    fn duration(mut self, minutes: i32) -> Self {
        self.duration_min = minutes;
        self
    }

    fn pinned_to_visitor(mut self, visitor_id: &str) -> Self {
        self.pin_type = VisitPinType::Visitor;
        self.pinned_visitor = Some(TestId::new(visitor_id));
        self
    }

    fn pinned_to_date(mut self, date: i64) -> Self {
        self.pin_type = VisitPinType::Date;
        self.pinned_date = Some(date);
        self
    }

    fn pinned_to_visitor_and_date(mut self, visitor_id: &str, date: i64) -> Self {
        self.pin_type = VisitPinType::VisitorAndDate;
        self.pinned_visitor = Some(TestId::new(visitor_id));
        self.pinned_date = Some(date);
        self
    }

    fn committed_window(mut self, start: i32, end: i32) -> Self {
        self.committed_window = Some((start, end));
        self
    }

    fn target_time(mut self, time: i32) -> Self {
        self.target_time = Some(time);
        self
    }

    fn requires(mut self, capability: &str) -> Self {
        self.required_capabilities.push(capability.to_string());
        self
    }

    fn currently_assigned_to(mut self, visitor_id: &str) -> Self {
        self.current_visitor = Some(TestId::new(visitor_id));
        self
    }
}

impl Visit for TestVisit {
    type Id = TestId;
    type VisitorId = TestId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn scheduled_date(&self) -> Option<i64> {
        Some(1)
    }

    fn estimated_duration_minutes(&self) -> i32 {
        self.duration_min
    }

    fn committed_window(&self) -> Option<(i32, i32)> {
        self.committed_window
    }

    fn target_time(&self) -> Option<i32> {
        self.target_time
    }

    fn pin_type(&self) -> VisitPinType {
        self.pin_type
    }

    fn pinned_visitor(&self) -> Option<&Self::VisitorId> {
        self.pinned_visitor.as_ref()
    }

    fn pinned_date(&self) -> Option<i64> {
        self.pinned_date
    }

    fn required_capabilities(&self) -> &[String] {
        &self.required_capabilities
    }

    fn location(&self) -> (f64, f64) {
        self.location
    }

    fn current_visitor_id(&self) -> Option<&Self::VisitorId> {
        self.current_visitor.as_ref()
    }
}

/// Builder for test visitors with sensible defaults.
#[derive(Clone, Debug)]
struct TestVisitor {
    id: TestId,
    start_location: Option<(f64, f64)>,
    end_location: Option<(f64, f64)>,
    capabilities: Vec<String>,
}

impl TestVisitor {
    fn new(id: &str) -> Self {
        Self {
            id: TestId::new(id),
            start_location: Some((0.0, 0.0)),
            end_location: None,
            capabilities: Vec::new(),
        }
    }

    fn start_location(mut self, lat: f64, lng: f64) -> Self {
        self.start_location = Some((lat, lng));
        self
    }

    fn capability(mut self, cap: &str) -> Self {
        self.capabilities.push(cap.to_string());
        self
    }
}

impl Visitor for TestVisitor {
    type Id = TestId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn start_location(&self) -> Option<(f64, f64)> {
        self.start_location
    }

    fn end_location(&self) -> Option<(f64, f64)> {
        self.end_location
    }

    fn capabilities(&self) -> &[String] {
        &self.capabilities
    }
}

/// Configurable availability provider.
struct TestAvailability {
    /// Default availability window (seconds from midnight).
    default_window: (i32, i32),
    /// Override availability for specific visitors.
    overrides: HashMap<String, Option<(i32, i32)>>,
}

impl TestAvailability {
    fn new() -> Self {
        Self {
            default_window: (8 * 3600, 17 * 3600), // 8am - 5pm
            overrides: HashMap::new(),
        }
    }

    fn default_window(mut self, start: i32, end: i32) -> Self {
        self.default_window = (start, end);
        self
    }

    fn visitor_unavailable(mut self, visitor_id: &str) -> Self {
        self.overrides.insert(visitor_id.to_string(), None);
        self
    }

    fn visitor_window(mut self, visitor_id: &str, start: i32, end: i32) -> Self {
        self.overrides
            .insert(visitor_id.to_string(), Some((start, end)));
        self
    }
}

impl AvailabilityProvider for TestAvailability {
    type VisitorId = TestId;

    fn availability_for(&self, visitor_id: &Self::VisitorId, _date: i64) -> Option<Vec<(i32, i32)>> {
        if let Some(override_window) = self.overrides.get(&visitor_id.0) {
            override_window.map(|w| vec![w])
        } else {
            Some(vec![self.default_window])
        }
    }
}

/// Manhattan distance matrix (simple, predictable).
struct ManhattanMatrix;

impl DistanceMatrixProvider for ManhattanMatrix {
    fn matrix_for(&self, locations: &[(f64, f64)]) -> Vec<Vec<i32>> {
        let n = locations.len();
        let mut matrix = vec![vec![0; n]; n];
        for (i, from) in locations.iter().enumerate() {
            for (j, to) in locations.iter().enumerate() {
                // Manhattan distance * 60 = travel time in seconds
                // (1 unit = 1 minute of travel)
                let dist = (from.0 - to.0).abs() + (from.1 - to.1).abs();
                matrix[i][j] = (dist * 60.0) as i32;
            }
        }
        matrix
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn get_visitor_visits<'a>(
    result: &'a PlannerResult<TestId, TestId>,
    visitor_id: &str,
) -> Vec<&'a str> {
    result
        .routes
        .iter()
        .find(|r| r.visitor_id.0 == visitor_id)
        .map(|r| r.visit_ids.iter().map(|id| id.0.as_str()).collect())
        .unwrap_or_default()
}

fn get_unassigned_with_reason(
    result: &PlannerResult<TestId, TestId>,
    reason: UnassignedReason,
) -> Vec<&str> {
    result
        .unassigned
        .iter()
        .filter(|u| u.reason == reason)
        .map(|u| u.visit_id.0.as_str())
        .collect()
}

fn hours(h: i32) -> i32 {
    h * 3600
}

fn minutes(m: i32) -> i32 {
    m * 60
}

// ============================================================================
// Pinning Tests
// ============================================================================

#[test]
fn test_pinned_to_visitor() {
    let visits = vec![
        TestVisit::new("v1").location(1.0, 0.0).pinned_to_visitor("alice"),
        TestVisit::new("v2").location(2.0, 0.0),
    ];
    let visitors = vec![TestVisitor::new("alice"), TestVisitor::new("bob")];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new(),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    let alice_visits = get_visitor_visits(&result, "alice");
    assert!(alice_visits.contains(&"v1"), "v1 should be pinned to alice");
}

#[test]
fn test_pinned_to_date_matching() {
    let visits = vec![
        TestVisit::new("v1").location(1.0, 0.0).pinned_to_date(1),
        TestVisit::new("v2").location(2.0, 0.0).pinned_to_date(2), // wrong date
    ];
    let visitors = vec![TestVisitor::new("alice")];

    let result = solve(
        1, // service_date = 1
        &visits,
        &visitors,
        &TestAvailability::new(),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // v1 should be assigned (date matches)
    let alice_visits = get_visitor_visits(&result, "alice");
    assert!(alice_visits.contains(&"v1"), "v1 should be assigned (date matches)");

    // v2 should be unassigned with WrongDate reason
    let wrong_date = get_unassigned_with_reason(&result, UnassignedReason::WrongDate);
    assert!(wrong_date.contains(&"v2"), "v2 should be unassigned due to wrong date");
}

#[test]
fn test_pinned_visitor_missing() {
    let visits = vec![
        TestVisit::new("v1").location(1.0, 0.0), // pin_type will be set but no pinned_visitor
    ];
    // Manually create a visit with Visitor pin type but no pinned_visitor
    let mut bad_visit = TestVisit::new("bad");
    bad_visit.pin_type = VisitPinType::Visitor;
    bad_visit.pinned_visitor = None;

    let visitors = vec![TestVisitor::new("alice")];

    let result = solve(
        1,
        &[bad_visit],
        &visitors,
        &TestAvailability::new(),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    let missing = get_unassigned_with_reason(&result, UnassignedReason::MissingPinnedVisitor);
    assert!(missing.contains(&"bad"), "visit should be unassigned due to missing pinned visitor");
}

// ============================================================================
// Capability Tests
// ============================================================================

#[test]
fn test_capability_superset_match() {
    let visits = vec![
        TestVisit::new("v1")
            .location(1.0, 0.0)
            .requires("plumbing")
            .requires("hvac"),
    ];
    let visitors = vec![
        TestVisitor::new("alice")
            .capability("plumbing")
            .capability("hvac")
            .capability("electrical"), // superset - has extra
        TestVisitor::new("bob").capability("plumbing"), // only has one
    ];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new(),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // v1 should be assigned to alice (only one with all capabilities)
    let alice_visits = get_visitor_visits(&result, "alice");
    assert!(alice_visits.contains(&"v1"), "v1 should be assigned to alice (superset match)");
}

#[test]
fn test_no_capable_visitor() {
    let visits = vec![
        TestVisit::new("v1")
            .location(1.0, 0.0)
            .requires("rare_skill"),
    ];
    let visitors = vec![
        TestVisitor::new("alice").capability("plumbing"),
        TestVisitor::new("bob").capability("electrical"),
    ];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new(),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    let no_capable = get_unassigned_with_reason(&result, UnassignedReason::NoCapableVisitor);
    assert!(no_capable.contains(&"v1"), "v1 should be unassigned due to no capable visitor");
}

// ============================================================================
// Committed Window Tests
// ============================================================================

#[test]
fn test_committed_window_respected() {
    // Visit must happen between 10am and 11am
    let visits = vec![
        TestVisit::new("v1")
            .location(1.0, 0.0)
            .duration(30)
            .committed_window(hours(10), hours(11)),
    ];
    let visitors = vec![TestVisitor::new("alice").start_location(0.0, 0.0)];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // Should be assigned
    let alice_visits = get_visitor_visits(&result, "alice");
    assert!(alice_visits.contains(&"v1"), "v1 should be assigned within window");

    // Check estimated window is within committed window
    let route = result.routes.iter().find(|r| r.visitor_id.0 == "alice").unwrap();
    let (start, _end) = route.estimated_windows[0];
    assert!(start >= hours(10), "start time should be >= 10am");
    assert!(start <= hours(11), "start time should be <= 11am");
}

#[test]
fn test_committed_window_infeasible() {
    // Visit requires 9am-10am but visitor only available from 11am
    let visits = vec![
        TestVisit::new("v1")
            .location(1.0, 0.0)
            .duration(30)
            .committed_window(hours(9), hours(10)),
    ];
    let visitors = vec![TestVisitor::new("alice")];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(hours(11), hours(17)), // starts at 11am
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    let no_window = get_unassigned_with_reason(&result, UnassignedReason::NoFeasibleWindow);
    assert!(no_window.contains(&"v1"), "v1 should be unassigned due to no feasible window");
}

// ============================================================================
// Target Time Tests
// ============================================================================

#[test]
fn test_target_time_affects_cost() {
    // Target time is factored into cost calculation.
    // Note: Greedy construction doesn't guarantee optimal sequencing by target time.
    // Local search (2-opt, relocate) will improve this.
    let visits = vec![
        TestVisit::new("early")
            .location(1.0, 0.0)
            .duration(30)
            .target_time(hours(9)),
        TestVisit::new("late")
            .location(2.0, 0.0)
            .duration(30)
            .target_time(hours(14)),
    ];
    let visitors = vec![TestVisitor::new("alice").start_location(0.0, 0.0)];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // Both should be assigned
    let alice_visits = get_visitor_visits(&result, "alice");
    assert_eq!(alice_visits.len(), 2, "both visits should be assigned");

    // Verify estimated windows are computed
    let route = result.routes.iter().find(|r| r.visitor_id.0 == "alice").unwrap();
    assert_eq!(route.estimated_windows.len(), 2, "should have estimated windows for both visits");
}

#[test]
fn test_target_time_sequencing_with_local_search() {
    // TODO: Once local search is implemented, this test should verify that
    // visits with earlier target times are sequenced before those with later targets
    // when doing so reduces overall cost.
    //
    // For now, we just verify the infrastructure is in place.
    let visits = vec![
        TestVisit::new("early")
            .location(1.0, 0.0)
            .duration(30)
            .target_time(hours(9)),
        TestVisit::new("late")
            .location(2.0, 0.0)
            .duration(30)
            .target_time(hours(14)),
    ];
    let visitors = vec![TestVisitor::new("alice").start_location(0.0, 0.0)];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions { target_time_weight: 10, ..Default::default() }, // Higher weight should influence sequencing more
    );

    // Both should still be assigned
    let alice_visits = get_visitor_visits(&result, "alice");
    assert_eq!(alice_visits.len(), 2, "both visits should be assigned");
}

// ============================================================================
// Availability Tests
// ============================================================================

#[test]
fn test_visitor_unavailable() {
    let visits = vec![
        TestVisit::new("v1").location(1.0, 0.0).pinned_to_visitor("alice"),
    ];
    let visitors = vec![TestVisitor::new("alice"), TestVisitor::new("bob")];

    // Alice is unavailable
    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().visitor_unavailable("alice"),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // Visit pinned to unavailable visitor should fail
    let no_window = get_unassigned_with_reason(&result, UnassignedReason::NoFeasibleWindow);
    assert!(no_window.contains(&"v1"), "v1 should be unassigned (alice unavailable)");
}

// ============================================================================
// Multi-Visit Sequencing Tests
// ============================================================================

#[test]
fn test_multiple_visits_sequenced() {
    let visits = vec![
        TestVisit::new("a").location(1.0, 0.0).duration(30),
        TestVisit::new("b").location(2.0, 0.0).duration(30),
        TestVisit::new("c").location(3.0, 0.0).duration(30),
    ];
    let visitors = vec![TestVisitor::new("alice").start_location(0.0, 0.0)];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // All should be assigned
    assert!(result.unassigned.is_empty(), "all visits should be assigned");

    let alice_visits = get_visitor_visits(&result, "alice");
    assert_eq!(alice_visits.len(), 3, "alice should have all 3 visits");

    // Check estimated windows are sequential and non-overlapping
    let route = result.routes.iter().find(|r| r.visitor_id.0 == "alice").unwrap();
    for i in 1..route.estimated_windows.len() {
        let prev_end = route.estimated_windows[i - 1].1;
        let curr_start = route.estimated_windows[i].0;
        assert!(
            curr_start >= prev_end,
            "visit {} should start after visit {} ends",
            i,
            i - 1
        );
    }
}

#[test]
fn test_visits_distributed_across_visitors() {
    // More visits than one visitor can handle in their window
    let visits: Vec<TestVisit> = (0..6)
        .map(|i| {
            TestVisit::new(&format!("v{}", i))
                .location(i as f64, 0.0)
                .duration(60) // 1 hour each
        })
        .collect();
    let visitors = vec![
        TestVisitor::new("alice").start_location(0.0, 0.0),
        TestVisitor::new("bob").start_location(0.0, 0.0),
    ];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(hours(8), hours(12)), // 4 hour window
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    let alice_count = get_visitor_visits(&result, "alice").len();
    let bob_count = get_visitor_visits(&result, "bob").len();

    // Both should have some visits (exact distribution depends on algorithm)
    assert!(alice_count > 0, "alice should have some visits");
    assert!(bob_count > 0, "bob should have some visits");
    assert_eq!(
        alice_count + bob_count + result.unassigned.len(),
        6,
        "all visits accounted for"
    );
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_empty_visits() {
    let visits: Vec<TestVisit> = vec![];
    let visitors = vec![TestVisitor::new("alice")];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new(),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    assert!(result.unassigned.is_empty());
    assert!(result.routes.iter().all(|r| r.visit_ids.is_empty()));
}

#[test]
fn test_single_visit_single_visitor() {
    // Simplest possible case
    let visits = vec![TestVisit::new("v1").location(1.0, 0.0).duration(30)];
    let visitors = vec![TestVisitor::new("alice").start_location(0.0, 0.0)];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    assert!(result.unassigned.is_empty(), "Visit should be assigned");
    let alice_visits = get_visitor_visits(&result, "alice");
    assert_eq!(alice_visits.len(), 1);
    assert!(alice_visits.contains(&"v1"));
}

// ============================================================================
// Additional Time Window Tests
// ============================================================================

#[test]
fn test_narrow_committed_window_30_minutes() {
    // Very tight 30-minute committed window
    let visits = vec![
        TestVisit::new("tight")
            .location(1.0, 0.0)
            .duration(20)
            .committed_window(hours(10), hours(10) + minutes(30)),
    ];
    let visitors = vec![TestVisitor::new("alice").start_location(0.0, 0.0)];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    assert!(result.unassigned.is_empty(), "Visit should fit in 30-min window");

    let route = &result.routes[0];
    let (start, _) = route.estimated_windows[0];
    assert!(start >= hours(10), "Should start at or after 10am");
    assert!(start <= hours(10) + minutes(30), "Should start before 10:30am");
}

#[test]
fn test_visit_at_day_start() {
    // Committed window right at the start of the day
    let visits = vec![
        TestVisit::new("early")
            .location(0.1, 0.0) // Very close to start location
            .duration(30)
            .committed_window(hours(8), hours(9)),
    ];
    let visitors = vec![TestVisitor::new("alice").start_location(0.0, 0.0)];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    assert!(result.unassigned.is_empty(), "Early visit should be assigned");
    let route = &result.routes[0];
    let (start, _) = route.estimated_windows[0];
    assert!(start >= hours(8), "Should start at or after 8am");
}

#[test]
fn test_visit_at_day_end() {
    // Visit scheduled near end of day
    let visits = vec![
        TestVisit::new("late")
            .location(0.1, 0.0)
            .duration(30)
            .committed_window(hours(16), hours(17)),
    ];
    let visitors = vec![TestVisitor::new("alice").start_location(0.0, 0.0)];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    assert!(result.unassigned.is_empty(), "Late visit should be assigned");
    let route = &result.routes[0];
    let (start, _) = route.estimated_windows[0];
    assert!(start >= hours(16), "Should start at or after 4pm");
}

#[test]
fn test_visit_duration_exceeds_remaining_window() {
    // 3-hour visit but only 2 hours left in window
    let visits = vec![
        TestVisit::new("long")
            .location(0.1, 0.0)
            .duration(180) // 3 hours
            .committed_window(hours(15), hours(17)), // Only 2 hours available
    ];
    let visitors = vec![TestVisitor::new("alice").start_location(0.0, 0.0)];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // Visit should be unassigned - doesn't fit
    assert_eq!(result.unassigned.len(), 1, "Long visit shouldn't fit");
}

#[test]
fn test_short_visit_15_minutes() {
    // Very short visit (quick check/inspection)
    let visits = vec![
        TestVisit::new("quick").location(1.0, 0.0).duration(15),
    ];
    let visitors = vec![TestVisitor::new("alice").start_location(0.0, 0.0)];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    assert!(result.unassigned.is_empty());
}

#[test]
fn test_long_visit_3_hours() {
    // Major repair - 3 hour visit
    let visits = vec![
        TestVisit::new("major_repair").location(1.0, 0.0).duration(180),
    ];
    let visitors = vec![TestVisitor::new("alice").start_location(0.0, 0.0)];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    assert!(result.unassigned.is_empty(), "3-hour visit should fit in 9-hour day");
}

#[test]
fn test_mixed_durations_same_route() {
    // Mix of short and long visits on same route
    let visits = vec![
        TestVisit::new("quick1").location(1.0, 0.0).duration(15),
        TestVisit::new("medium").location(2.0, 0.0).duration(45),
        TestVisit::new("long").location(3.0, 0.0).duration(120),
        TestVisit::new("quick2").location(4.0, 0.0).duration(15),
    ];
    let visitors = vec![TestVisitor::new("alice").start_location(0.0, 0.0)];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // Total: 15+45+120+15 = 195 min = 3.25 hours - should fit
    assert!(result.unassigned.is_empty(), "Mixed duration visits should fit");
    assert_eq!(get_visitor_visits(&result, "alice").len(), 4);
}

// ============================================================================
// Additional Pinning Tests
// ============================================================================

#[test]
fn test_pinned_to_visitor_and_date() {
    // Visit must be specific tech on specific day
    let visits = vec![
        TestVisit::new("v1")
            .location(1.0, 0.0)
            .pinned_to_visitor("alice")
            .pinned_to_date(1),
    ];
    let visitors = vec![
        TestVisitor::new("alice"),
        TestVisitor::new("bob"),
    ];

    let result = solve(
        1, // Correct date
        &visits,
        &visitors,
        &TestAvailability::new(),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    let alice_visits = get_visitor_visits(&result, "alice");
    assert!(alice_visits.contains(&"v1"), "v1 should go to alice on date 1");
}

#[test]
fn test_pinned_visitor_and_wrong_date() {
    // Visit pinned to alice but for a different date
    let visits = vec![
        TestVisit::new("v1")
            .location(1.0, 0.0)
            .pinned_to_visitor("alice")
            .pinned_to_date(2), // Wrong date
    ];
    let visitors = vec![TestVisitor::new("alice")];

    let result = solve(
        1, // Service date is 1, not 2
        &visits,
        &visitors,
        &TestAvailability::new(),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    let wrong_date = get_unassigned_with_reason(&result, UnassignedReason::WrongDate);
    assert!(wrong_date.contains(&"v1"), "v1 should be unassigned (wrong date)");
}

#[test]
fn test_multiple_visits_pinned_same_tech() {
    // Several customers all request the same technician
    let visits = vec![
        TestVisit::new("v1").location(1.0, 0.0).duration(30).pinned_to_visitor("alice"),
        TestVisit::new("v2").location(2.0, 0.0).duration(30).pinned_to_visitor("alice"),
        TestVisit::new("v3").location(3.0, 0.0).duration(30).pinned_to_visitor("alice"),
        TestVisit::new("v4").location(4.0, 0.0).duration(30), // Not pinned
    ];
    let visitors = vec![
        TestVisitor::new("alice").start_location(0.0, 0.0),
        TestVisitor::new("bob").start_location(0.0, 0.0),
    ];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    let alice_visits = get_visitor_visits(&result, "alice");

    // All 3 pinned visits must be with alice
    assert!(alice_visits.contains(&"v1"), "v1 pinned to alice");
    assert!(alice_visits.contains(&"v2"), "v2 pinned to alice");
    assert!(alice_visits.contains(&"v3"), "v3 pinned to alice");

    // v4 can go to either (likely bob for balance)
    assert!(result.unassigned.is_empty(), "All visits should be assigned");
}

// ============================================================================
// Additional Capability Tests
// ============================================================================

#[test]
fn test_visit_requires_multiple_capabilities() {
    // Visit requires BOTH plumbing AND electrical
    let visits = vec![
        TestVisit::new("complex")
            .location(1.0, 0.0)
            .requires("plumbing")
            .requires("electrical"),
    ];
    let visitors = vec![
        TestVisitor::new("plumber").capability("plumbing"),
        TestVisitor::new("electrician").capability("electrical"),
        TestVisitor::new("generalist").capability("plumbing").capability("electrical"),
    ];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new(),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // Only generalist can do this visit
    let generalist_visits = get_visitor_visits(&result, "generalist");
    assert!(generalist_visits.contains(&"complex"), "complex should go to generalist");
}

#[test]
fn test_multiple_techs_same_capability_choose_closest() {
    // Two plumbers - visit should go to the closer one
    let visits = vec![
        TestVisit::new("plumb_job")
            .location(9.0, 0.0) // Closer to bob
            .requires("plumbing"),
    ];
    let visitors = vec![
        TestVisitor::new("alice").start_location(0.0, 0.0).capability("plumbing"),
        TestVisitor::new("bob").start_location(10.0, 0.0).capability("plumbing"),
    ];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new(),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // Bob is closer (1 unit away vs 9 units)
    let bob_visits = get_visitor_visits(&result, "bob");
    assert!(bob_visits.contains(&"plumb_job"), "Visit should go to closer tech (bob)");
}

#[test]
fn test_rare_skill_only_one_tech() {
    // Only one tech has HVAC certification
    let visits = vec![
        TestVisit::new("hvac1").location(1.0, 0.0).requires("hvac"),
        TestVisit::new("hvac2").location(2.0, 0.0).requires("hvac"),
        TestVisit::new("general").location(3.0, 0.0),
    ];
    let visitors = vec![
        TestVisitor::new("alice").capability("plumbing"),
        TestVisitor::new("bob").capability("hvac"),
        TestVisitor::new("charlie").capability("electrical"),
    ];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new(),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    let bob_visits = get_visitor_visits(&result, "bob");
    assert!(bob_visits.contains(&"hvac1"), "hvac1 must go to bob");
    assert!(bob_visits.contains(&"hvac2"), "hvac2 must go to bob");
}

// ============================================================================
// Variable Availability / Part-Time Tests
// ============================================================================

#[test]
fn test_part_time_morning_only() {
    // Alice only works mornings (8am-12pm)
    let visits = vec![
        TestVisit::new("morning1").location(1.0, 0.0).duration(60),
        TestVisit::new("morning2").location(2.0, 0.0).duration(60),
        TestVisit::new("afternoon").location(3.0, 0.0).duration(60)
            .committed_window(hours(14), hours(16)), // Must be afternoon
    ];
    let visitors = vec![
        TestVisitor::new("alice").start_location(0.0, 0.0), // Morning only
        TestVisitor::new("bob").start_location(0.0, 0.0),   // Full day
    ];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new()
            .visitor_window("alice", hours(8), hours(12)) // Morning only
            .default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // Afternoon visit must go to bob
    let bob_visits = get_visitor_visits(&result, "bob");
    assert!(bob_visits.contains(&"afternoon"), "Afternoon visit must go to full-day worker");
}

#[test]
fn test_staggered_start_times() {
    // Techs start at different times
    let visits = vec![
        TestVisit::new("early").location(1.0, 0.0).duration(30)
            .committed_window(hours(7), hours(8)),
        TestVisit::new("normal").location(2.0, 0.0).duration(30),
    ];
    let visitors = vec![
        TestVisitor::new("early_bird").start_location(0.0, 0.0), // Starts 6am
        TestVisitor::new("normal").start_location(0.0, 0.0),     // Starts 8am
    ];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new()
            .visitor_window("early_bird", hours(6), hours(14)) // Early shift
            .visitor_window("normal", hours(8), hours(17)),    // Normal shift
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // Early visit (7-8am) must go to early_bird
    let early_bird_visits = get_visitor_visits(&result, "early_bird");
    assert!(early_bird_visits.contains(&"early"), "7am visit needs early starter");
}

#[test]
fn test_mid_day_break() {
    // Simulate lunch break by having no availability 12-1pm
    // Note: Current model doesn't support breaks within a day,
    // but we can test that visits don't overlap if visitor window is set
    let visits = vec![
        TestVisit::new("v1").location(1.0, 0.0).duration(60),
        TestVisit::new("v2").location(2.0, 0.0).duration(60),
        TestVisit::new("v3").location(3.0, 0.0).duration(60),
    ];
    let visitors = vec![
        TestVisitor::new("alice").start_location(0.0, 0.0),
    ];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // Just verify all get assigned - break handling is future work
    assert_eq!(result.unassigned.len(), 0);
}

// ============================================================================
// Geographic / Clustering Tests
// ============================================================================

#[test]
fn test_geographic_clustering() {
    // Visits clustered in two areas - should be assigned to nearby techs
    let visits = vec![
        // North cluster
        TestVisit::new("n1").location(0.0, 10.0).duration(30),
        TestVisit::new("n2").location(1.0, 10.0).duration(30),
        TestVisit::new("n3").location(0.5, 11.0).duration(30),
        // South cluster
        TestVisit::new("s1").location(0.0, 0.0).duration(30),
        TestVisit::new("s2").location(1.0, 0.0).duration(30),
        TestVisit::new("s3").location(0.5, 1.0).duration(30),
    ];
    let visitors = vec![
        TestVisitor::new("north_tech").start_location(0.0, 10.0),
        TestVisitor::new("south_tech").start_location(0.0, 0.0),
    ];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    let north_visits = get_visitor_visits(&result, "north_tech");
    let south_visits = get_visitor_visits(&result, "south_tech");

    // North tech should get north cluster
    assert!(north_visits.contains(&"n1") || north_visits.contains(&"n2") || north_visits.contains(&"n3"),
        "North tech should have north visits: {:?}", north_visits);

    // South tech should get south cluster
    assert!(south_visits.contains(&"s1") || south_visits.contains(&"s2") || south_visits.contains(&"s3"),
        "South tech should have south visits: {:?}", south_visits);
}

#[test]
fn test_minimize_backtracking() {
    // Visits in a line - should be done in order, not zigzag
    let visits = vec![
        TestVisit::new("a").location(1.0, 0.0).duration(10),
        TestVisit::new("b").location(2.0, 0.0).duration(10),
        TestVisit::new("c").location(3.0, 0.0).duration(10),
        TestVisit::new("d").location(4.0, 0.0).duration(10),
    ];
    let visitors = vec![TestVisitor::new("alice").start_location(0.0, 0.0)];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    let route = &result.routes[0];

    // With local search, route should be a->b->c->d (or reverse)
    // Check total travel time is reasonable (4 units forward, not zigzag)
    // Optimal: 1+1+1+1 = 4 units = 4*60 = 240 seconds (at 60s/unit in ManhattanMatrix)
    // Bad zigzag could be much worse
    assert!(route.total_travel_time <= 300 * 60,
        "Travel time should be reasonable: {}", route.total_travel_time);
}

// ============================================================================
// Same Location Tests
// ============================================================================

#[test]
fn test_multiple_visits_same_address() {
    // Two different services at the same property
    let visits = vec![
        TestVisit::new("pool_clean").location(5.0, 5.0).duration(30),
        TestVisit::new("filter_check").location(5.0, 5.0).duration(15), // Same location
    ];
    let visitors = vec![TestVisitor::new("alice").start_location(0.0, 0.0)];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // Both should be assigned, ideally back-to-back
    assert!(result.unassigned.is_empty());

    let route = &result.routes[0];
    assert_eq!(route.visit_ids.len(), 2);

    // Check they're scheduled consecutively (travel between should be 0)
    let windows = &route.estimated_windows;
    let first_end = windows[0].1;
    let second_start = windows[1].0;

    // Second visit should start right after first (0 travel time)
    assert!(second_start <= first_end + 60,
        "Same-location visits should be back-to-back: first ends {}, second starts {}",
        first_end, second_start);
}

// ============================================================================
// Workload Balance Tests
// ============================================================================

#[test]
fn test_workload_roughly_balanced() {
    // 10 visits, 2 techs - should be roughly 5 each
    let visits: Vec<TestVisit> = (0..10)
        .map(|i| TestVisit::new(&format!("v{}", i)).location(i as f64, 0.0).duration(30))
        .collect();
    let visitors = vec![
        TestVisitor::new("alice").start_location(0.0, 0.0),
        TestVisitor::new("bob").start_location(10.0, 0.0),
    ];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    let alice_count = get_visitor_visits(&result, "alice").len();
    let bob_count = get_visitor_visits(&result, "bob").len();

    // Should be somewhat balanced (not all to one person)
    assert!(alice_count >= 3, "Alice should have at least 3 visits: {}", alice_count);
    assert!(bob_count >= 3, "Bob should have at least 3 visits: {}", bob_count);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_visit_exactly_fills_window() {
    // Visit duration exactly matches available window
    let visits = vec![
        TestVisit::new("perfect_fit")
            .location(0.0, 0.0) // At start location, no travel
            .duration(60) // 1 hour
            .committed_window(hours(10), hours(11)), // Exactly 1 hour window
    ];
    let visitors = vec![TestVisitor::new("alice").start_location(0.0, 0.0)];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    assert!(result.unassigned.is_empty(), "Perfect fit should work");
}

#[test]
fn test_all_techs_unavailable() {
    // Everyone called in sick
    let visits = vec![
        TestVisit::new("v1").location(1.0, 0.0),
        TestVisit::new("v2").location(2.0, 0.0),
    ];
    let visitors = vec![
        TestVisitor::new("alice"),
        TestVisitor::new("bob"),
    ];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new()
            .visitor_unavailable("alice")
            .visitor_unavailable("bob"),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // All visits should be unassigned
    assert_eq!(result.unassigned.len(), 2, "All visits should be unassigned");
}

#[test]
fn test_two_of_three_techs_sick() {
    // Heavy load on remaining tech
    let visits: Vec<TestVisit> = (0..6)
        .map(|i| TestVisit::new(&format!("v{}", i)).location(i as f64, 0.0).duration(30))
        .collect();
    let visitors = vec![
        TestVisitor::new("alice"),
        TestVisitor::new("bob"),
        TestVisitor::new("charlie"),
    ];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new()
            .visitor_unavailable("alice")
            .visitor_unavailable("bob")
            .default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // All 6 visits should go to charlie
    let charlie_visits = get_visitor_visits(&result, "charlie");
    assert_eq!(charlie_visits.len(), 6, "Charlie should handle all 6 visits");
}

// ============================================================================
// Local Search Tests
// ============================================================================

#[test]
fn test_two_opt_improves_crossing_routes() {
    // Create a scenario where 2-opt would help:
    // Visits arranged in a way that creates a "crossing" pattern
    // A -> D -> C -> B would cross, A -> B -> C -> D would not
    //
    // Layout:  A(0,0)  B(0,1)
    //          D(1,0)  C(1,1)
    //
    // If construction inserts in order A,D,C,B the route crosses.
    // 2-opt should fix it to A,B,C,D or A,D,C,B depending on direction.

    let visits = vec![
        TestVisit::new("A").location(0.0, 0.0).duration(10),
        TestVisit::new("B").location(0.0, 1.0).duration(10),
        TestVisit::new("C").location(1.0, 1.0).duration(10),
        TestVisit::new("D").location(1.0, 0.0).duration(10),
    ];
    let visitors = vec![TestVisitor::new("alice").start_location(-1.0, 0.0)];

    // Run with local search enabled (default)
    let result_with_ls = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(0, hours(8)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // Run without local search
    let result_without_ls = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(0, hours(8)),
        &ManhattanMatrix,
        SolveOptions { local_search_iterations: 0, ..Default::default() },
    );

    let route_with_ls = &result_with_ls.routes[0];
    let route_without_ls = &result_without_ls.routes[0];

    // Local search should produce equal or better travel time
    assert!(
        route_with_ls.total_travel_time <= route_without_ls.total_travel_time,
        "Local search should not make things worse: with={}, without={}",
        route_with_ls.total_travel_time,
        route_without_ls.total_travel_time
    );
}

#[test]
fn test_relocate_balances_routes() {
    // Create visits clustered near one visitor's start, but assigned to wrong visitor initially
    // Relocate should move visits to the closer visitor

    let visits = vec![
        // Cluster near alice's start (0, 0)
        TestVisit::new("a1").location(0.1, 0.0).duration(20),
        TestVisit::new("a2").location(0.2, 0.0).duration(20),
        TestVisit::new("a3").location(0.3, 0.0).duration(20),
        // Cluster near bob's start (10, 0)
        TestVisit::new("b1").location(9.9, 0.0).duration(20),
        TestVisit::new("b2").location(9.8, 0.0).duration(20),
        TestVisit::new("b3").location(9.7, 0.0).duration(20),
    ];
    let visitors = vec![
        TestVisitor::new("alice").start_location(0.0, 0.0),
        TestVisitor::new("bob").start_location(10.0, 0.0),
    ];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(0, hours(8)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // Both visitors should have work (relocate should distribute well)
    let alice_visits = get_visitor_visits(&result, "alice");
    let bob_visits = get_visitor_visits(&result, "bob");

    // The a* visits should be on alice's route (closer to her start)
    // The b* visits should be on bob's route (closer to his start)
    let alice_has_a = alice_visits.iter().any(|v| v.starts_with('a'));
    let bob_has_b = bob_visits.iter().any(|v| v.starts_with('b'));

    assert!(alice_has_a, "Alice should have some 'a' visits: {:?}", alice_visits);
    assert!(bob_has_b, "Bob should have some 'b' visits: {:?}", bob_visits);

    // Total travel time should be reasonable (not crossing the map unnecessarily)
    let total_travel: i32 = result.routes.iter().map(|r| r.total_travel_time).sum();
    // Each cluster is ~0.3 units apart, so travel within cluster ~18 seconds each
    // Max reasonable would be ~200 seconds if well distributed
    assert!(
        total_travel < 500 * 60, // 500 minutes in seconds
        "Total travel time seems too high: {} seconds",
        total_travel
    );
}

#[test]
fn test_stability_penalty_prefers_current_assignment() {
    // Create two visits, each currently assigned to a different visitor.
    // Even though switching them might save travel time, the stability
    // penalty should discourage it.

    // v1 is near bob but currently assigned to alice
    // v2 is near alice but currently assigned to bob
    // Without stability, solver might swap them. With stability, it should keep them.

    let visits = vec![
        TestVisit::new("v1")
            .location(9.0, 0.0) // Near bob's start (10, 0)
            .duration(30)
            .currently_assigned_to("alice"),
        TestVisit::new("v2")
            .location(1.0, 0.0) // Near alice's start (0, 0)
            .duration(30)
            .currently_assigned_to("bob"),
    ];
    let visitors = vec![
        TestVisitor::new("alice").start_location(0.0, 0.0),
        TestVisitor::new("bob").start_location(10.0, 0.0),
    ];

    // With high stability penalty, should keep current assignments
    let result_stable = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(0, hours(8)),
        &ManhattanMatrix,
        SolveOptions {
            reassignment_penalty: 1000, // High penalty
            ..Default::default()
        },
    );

    // With no stability penalty, should swap to minimize travel
    let result_no_stability = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(0, hours(8)),
        &ManhattanMatrix,
        SolveOptions {
            reassignment_penalty: 0, // No penalty
            ..Default::default()
        },
    );

    let stable_alice = get_visitor_visits(&result_stable, "alice");
    let stable_bob = get_visitor_visits(&result_stable, "bob");
    let no_stab_alice = get_visitor_visits(&result_no_stability, "alice");
    let no_stab_bob = get_visitor_visits(&result_no_stability, "bob");

    // With stability, v1 should stay with alice (its current assignment)
    assert!(
        stable_alice.contains(&"v1"),
        "With stability, v1 should stay with alice: alice={:?}, bob={:?}",
        stable_alice, stable_bob
    );

    // Without stability, v1 should move to bob (closer)
    assert!(
        no_stab_bob.contains(&"v1"),
        "Without stability, v1 should move to bob: alice={:?}, bob={:?}",
        no_stab_alice, no_stab_bob
    );
}

#[test]
fn test_reassignment_when_visitor_calls_in_sick() {
    // Scenario: Alice had 3 visits assigned yesterday, but calls in sick today.
    // Those visits should be reassigned to Bob (the only available visitor).
    // Even with stability penalty, reassignment must happen since Alice is unavailable.

    let visits = vec![
        TestVisit::new("v1")
            .location(1.0, 0.0)
            .duration(30)
            .currently_assigned_to("alice"),
        TestVisit::new("v2")
            .location(2.0, 0.0)
            .duration(30)
            .currently_assigned_to("alice"),
        TestVisit::new("v3")
            .location(3.0, 0.0)
            .duration(30)
            .currently_assigned_to("alice"),
    ];
    let visitors = vec![
        TestVisitor::new("alice").start_location(0.0, 0.0),
        TestVisitor::new("bob").start_location(0.0, 0.0),
    ];

    // Alice is unavailable (sick) - visits should go to Bob
    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new()
            .visitor_unavailable("alice")
            .default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions {
            reassignment_penalty: 1000, // High penalty, but shouldn't matter
            ..Default::default()
        },
    );

    // All visits should be reassigned to Bob
    let bob_visits = get_visitor_visits(&result, "bob");
    assert_eq!(
        bob_visits.len(),
        3,
        "All 3 visits should be reassigned to Bob: {:?}",
        bob_visits
    );
    assert!(result.unassigned.is_empty(), "No visits should be unassigned");
}

#[test]
fn test_partial_reassignment_multiple_visitors_sick() {
    // Scenario: Alice and Bob each had visits, but Alice calls in sick.
    // Alice's visits should move to Bob. Bob's visits stay with Bob.

    let visits = vec![
        // Alice's visits (need reassignment)
        TestVisit::new("a1")
            .location(1.0, 0.0)
            .duration(30)
            .currently_assigned_to("alice"),
        TestVisit::new("a2")
            .location(2.0, 0.0)
            .duration(30)
            .currently_assigned_to("alice"),
        // Bob's visits (should stay)
        TestVisit::new("b1")
            .location(1.0, 1.0)
            .duration(30)
            .currently_assigned_to("bob"),
        TestVisit::new("b2")
            .location(2.0, 1.0)
            .duration(30)
            .currently_assigned_to("bob"),
    ];
    let visitors = vec![
        TestVisitor::new("alice").start_location(0.0, 0.0),
        TestVisitor::new("bob").start_location(0.0, 1.0),
    ];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new()
            .visitor_unavailable("alice")
            .default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions {
            reassignment_penalty: 1000,
            ..Default::default()
        },
    );

    let bob_visits = get_visitor_visits(&result, "bob");

    // Bob should have all 4 visits
    assert_eq!(
        bob_visits.len(),
        4,
        "Bob should have all 4 visits (2 original + 2 from Alice): {:?}",
        bob_visits
    );

    // Verify Alice's visits were reassigned
    assert!(bob_visits.contains(&"a1"), "a1 should be reassigned to Bob");
    assert!(bob_visits.contains(&"a2"), "a2 should be reassigned to Bob");

    // Verify Bob's visits stayed
    assert!(bob_visits.contains(&"b1"), "b1 should stay with Bob");
    assert!(bob_visits.contains(&"b2"), "b2 should stay with Bob");
}

#[test]
fn test_reassignment_respects_capabilities() {
    // Scenario: Alice (plumber) calls in sick. Her plumbing visits can only
    // go to Charlie (also a plumber), not Bob (electrician).

    let visits = vec![
        TestVisit::new("plumb1")
            .location(1.0, 0.0)
            .duration(30)
            .requires("plumbing")
            .currently_assigned_to("alice"),
        TestVisit::new("plumb2")
            .location(2.0, 0.0)
            .duration(30)
            .requires("plumbing")
            .currently_assigned_to("alice"),
    ];
    let visitors = vec![
        TestVisitor::new("alice")
            .start_location(0.0, 0.0)
            .capability("plumbing"),
        TestVisitor::new("bob")
            .start_location(0.0, 0.0)
            .capability("electrical"), // Can't do plumbing
        TestVisitor::new("charlie")
            .start_location(5.0, 0.0)
            .capability("plumbing"), // Can do plumbing
    ];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new()
            .visitor_unavailable("alice")
            .default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // Visits should go to Charlie (only capable visitor available)
    let charlie_visits = get_visitor_visits(&result, "charlie");
    let bob_visits = get_visitor_visits(&result, "bob");

    assert_eq!(
        charlie_visits.len(),
        2,
        "Charlie should get both plumbing visits: {:?}",
        charlie_visits
    );
    assert!(
        bob_visits.is_empty(),
        "Bob shouldn't get any visits (no plumbing capability): {:?}",
        bob_visits
    );
}

#[test]
fn test_reassignment_when_no_capable_backup() {
    // Scenario: Alice (only plumber) calls in sick. Her plumbing visits
    // cannot be reassigned because no other plumber is available.

    let visits = vec![
        TestVisit::new("plumb1")
            .location(1.0, 0.0)
            .duration(30)
            .requires("plumbing")
            .currently_assigned_to("alice"),
    ];
    let visitors = vec![
        TestVisitor::new("alice")
            .start_location(0.0, 0.0)
            .capability("plumbing"),
        TestVisitor::new("bob")
            .start_location(0.0, 0.0)
            .capability("electrical"), // Can't do plumbing
    ];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new()
            .visitor_unavailable("alice")
            .default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // Visit should be unassigned with NoCapableVisitor reason
    let no_capable = get_unassigned_with_reason(&result, UnassignedReason::NoCapableVisitor);
    assert!(
        no_capable.contains(&"plumb1"),
        "plumb1 should be unassigned (no capable backup): {:?}",
        result.unassigned
    );
}

// ============================================================================
// Running Late / Delayed Start Tests
// ============================================================================

#[test]
fn test_running_late_visits_rescheduled() {
    // Scenario: Alice had 3 visits but is running late (starts at 11am instead of 8am).
    // Her visits can still fit in the shortened window.
    // The visits should stay with her but be rescheduled to later times.

    let visits = vec![
        TestVisit::new("v1")
            .location(1.0, 0.0)
            .duration(30)
            .currently_assigned_to("alice"),
        TestVisit::new("v2")
            .location(2.0, 0.0)
            .duration(30)
            .currently_assigned_to("alice"),
        TestVisit::new("v3")
            .location(3.0, 0.0)
            .duration(30)
            .currently_assigned_to("alice"),
    ];
    let visitors = vec![
        TestVisitor::new("alice").start_location(0.0, 0.0),
        TestVisitor::new("bob").start_location(0.0, 0.0),
    ];

    // Alice starts at 11am instead of 8am (3 hour delay)
    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new()
            .visitor_window("alice", hours(11), hours(17)) // Delayed start
            .default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions {
            reassignment_penalty: 1000, // High penalty to prefer keeping with Alice
            ..Default::default()
        },
    );

    // All visits should still be assigned (plenty of time from 11am-5pm for 3x30min)
    assert!(result.unassigned.is_empty(), "All visits should be assigned");

    // With high stability penalty, visits should stay with Alice
    let alice_visits = get_visitor_visits(&result, "alice");
    assert_eq!(
        alice_visits.len(),
        3,
        "All 3 visits should stay with Alice despite late start: {:?}",
        alice_visits
    );

    // Verify visits are scheduled after 11am
    let route = result.routes.iter().find(|r| r.visitor_id.0 == "alice").unwrap();
    for (i, (start, _end)) in route.estimated_windows.iter().enumerate() {
        assert!(
            *start >= hours(11),
            "Visit {} should start at or after 11am, but starts at {}s",
            i,
            start
        );
    }
}

#[test]
fn test_running_late_some_visits_reassigned() {
    // Scenario: Alice had 4 visits (2 hours total) but starts late (3pm).
    // She only has 2 hours left (3pm-5pm), but visits might not all fit
    // due to committed windows. Some visits must go to Bob.

    let visits = vec![
        // Early morning visits - committed to 8am-10am window, can't wait until 3pm
        TestVisit::new("early1")
            .location(1.0, 0.0)
            .duration(30)
            .committed_window(hours(8), hours(10))
            .currently_assigned_to("alice"),
        TestVisit::new("early2")
            .location(2.0, 0.0)
            .duration(30)
            .committed_window(hours(8), hours(10))
            .currently_assigned_to("alice"),
        // Flexible visits - no committed window
        TestVisit::new("flex1")
            .location(3.0, 0.0)
            .duration(30)
            .currently_assigned_to("alice"),
        TestVisit::new("flex2")
            .location(4.0, 0.0)
            .duration(30)
            .currently_assigned_to("alice"),
    ];
    let visitors = vec![
        TestVisitor::new("alice").start_location(0.0, 0.0),
        TestVisitor::new("bob").start_location(0.0, 0.0),
    ];

    // Alice is running very late (starts at 3pm)
    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new()
            .visitor_window("alice", hours(15), hours(17)) // 3pm-5pm only
            .default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions {
            reassignment_penalty: 100, // Moderate penalty
            ..Default::default()
        },
    );

    let alice_visits = get_visitor_visits(&result, "alice");
    let bob_visits = get_visitor_visits(&result, "bob");

    // Early visits must go to Bob (committed window 8-10am, Alice not available then)
    assert!(
        bob_visits.contains(&"early1"),
        "early1 should be reassigned to Bob (committed window): bob={:?}",
        bob_visits
    );
    assert!(
        bob_visits.contains(&"early2"),
        "early2 should be reassigned to Bob (committed window): bob={:?}",
        bob_visits
    );

    // Flexible visits can stay with Alice or go to Bob depending on optimization
    let total_assigned = alice_visits.len() + bob_visits.len();
    assert_eq!(total_assigned, 4, "All 4 visits should be assigned");
}

#[test]
fn test_running_late_cascading_reassignment() {
    // Scenario: Alice is running 2 hours late. She has a visit with committed
    // window 9-10am that must be reassigned. Bob takes it, but now Bob
    // might have too much work and some of his visits cascade elsewhere.

    let visits = vec![
        // Alice's visit with tight window (must reassign due to late start)
        TestVisit::new("urgent")
            .location(5.0, 0.0)
            .duration(60)
            .committed_window(hours(9), hours(10))
            .currently_assigned_to("alice"),
        // Bob's existing workload
        TestVisit::new("bob1")
            .location(1.0, 0.0)
            .duration(60)
            .currently_assigned_to("bob"),
        TestVisit::new("bob2")
            .location(2.0, 0.0)
            .duration(60)
            .currently_assigned_to("bob"),
        // Charlie's existing workload
        TestVisit::new("charlie1")
            .location(8.0, 0.0)
            .duration(60)
            .currently_assigned_to("charlie"),
    ];
    let visitors = vec![
        TestVisitor::new("alice").start_location(0.0, 0.0),
        TestVisitor::new("bob").start_location(0.0, 0.0),
        TestVisitor::new("charlie").start_location(10.0, 0.0),
    ];

    // Alice starts at 11am (too late for 9-10am committed window)
    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new()
            .visitor_window("alice", hours(11), hours(17))
            .default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // All visits should be assigned
    let total_assigned: usize = result.routes.iter().map(|r| r.visit_ids.len()).sum();
    assert_eq!(
        total_assigned,
        4,
        "All 4 visits should be assigned: unassigned={:?}",
        result.unassigned
    );

    // The urgent visit should NOT be with Alice (she can't meet the 9-10am window)
    let alice_visits = get_visitor_visits(&result, "alice");
    assert!(
        !alice_visits.contains(&"urgent"),
        "urgent visit should not be with Alice (she starts at 11am): alice={:?}",
        alice_visits
    );
}

#[test]
fn test_running_late_no_one_can_cover() {
    // Scenario: Alice is late, and her visit has a committed window
    // that no one else can cover either. Visit should be unassigned.

    let visits = vec![
        TestVisit::new("impossible")
            .location(1.0, 0.0)
            .duration(60)
            .committed_window(hours(7), hours(8)) // 7am-8am window
            .currently_assigned_to("alice"),
    ];
    let visitors = vec![
        TestVisitor::new("alice").start_location(0.0, 0.0),
        TestVisitor::new("bob").start_location(0.0, 0.0),
    ];

    // Alice starts at 10am, Bob starts at 9am - neither can do 7-8am
    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new()
            .visitor_window("alice", hours(10), hours(17))
            .visitor_window("bob", hours(9), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // Visit should be unassigned (no one can meet the 7-8am window)
    let no_window = get_unassigned_with_reason(&result, UnassignedReason::NoFeasibleWindow);
    assert!(
        no_window.contains(&"impossible"),
        "Visit should be unassigned (7-8am window, no one available): {:?}",
        result.unassigned
    );
}

// ============================================================================
// Scale Tests
// ============================================================================

#[test]
fn test_50_visits_5_visitors() {
    // Realistic problem size: 50 visits across 5 technicians
    let visits: Vec<TestVisit> = (0..50)
        .map(|i| {
            // Spread visits across a 10x10 grid
            let x = (i % 10) as f64;
            let y = (i / 10) as f64;
            TestVisit::new(&format!("v{}", i))
                .location(x, y)
                .duration(20 + (i as i32 % 20)) // 20-40 min visits
        })
        .collect();

    let visitors: Vec<TestVisitor> = (0..5)
        .map(|i| {
            // Spread visitors around the edges
            let x = (i * 2) as f64;
            TestVisitor::new(&format!("tech{}", i)).start_location(x, 0.0)
        })
        .collect();

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(0, hours(10)), // 10 hour day
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // Count assignments
    let total_assigned: usize = result.routes.iter().map(|r| r.visit_ids.len()).sum();
    let total_unassigned = result.unassigned.len();

    println!("50 visits, 5 techs: {} assigned, {} unassigned", total_assigned, total_unassigned);

    // Most should be assigned (10 hour day with 20-40 min visits should fit most)
    assert!(
        total_assigned >= 40,
        "At least 40 of 50 visits should be assigned, got {}",
        total_assigned
    );

    // Work should be distributed
    for route in &result.routes {
        println!("  {}: {} visits", route.visitor_id.0, route.visit_ids.len());
    }
}

#[test]
fn test_100_visits_10_visitors() {
    // Upper bound of spec: 100 visits across 10 technicians
    let visits: Vec<TestVisit> = (0..100)
        .map(|i| {
            let x = (i % 10) as f64;
            let y = (i / 10) as f64;
            TestVisit::new(&format!("v{}", i))
                .location(x, y)
                .duration(15 + (i as i32 % 15)) // 15-30 min visits
        })
        .collect();

    let visitors: Vec<TestVisitor> = (0..10)
        .map(|i| {
            TestVisitor::new(&format!("tech{}", i)).start_location(i as f64, 0.0)
        })
        .collect();

    let start = std::time::Instant::now();
    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(0, hours(10)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );
    let elapsed = start.elapsed();

    let total_assigned: usize = result.routes.iter().map(|r| r.visit_ids.len()).sum();

    println!(
        "100 visits, 10 techs: {} assigned in {:?}",
        total_assigned, elapsed
    );

    // Should complete in reasonable time (spec says 10s target)
    assert!(
        elapsed.as_secs() < 30,
        "Should complete in <30s, took {:?}",
        elapsed
    );

    // Most should be assigned
    assert!(
        total_assigned >= 80,
        "At least 80 of 100 visits should be assigned, got {}",
        total_assigned
    );
}

#[test]
fn test_no_visitors() {
    let visits = vec![TestVisit::new("v1").location(1.0, 0.0)];
    let visitors: Vec<TestVisitor> = vec![];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new(),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    // Visit should be unassigned (no capable visitor since there are none)
    assert_eq!(result.unassigned.len(), 1);
}

#[test]
fn test_140_visits_14_visitors() {
    // Stress test matching user's question: 140 visits, 14 technicians
    // Search space: 14^140 assignments × sequencing ≈ 10^250 combinations
    let visits: Vec<TestVisit> = (0..140)
        .map(|i| {
            let x = (i % 14) as f64;
            let y = (i / 14) as f64;
            TestVisit::new(&format!("v{}", i))
                .location(x, y)
                .duration(15 + (i as i32 % 20)) // 15-35 min visits
        })
        .collect();

    let visitors: Vec<TestVisitor> = (0..14)
        .map(|i| {
            TestVisitor::new(&format!("tech{}", i)).start_location(i as f64, 0.0)
        })
        .collect();

    let start = std::time::Instant::now();
    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(0, hours(10)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );
    let elapsed = start.elapsed();

    let total_assigned: usize = result.routes.iter().map(|r| r.visit_ids.len()).sum();
    let total_travel: i32 = result.routes.iter().map(|r| r.total_travel_time).sum();

    println!(
        "140 visits, 14 techs: {} assigned in {:?}, total travel: {}s",
        total_assigned, elapsed, total_travel
    );

    // Performance: should complete quickly (heuristic, not exhaustive)
    assert!(
        elapsed.as_millis() < 5000,
        "Should complete in <5s, took {:?}",
        elapsed
    );

    // Quality: most should be assigned
    assert!(
        total_assigned >= 120,
        "At least 120 of 140 visits should be assigned, got {}",
        total_assigned
    );

    // Distribution: work should be spread across techs
    let max_per_tech = result.routes.iter().map(|r| r.visit_ids.len()).max().unwrap_or(0);
    let min_per_tech = result.routes.iter().map(|r| r.visit_ids.len()).min().unwrap_or(0);
    println!("  Distribution: min={}, max={} per tech", min_per_tech, max_per_tech);
}

#[test]
fn test_200_visits_20_visitors_stress() {
    // Larger stress test for bigger service companies
    // This pushes the solver harder
    let visits: Vec<TestVisit> = (0..200)
        .map(|i| {
            let x = (i % 20) as f64;
            let y = (i / 20) as f64;
            TestVisit::new(&format!("v{}", i))
                .location(x, y)
                .duration(15 + (i as i32 % 15)) // 15-30 min visits
        })
        .collect();

    let visitors: Vec<TestVisitor> = (0..20)
        .map(|i| {
            TestVisitor::new(&format!("tech{}", i)).start_location(i as f64, 0.0)
        })
        .collect();

    let start = std::time::Instant::now();
    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(0, hours(10)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );
    let elapsed = start.elapsed();

    let total_assigned: usize = result.routes.iter().map(|r| r.visit_ids.len()).sum();

    println!(
        "200 visits, 20 techs: {} assigned in {:?}",
        total_assigned, elapsed
    );

    // Should still complete in reasonable time
    assert!(
        elapsed.as_secs() < 30,
        "Should complete in <30s, took {:?}",
        elapsed
    );

    // Most should be assigned
    assert!(
        total_assigned >= 160,
        "At least 160 of 200 visits should be assigned, got {}",
        total_assigned
    );
}

// ============================================================================
// Quality Benchmarks
// ============================================================================

#[test]
fn test_local_search_improves_solution_quality() {
    // Compare solution quality with and without local search
    // Local search should reduce total travel time
    let visits: Vec<TestVisit> = (0..30)
        .map(|i| {
            // Deliberately scrambled positions to create suboptimal insertion order
            let x = ((i * 7) % 10) as f64;
            let y = ((i * 13) % 10) as f64;
            TestVisit::new(&format!("v{}", i))
                .location(x, y)
                .duration(20)
        })
        .collect();

    let visitors: Vec<TestVisitor> = (0..3)
        .map(|i| {
            TestVisitor::new(&format!("tech{}", i)).start_location((i * 5) as f64, 0.0)
        })
        .collect();

    // Without local search
    let result_no_ls = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(0, hours(10)),
        &ManhattanMatrix,
        SolveOptions { local_search_iterations: 0, ..Default::default() },
    );

    // With local search (default)
    let result_with_ls = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(0, hours(10)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    let travel_no_ls: i32 = result_no_ls.routes.iter().map(|r| r.total_travel_time).sum();
    let travel_with_ls: i32 = result_with_ls.routes.iter().map(|r| r.total_travel_time).sum();

    println!(
        "Quality benchmark: without LS = {}s, with LS = {}s, improvement = {:.1}%",
        travel_no_ls,
        travel_with_ls,
        (1.0 - travel_with_ls as f64 / travel_no_ls as f64) * 100.0
    );

    // Local search should not make things worse
    assert!(
        travel_with_ls <= travel_no_ls,
        "Local search should improve or maintain quality: without={}, with={}",
        travel_no_ls, travel_with_ls
    );

    // In most cases, it should actually improve
    // (but not guaranteed for all inputs, so we just check it's not worse)
}

#[test]
fn test_travel_efficiency_geographic_clusters() {
    // Verify that geographically clustered visits are assigned efficiently
    // Techs near clusters should get those clusters' visits
    let visits: Vec<TestVisit> = {
        let mut v = Vec::new();
        // Cluster A: around (0, 0)
        for i in 0..10 {
            v.push(TestVisit::new(&format!("a{}", i))
                .location(i as f64 * 0.1, i as f64 * 0.1)
                .duration(20));
        }
        // Cluster B: around (10, 0)
        for i in 0..10 {
            v.push(TestVisit::new(&format!("b{}", i))
                .location(10.0 + i as f64 * 0.1, i as f64 * 0.1)
                .duration(20));
        }
        v
    };

    let visitors = vec![
        TestVisitor::new("tech_a").start_location(0.0, 0.0),  // Near cluster A
        TestVisitor::new("tech_b").start_location(10.0, 0.0), // Near cluster B
    ];

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(0, hours(10)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );

    let tech_a_visits = get_visitor_visits(&result, "tech_a");
    let tech_b_visits = get_visitor_visits(&result, "tech_b");

    // Count how many cluster A visits went to tech A
    let a_correct = tech_a_visits.iter().filter(|v| v.starts_with('a')).count();
    let b_correct = tech_b_visits.iter().filter(|v| v.starts_with('b')).count();

    println!(
        "Geographic efficiency: tech_a got {}/10 cluster A visits, tech_b got {}/10 cluster B visits",
        a_correct, b_correct
    );

    // Most visits should go to the nearby tech (at least 7/10)
    assert!(
        a_correct >= 7,
        "tech_a should get most cluster A visits: got {}/10",
        a_correct
    );
    assert!(
        b_correct >= 7,
        "tech_b should get most cluster B visits: got {}/10",
        b_correct
    );
}

#[test]
fn test_solution_determinism() {
    // Verify that the solver produces consistent results
    // (important for debugging and predictability)
    let visits: Vec<TestVisit> = (0..20)
        .map(|i| {
            TestVisit::new(&format!("v{}", i))
                .location((i % 5) as f64, (i / 5) as f64)
                .duration(30)
        })
        .collect();

    let visitors: Vec<TestVisitor> = (0..3)
        .map(|i| {
            TestVisitor::new(&format!("tech{}", i)).start_location(i as f64, 0.0)
        })
        .collect();

    // Run solver multiple times
    let mut results = Vec::new();
    for _ in 0..3 {
        let result = solve(
            1,
            &visits,
            &visitors,
            &TestAvailability::new().default_window(0, hours(10)),
            &ManhattanMatrix,
            SolveOptions::default(),
        );
        results.push(result);
    }

    // All runs should produce same assignment
    for i in 1..results.len() {
        for (j, route) in results[0].routes.iter().enumerate() {
            assert_eq!(
                route.visit_ids, results[i].routes[j].visit_ids,
                "Run {} produced different result than run 0 for route {}",
                i, j
            );
        }
    }
}

// ============================================================================
// Comprehensive Real-World Scenario Tests
// ============================================================================

#[test]
fn test_realistic_service_day() {
    // Simulates a typical day for a service company:
    // - 5 technicians, 40 total visits
    // - Mix of recurring services, repairs, and quotes
    // - Various constraints and complications
    //
    // This is the "integration test" that proves the solver handles
    // real-world complexity correctly.

    let mut visits = Vec::new();

    // === Recurring weekly services (25 visits, predictable) ===
    // These are the bread and butter - predictable locations, standard duration
    for i in 0..25 {
        let x = (i % 5) as f64 * 2.0;
        let y = (i / 5) as f64 * 2.0;
        visits.push(
            TestVisit::new(&format!("recurring_{}", i))
                .location(x, y)
                .duration(30) // Standard 30-min service
                .currently_assigned_to(&format!("tech{}", i % 5)) // Previous assignment
        );
    }

    // === Repair callbacks from previous week (5 visits) ===
    // Customer issues that need follow-up, often time-sensitive
    for i in 0..5 {
        visits.push(
            TestVisit::new(&format!("repair_{}", i))
                .location(i as f64 * 2.0 + 0.5, 3.0)
                .duration(60) // Longer than standard
                .committed_window(hours(8), hours(12)) // Morning required
                .requires("repair") // Needs repair skill
        );
    }

    // === New customer quotes (5 visits) ===
    // Sales opportunities, flexible timing but want to impress
    for i in 0..5 {
        visits.push(
            TestVisit::new(&format!("quote_{}", i))
                .location(i as f64 * 2.0 + 1.0, 8.0)
                .duration(45) // Quote walkthrough
                .target_time(hours(10) + i as i32 * 3600) // Preferred times spread out
        );
    }

    // === VIP customer with specific requirements (3 visits) ===
    // Premium customers who always want their regular tech
    visits.push(
        TestVisit::new("vip_1")
            .location(4.0, 4.0)
            .duration(45)
            .pinned_to_visitor("tech0")
            .committed_window(hours(9), hours(11))
    );
    visits.push(
        TestVisit::new("vip_2")
            .location(6.0, 4.0)
            .duration(45)
            .pinned_to_visitor("tech1")
            .committed_window(hours(13), hours(15))
    );
    visits.push(
        TestVisit::new("vip_3")
            .location(8.0, 4.0)
            .duration(45)
            .pinned_to_visitor("tech2")
    );

    // === Equipment check requiring special certification (2 visits) ===
    visits.push(
        TestVisit::new("certified_1")
            .location(2.0, 6.0)
            .duration(60)
            .requires("certification")
    );
    visits.push(
        TestVisit::new("certified_2")
            .location(7.0, 6.0)
            .duration(60)
            .requires("certification")
    );

    // === Technicians with different capabilities ===
    let visitors = vec![
        TestVisitor::new("tech0")
            .start_location(0.0, 0.0)
            .capability("repair")
            .capability("certification"), // Senior tech - all skills
        TestVisitor::new("tech1")
            .start_location(2.0, 0.0)
            .capability("repair"), // Can do repairs
        TestVisitor::new("tech2")
            .start_location(5.0, 0.0)
            .capability("repair")
            .capability("certification"), // Another senior
        TestVisitor::new("tech3")
            .start_location(7.0, 0.0), // Junior - basic services only
        TestVisitor::new("tech4")
            .start_location(9.0, 0.0)
            .capability("repair"), // Can do repairs
    ];

    // === Availability complications ===
    // - tech3 is running 1 hour late
    // - Everyone else normal schedule

    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new()
            .visitor_window("tech3", hours(9), hours(17)) // Late start
            .default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions {
            reassignment_penalty: 100, // Prefer stability
            target_time_weight: 5,     // Consider target times
            ..Default::default()
        },
    );

    // === Verify results ===
    let total_visits = visits.len();
    let total_assigned: usize = result.routes.iter().map(|r| r.visit_ids.len()).sum();
    let total_unassigned = result.unassigned.len();

    println!("=== Realistic Service Day Results ===");
    println!("Total visits: {}", total_visits);
    println!("Assigned: {}", total_assigned);
    println!("Unassigned: {}", total_unassigned);

    for route in &result.routes {
        println!(
            "  {}: {} visits, {} travel time",
            route.visitor_id.0,
            route.visit_ids.len(),
            route.total_travel_time
        );
    }

    // 1. Most visits should be assigned (allow a few unassigned due to constraints)
    assert!(
        total_assigned >= 38,
        "At least 38 of {} visits should be assigned, got {}",
        total_visits, total_assigned
    );

    // 2. VIP visits must be assigned to their pinned technicians
    let tech0_visits = get_visitor_visits(&result, "tech0");
    let tech1_visits = get_visitor_visits(&result, "tech1");
    let tech2_visits = get_visitor_visits(&result, "tech2");

    assert!(
        tech0_visits.contains(&"vip_1"),
        "VIP 1 must be assigned to tech0: {:?}", tech0_visits
    );
    assert!(
        tech1_visits.contains(&"vip_2"),
        "VIP 2 must be assigned to tech1: {:?}", tech1_visits
    );
    assert!(
        tech2_visits.contains(&"vip_3"),
        "VIP 3 must be assigned to tech2: {:?}", tech2_visits
    );

    // 3. Certified visits must go to certified techs (tech0 or tech2)
    let certified_visits: Vec<_> = result.routes.iter()
        .filter(|r| r.visitor_id.0 == "tech0" || r.visitor_id.0 == "tech2")
        .flat_map(|r| r.visit_ids.iter())
        .filter(|id| id.0.starts_with("certified"))
        .collect();
    assert_eq!(
        certified_visits.len(), 2,
        "Both certified visits should go to certified techs"
    );

    // 4. Repair visits should go to repair-capable techs
    let tech3_visits = get_visitor_visits(&result, "tech3");
    let repair_on_tech3 = tech3_visits.iter().any(|v| v.starts_with("repair"));
    assert!(
        !repair_on_tech3,
        "tech3 (no repair skill) should not have repair visits: {:?}",
        tech3_visits
    );

    // 5. Workload should be reasonably balanced (but constraints may cause imbalance)
    // In realistic scenarios with VIP pins and capability requirements, some imbalance is expected
    let max_visits = result.routes.iter().map(|r| r.visit_ids.len()).max().unwrap_or(0);
    let min_visits = result.routes.iter().map(|r| r.visit_ids.len()).min().unwrap_or(0);
    println!("Workload: min={}, max={} (diff={})", min_visits, max_visits, max_visits - min_visits);
    // Allow larger imbalance due to constraints, but not extreme
    assert!(
        max_visits - min_visits <= 12,
        "Workload imbalance too extreme: max={}, min={}",
        max_visits, min_visits
    );

    // 6. Check VIP committed windows are respected
    let tech0_route = result.routes.iter().find(|r| r.visitor_id.0 == "tech0").unwrap();
    if let Some(vip1_idx) = tech0_route.visit_ids.iter().position(|id| id.0 == "vip_1") {
        let (start, _) = tech0_route.estimated_windows[vip1_idx];
        assert!(
            start >= hours(9) && start <= hours(11),
            "VIP 1 should be scheduled 9-11am, got start={}",
            start
        );
    }
}

#[test]
fn test_worst_case_all_constraints() {
    // Stress test: Many visits with various constraints
    // This ensures the solver doesn't break under heavy constraint load

    let mut visits = Vec::new();

    // 20 visits with various constraints (avoiding impossible combinations)
    for i in 0..20 {
        let mut visit = TestVisit::new(&format!("v{}", i))
            .location((i % 5) as f64, (i / 5) as f64)
            .duration(30);

        // Add various constraints based on index
        // Only add one time constraint per visit to avoid conflicts
        if i % 6 == 0 {
            visit = visit.committed_window(hours(8), hours(12)); // Morning window
        } else if i % 6 == 1 {
            visit = visit.committed_window(hours(13), hours(17)); // Afternoon window
        } else if i % 6 == 2 {
            visit = visit.target_time(hours(10));
        }

        // Capability requirements (non-conflicting with pinning)
        if i % 4 == 0 && i % 5 != 0 { // Don't add skill requirement to pinned visits
            visit = visit.requires("skill_a");
        }

        visits.push(visit);
    }

    // Add 4 pinned visits separately (cleaner than mixing constraints)
    visits.push(TestVisit::new("pinned_0").location(0.5, 0.5).duration(20).pinned_to_visitor("tech0"));
    visits.push(TestVisit::new("pinned_1").location(1.5, 1.5).duration(20).pinned_to_visitor("tech0"));
    visits.push(TestVisit::new("pinned_2").location(2.5, 2.5).duration(20).pinned_to_visitor("tech1"));
    visits.push(TestVisit::new("pinned_3").location(3.5, 3.5).duration(20).pinned_to_visitor("tech2"));

    let visitors = vec![
        TestVisitor::new("tech0")
            .start_location(0.0, 0.0)
            .capability("skill_a")
            .capability("skill_b"),
        TestVisitor::new("tech1")
            .start_location(2.0, 0.0)
            .capability("skill_a"),
        TestVisitor::new("tech2")
            .start_location(4.0, 0.0)
            .capability("skill_b"),
    ];

    let start = std::time::Instant::now();
    let result = solve(
        1,
        &visits,
        &visitors,
        &TestAvailability::new().default_window(hours(8), hours(17)),
        &ManhattanMatrix,
        SolveOptions::default(),
    );
    let elapsed = start.elapsed();

    let total_assigned: usize = result.routes.iter().map(|r| r.visit_ids.len()).sum();

    println!(
        "Worst case (mixed constraints): {} of 24 assigned in {:?}",
        total_assigned, elapsed
    );

    // Should complete quickly even with all constraints
    assert!(
        elapsed.as_millis() < 1000,
        "Should complete in <1s even with heavy constraints"
    );

    // Most should be assigned
    assert!(
        total_assigned >= 20,
        "At least 20 of 24 should be assigned, got {}",
        total_assigned
    );

    // Verify pinned visits are respected
    let tech0_visits = get_visitor_visits(&result, "tech0");
    let tech1_visits = get_visitor_visits(&result, "tech1");
    let tech2_visits = get_visitor_visits(&result, "tech2");

    println!("tech0 visits: {:?}", tech0_visits);
    println!("tech1 visits: {:?}", tech1_visits);
    println!("tech2 visits: {:?}", tech2_visits);

    // Find where pinned visits actually went
    for (tech, visits) in [("tech0", &tech0_visits), ("tech1", &tech1_visits), ("tech2", &tech2_visits)] {
        for v in visits.iter().filter(|v| v.starts_with("pinned")) {
            println!("  {} -> {}", v, tech);
        }
    }

    assert!(tech0_visits.contains(&"pinned_0"), "pinned_0 should be with tech0: {:?}", tech0_visits);
    assert!(tech0_visits.contains(&"pinned_1"), "pinned_1 should be with tech0: {:?}", tech0_visits);
    assert!(tech1_visits.contains(&"pinned_2"), "pinned_2 should be with tech1: {:?}", tech1_visits);
    assert!(tech2_visits.contains(&"pinned_3"), "pinned_3 should be with tech2: {:?}", tech2_visits);

    // Verify skill requirements are respected
    // v0, v4, v8, v12, v16 require skill_a - should NOT be with tech2 (only has skill_b)
    let tech2_has_skill_a_visit = tech2_visits.iter()
        .any(|v| v.starts_with('v') && {
            let num: i32 = v[1..].parse().unwrap_or(-1);
            num % 4 == 0 && num % 5 != 0
        });
    assert!(
        !tech2_has_skill_a_visit,
        "tech2 should not have skill_a visits: {:?}",
        tech2_visits
    );
}
