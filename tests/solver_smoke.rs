use std::collections::HashMap;

use vrp_planner::solver::{solve, SolveOptions};
use vrp_planner::traits::{AvailabilityProvider, DistanceMatrixProvider, Visit, VisitPinType, Visitor};

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct Id(&'static str);

#[derive(Clone, Debug)]
struct MockVisit {
    id: Id,
    location: (f64, f64),
    duration_min: i32,
    pin_type: VisitPinType,
    pinned_visitor: Option<Id>,
}

impl Visit for MockVisit {
    type Id = Id;
    type VisitorId = Id;

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
        None
    }

    fn target_time(&self) -> Option<i32> {
        None
    }

    fn pin_type(&self) -> VisitPinType {
        self.pin_type
    }

    fn pinned_visitor(&self) -> Option<&Self::VisitorId> {
        self.pinned_visitor.as_ref()
    }

    fn pinned_date(&self) -> Option<i64> {
        None
    }

    fn required_capabilities(&self) -> &[String] {
        &[]
    }

    fn location(&self) -> (f64, f64) {
        self.location
    }
}

#[derive(Clone, Debug)]
struct MockVisitor {
    id: Id,
}

impl Visitor for MockVisitor {
    type Id = Id;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn start_location(&self) -> Option<(f64, f64)> {
        Some((0.0, 0.0))
    }

    fn end_location(&self) -> Option<(f64, f64)> {
        None
    }

    fn capabilities(&self) -> &[String] {
        &[]
    }
}

struct MockAvailability;

impl AvailabilityProvider for MockAvailability {
    type VisitorId = Id;

    fn availability_for(&self, _visitor_id: &Self::VisitorId, _date: i64) -> Option<(i32, i32)> {
        Some((0, 8 * 3600))
    }
}

struct MockMatrix;

impl DistanceMatrixProvider for MockMatrix {
    fn matrix_for(&self, locations: &[(f64, f64)]) -> Vec<Vec<i32>> {
        let mut matrix = vec![vec![0; locations.len()]; locations.len()];
        for (i, from) in locations.iter().enumerate() {
            for (j, to) in locations.iter().enumerate() {
                let dist = (from.0 - to.0).abs() + (from.1 - to.1).abs();
                matrix[i][j] = (dist * 60.0) as i32;
            }
        }
        matrix
    }
}

#[test]
fn honors_pinned_visitor() {
    let visits = vec![
        MockVisit {
            id: Id("v1"),
            location: (1.0, 0.0),
            duration_min: 30,
            pin_type: VisitPinType::Visitor,
            pinned_visitor: Some(Id("a")),
        },
        MockVisit {
            id: Id("v2"),
            location: (2.0, 0.0),
            duration_min: 30,
            pin_type: VisitPinType::None,
            pinned_visitor: None,
        },
    ];

    let visitors = vec![MockVisitor { id: Id("a") }, MockVisitor { id: Id("b") }];

    let result = solve(1, &visits, &visitors, &MockAvailability, &MockMatrix, SolveOptions::default());

    let mut assigned: HashMap<&str, Vec<&str>> = HashMap::new();
    for route in result.routes {
        let ids = route.visit_ids.iter().map(|id| id.0).collect::<Vec<_>>();
        assigned.insert(route.visitor_id.0, ids);
    }

    let a_route = assigned.get("a").cloned().unwrap_or_default();
    assert!(a_route.contains(&"v1"));
}
