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

    fn availability_for(&self, visitor_id: &Self::VisitorId, _date: i64) -> Option<(i32, i32)> {
        if let Some(override_window) = self.overrides.get(&visitor_id.0) {
            *override_window
        } else {
            Some(self.default_window)
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
        SolveOptions { target_time_weight: 10 }, // Higher weight should influence sequencing more
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
