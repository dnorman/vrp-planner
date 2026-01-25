//! Routing planner solver (baseline implementation).

use std::collections::HashMap;

use crate::traits::{AvailabilityProvider, DistanceMatrixProvider, Visit, VisitPinType, Visitor};

#[derive(Debug, Clone)]
pub struct SolveOptions {
    pub target_time_weight: i32,
}

impl Default for SolveOptions {
    fn default() -> Self {
        Self { target_time_weight: 1 }
    }
}

#[derive(Debug, Clone)]
pub struct RouteResult<VisitorId, VisitId> {
    pub visitor_id: VisitorId,
    pub visit_ids: Vec<VisitId>,
    pub estimated_windows: Vec<(i32, i32)>,
    pub total_travel_time: i32,
}

#[derive(Debug, Clone)]
pub struct PlannerResult<VisitorId, VisitId> {
    pub routes: Vec<RouteResult<VisitorId, VisitId>>,
    pub unassigned: Vec<VisitId>,
}

#[derive(Debug, Clone)]
struct RouteState<'a, V: Visit, R: Visitor<Id = V::VisitorId>> {
    visitor: &'a R,
    visits: Vec<&'a V>,
    estimated_windows: Vec<(i32, i32)>,
    total_travel_time: i32,
}

pub fn solve<'a, V, R, A, M>(
    service_date: i64,
    visits: &'a [V],
    visitors: &'a [R],
    availability: &A,
    matrix_provider: &M,
    options: SolveOptions,
) -> PlannerResult<V::VisitorId, V::Id>
where
    V: Visit,
    R: Visitor<Id = V::VisitorId>,
    A: AvailabilityProvider<VisitorId = V::VisitorId>,
    M: DistanceMatrixProvider,
{
    let mut unassigned: Vec<&V> = Vec::new();
    let mut pinned_assignments: HashMap<&V::VisitorId, Vec<&V>> = HashMap::new();

    for visit in visits {
        if let Some(date) = visit.pinned_date() {
            if date != service_date {
                unassigned.push(visit);
                continue;
            }
        }

        match visit.pin_type() {
            VisitPinType::Visitor | VisitPinType::VisitorAndDate => {
                if let Some(visitor_id) = visit.pinned_visitor() {
                    pinned_assignments.entry(visitor_id).or_default().push(visit);
                } else {
                    unassigned.push(visit);
                }
            }
            VisitPinType::Date | VisitPinType::None => {
                unassigned.push(visit);
            }
        }
    }

    let locations = collect_locations(visits, visitors);
    let index = location_index(&locations);
    let matrix = matrix_provider.matrix_for(&locations);

    let mut routes: Vec<RouteState<'a, V, R>> = Vec::new();
    for visitor in visitors {
        let pinned = pinned_assignments
            .get(visitor.id())
            .cloned()
            .unwrap_or_default();

        let mut route = RouteState {
            visitor,
            visits: pinned,
            estimated_windows: Vec::new(),
            total_travel_time: 0,
        };

        if !route.visits.is_empty() {
            if let Some(schedule) = compute_schedule(service_date, &route, availability, &matrix, &index, options.target_time_weight) {
                route.estimated_windows = schedule.0;
                route.total_travel_time = schedule.1;
            } else {
                unassigned.extend(route.visits.drain(..));
            }
        }

        routes.push(route);
    }

    for visit in unassigned.clone() {
        if !visit_is_compatible(visit, visitors) {
            continue;
        }

        let mut best_route: Option<usize> = None;
        let mut best_position: usize = 0;
        let mut best_cost: i32 = i32::MAX;
        let mut best_schedule: Option<(Vec<(i32, i32)>, i32)> = None;

        for (route_index, route) in routes.iter().enumerate() {
            for position in 0..=route.visits.len() {
                let mut candidate = route.visits.clone();
                candidate.insert(position, visit);

                let candidate_route = RouteState {
                    visitor: route.visitor,
                    visits: candidate,
                    estimated_windows: Vec::new(),
                    total_travel_time: 0,
                };

                if let Some(schedule) = compute_schedule(
                    service_date,
                    &candidate_route,
                    availability,
                    &matrix,
                    &index,
                    options.target_time_weight,
                ) {
                    if schedule.1 < best_cost {
                        best_cost = schedule.1;
                        best_route = Some(route_index);
                        best_position = position;
                        best_schedule = Some(schedule);
                    }
                }
            }
        }

        if let Some(route_index) = best_route {
            let route = &mut routes[route_index];
            route.visits.insert(best_position, visit);
            if let Some((windows, cost)) = best_schedule {
                route.estimated_windows = windows;
                route.total_travel_time = cost;
            }
        }
    }

    let mut assigned_ids: Vec<V::Id> = Vec::new();
    for route in &routes {
        for visit in &route.visits {
            assigned_ids.push(visit.id().clone());
        }
    }

    let mut unassigned_ids: Vec<V::Id> = Vec::new();
    for visit in visits {
        if !assigned_ids.contains(visit.id()) {
            unassigned_ids.push(visit.id().clone());
        }
    }

    let routes = routes
        .into_iter()
        .map(|route| RouteResult {
            visitor_id: route.visitor.id().clone(),
            visit_ids: route.visits.iter().map(|visit| visit.id().clone()).collect(),
            estimated_windows: route.estimated_windows,
            total_travel_time: route.total_travel_time,
        })
        .collect();

    PlannerResult {
        routes,
        unassigned: unassigned_ids,
    }
}

fn visit_is_compatible<V, R>(visit: &V, visitors: &[R]) -> bool
where
    V: Visit,
    R: Visitor<Id = V::VisitorId>,
{
    visitors.iter().any(|visitor| {
        let required = visit.required_capabilities();
        if required.is_empty() {
            return true;
        }
        let available = visitor.capabilities();
        required.iter().all(|cap| available.contains(cap))
    })
}

fn compute_schedule<V, R, A>(
    service_date: i64,
    route: &RouteState<'_, V, R>,
    availability: &A,
    matrix: &[Vec<i32>],
    index: &HashMap<String, usize>,
    target_weight: i32,
) -> Option<(Vec<(i32, i32)>, i32)>
where
    V: Visit,
    R: Visitor<Id = V::VisitorId>,
    A: AvailabilityProvider<VisitorId = V::VisitorId>,
{
    let availability_window = availability.availability_for(route.visitor.id(), service_date)?;
    let mut time = availability_window.0;
    let mut total_travel = 0;
    let mut windows = Vec::with_capacity(route.visits.len());

    let mut prev_location = route
        .visitor
        .start_location()
        .unwrap_or((0.0, 0.0));

    for visit in &route.visits {
        let travel = travel_time(prev_location, visit.location(), matrix, index);
        time += travel;
        total_travel += travel;

        if let Some((window_start, window_end)) = visit.committed_window() {
            if time < window_start {
                time = window_start;
            }
            if time > window_end {
                return None;
            }
        }

        let start_time = time;
        let duration_secs = visit.estimated_duration_minutes() * 60;
        time += duration_secs;

        if time > availability_window.1 {
            return None;
        }

        if let Some(target) = visit.target_time() {
            total_travel += (start_time - target).abs() * target_weight;
        }

        windows.push((start_time, start_time + duration_secs));
        prev_location = visit.location();
    }

    Some((windows, total_travel))
}

fn travel_time(
    from: (f64, f64),
    to: (f64, f64),
    matrix: &[Vec<i32>],
    index: &HashMap<String, usize>,
) -> i32 {
    let from_idx = index[&location_key(from)];
    let to_idx = index[&location_key(to)];
    matrix[from_idx][to_idx]
}

fn collect_locations<V, R>(visits: &[V], visitors: &[R]) -> Vec<(f64, f64)>
where
    V: Visit,
    R: Visitor<Id = V::VisitorId>,
{
    let mut locations = Vec::new();
    for visitor in visitors {
        if let Some(start) = visitor.start_location() {
            locations.push(start);
        }
        if let Some(end) = visitor.end_location() {
            locations.push(end);
        }
    }
    for visit in visits {
        locations.push(visit.location());
    }

    dedupe_locations(locations)
}

fn dedupe_locations(locations: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut unique = Vec::new();
    for location in locations {
        let key = location_key(location);
        if seen.contains_key(&key) {
            continue;
        }
        seen.insert(key, unique.len());
        unique.push(location);
    }
    unique
}

fn location_key(location: (f64, f64)) -> String {
    format!("{:.6},{:.6}", location.0, location.1)
}

fn location_index(locations: &[(f64, f64)]) -> HashMap<String, usize> {
    let mut index = HashMap::new();
    for (i, location) in locations.iter().enumerate() {
        index.insert(location_key(*location), i);
    }
    index
}
