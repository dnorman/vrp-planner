//! Routing planner solver (baseline implementation).

use std::collections::HashMap;

use crate::traits::{AvailabilityProvider, DistanceMatrixProvider, UnassignedReason, Visit, VisitPinType, Visitor};

#[derive(Debug, Clone)]
pub struct SolveOptions {
    /// Weight for target time deviation penalty (per second).
    pub target_time_weight: i32,
    /// Weight for reassigning a visit to a different visitor (stability penalty).
    pub reassignment_penalty: i32,
    /// Maximum iterations for local search improvement.
    pub local_search_iterations: usize,
}

impl Default for SolveOptions {
    fn default() -> Self {
        Self {
            target_time_weight: 1,
            reassignment_penalty: 300, // ~5 minutes equivalent
            local_search_iterations: 100,
        }
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
pub struct UnassignedVisit<VisitId> {
    pub visit_id: VisitId,
    pub reason: UnassignedReason,
}

#[derive(Debug, Clone)]
pub struct PlannerResult<VisitorId, VisitId> {
    pub routes: Vec<RouteResult<VisitorId, VisitId>>,
    pub unassigned: Vec<UnassignedVisit<VisitId>>,
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
    let mut to_assign: Vec<&V> = Vec::new();
    let mut unassigned_with_reason: Vec<(&V, UnassignedReason)> = Vec::new();
    let mut pinned_assignments: HashMap<&V::VisitorId, Vec<&V>> = HashMap::new();

    for visit in visits {
        if let Some(date) = visit.pinned_date() {
            if date != service_date {
                unassigned_with_reason.push((visit, UnassignedReason::WrongDate));
                continue;
            }
        }

        match visit.pin_type() {
            VisitPinType::Visitor | VisitPinType::VisitorAndDate => {
                if let Some(visitor_id) = visit.pinned_visitor() {
                    pinned_assignments.entry(visitor_id).or_default().push(visit);
                } else {
                    unassigned_with_reason.push((visit, UnassignedReason::MissingPinnedVisitor));
                }
            }
            VisitPinType::Date | VisitPinType::None => {
                to_assign.push(visit);
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
            if let Some(schedule) = compute_schedule(service_date, &route, availability, &matrix, &index, &options) {
                route.estimated_windows = schedule.0;
                route.total_travel_time = schedule.1;
            } else {
                for visit in route.visits.drain(..) {
                    unassigned_with_reason.push((visit, UnassignedReason::NoFeasibleWindow));
                }
            }
        }

        routes.push(route);
    }

    for visit in to_assign {
        if !visit_is_compatible(visit, visitors) {
            unassigned_with_reason.push((visit, UnassignedReason::NoCapableVisitor));
            continue;
        }

        let mut best_route: Option<usize> = None;
        let mut best_position: usize = 0;
        let mut best_cost: i32 = i32::MAX;
        let mut best_schedule: Option<(Vec<(i32, i32)>, i32)> = None;
        let mut found_capable_available_visitor = false;

        for (route_index, route) in routes.iter().enumerate() {
            // Skip visitors who don't have required capabilities
            if !visitor_can_do(visit, route.visitor) {
                continue;
            }

            // Check if this capable visitor is available
            if availability.availability_for(route.visitor.id(), service_date).is_some() {
                found_capable_available_visitor = true;
            }

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
                    &options,
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
        } else {
            // Determine the reason: no capable available visitor, or no feasible window
            let reason = if found_capable_available_visitor {
                UnassignedReason::NoFeasibleWindow
            } else {
                UnassignedReason::NoCapableVisitor
            };
            unassigned_with_reason.push((visit, reason));
        }
    }

    // Local search improvement phase
    local_search(
        &mut routes,
        service_date,
        availability,
        &matrix,
        &index,
        &options,
    );

    let routes = routes
        .into_iter()
        .map(|route| RouteResult {
            visitor_id: route.visitor.id().clone(),
            visit_ids: route.visits.iter().map(|visit| visit.id().clone()).collect(),
            estimated_windows: route.estimated_windows,
            total_travel_time: route.total_travel_time,
        })
        .collect();

    let unassigned = unassigned_with_reason
        .into_iter()
        .map(|(visit, reason)| UnassignedVisit {
            visit_id: visit.id().clone(),
            reason,
        })
        .collect();

    PlannerResult { routes, unassigned }
}

/// Check if a visitor has all required capabilities for a visit.
fn visitor_can_do<V, R>(visit: &V, visitor: &R) -> bool
where
    V: Visit,
    R: Visitor<Id = V::VisitorId>,
{
    let required = visit.required_capabilities();
    if required.is_empty() {
        return true;
    }
    let available = visitor.capabilities();
    required.iter().all(|cap| available.contains(cap))
}

/// Check if any visitor in the list can handle this visit.
fn visit_is_compatible<V, R>(visit: &V, visitors: &[R]) -> bool
where
    V: Visit,
    R: Visitor<Id = V::VisitorId>,
{
    visitors.iter().any(|visitor| visitor_can_do(visit, visitor))
}

fn compute_schedule<V, R, A>(
    service_date: i64,
    route: &RouteState<'_, V, R>,
    availability: &A,
    matrix: &[Vec<i32>],
    index: &HashMap<String, usize>,
    options: &SolveOptions,
) -> Option<(Vec<(i32, i32)>, i32)>
where
    V: Visit,
    R: Visitor<Id = V::VisitorId>,
    A: AvailabilityProvider<VisitorId = V::VisitorId>,
{
    let availability_windows = availability.availability_for(route.visitor.id(), service_date)?;
    if availability_windows.is_empty() {
        return None;
    }

    // Start at the beginning of the first availability window
    let mut time = availability_windows[0].0;
    let mut current_window_idx = 0;
    let mut total_cost = 0;
    let mut result_windows = Vec::with_capacity(route.visits.len());

    let mut prev_location = route
        .visitor
        .start_location()
        .unwrap_or((0.0, 0.0));

    for visit in &route.visits {
        let travel = travel_time(prev_location, visit.location(), matrix, index);
        time += travel;
        total_cost += travel;

        let duration_secs = visit.estimated_duration_minutes() * 60;

        // Handle committed window constraints
        if let Some((committed_start, committed_end)) = visit.committed_window() {
            if time < committed_start {
                time = committed_start;
            }
            if time > committed_end {
                return None;
            }
        }

        // Find a window where the visit fits entirely
        let (start_time, window_idx) = find_fitting_window(
            time,
            duration_secs,
            current_window_idx,
            &availability_windows,
            visit.committed_window(),
        )?;

        time = start_time + duration_secs;
        current_window_idx = window_idx;

        // Target time penalty
        if let Some(target) = visit.target_time() {
            total_cost += (start_time - target).abs() * options.target_time_weight;
        }

        // Stability penalty: penalize reassigning to a different visitor
        if let Some(current_visitor) = visit.current_visitor_id() {
            if current_visitor != route.visitor.id() {
                total_cost += options.reassignment_penalty;
            }
        }

        result_windows.push((start_time, start_time + duration_secs));
        prev_location = visit.location();
    }

    Some((result_windows, total_cost))
}

/// Find the earliest window where a visit can fit entirely.
///
/// Returns the start time and window index if found.
fn find_fitting_window(
    earliest_start: i32,
    duration: i32,
    current_window_idx: usize,
    windows: &[(i32, i32)],
    committed_window: Option<(i32, i32)>,
) -> Option<(i32, usize)> {
    for (idx, &(window_start, window_end)) in windows.iter().enumerate().skip(current_window_idx) {
        // Determine the earliest we can start in this window
        let start_in_window = earliest_start.max(window_start);

        // Check committed window constraints
        if let Some((committed_start, committed_end)) = committed_window {
            // If committed window ends before this availability window starts, no fit
            if committed_end < window_start {
                return None;
            }
            // If committed window starts after this availability window ends, try next
            if committed_start > window_end {
                continue;
            }
            // Adjust start time for committed window
            let adjusted_start = start_in_window.max(committed_start);
            let end_time = adjusted_start + duration;

            // Check if it fits in both the availability window and committed window
            if end_time <= window_end && adjusted_start <= committed_end && end_time <= committed_end {
                return Some((adjusted_start, idx));
            }
        } else {
            // No committed window, just check availability
            let end_time = start_in_window + duration;
            if end_time <= window_end {
                return Some((start_in_window, idx));
            }
        }
    }

    None
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

// ============================================================================
// Local Search Operators
// ============================================================================

/// 2-opt: Reverse a segment within a route to reduce travel time.
/// Returns true if an improvement was made.
fn two_opt_improve<'a, V, R, A>(
    route: &mut RouteState<'a, V, R>,
    service_date: i64,
    availability: &A,
    matrix: &[Vec<i32>],
    index: &HashMap<String, usize>,
    options: &SolveOptions,
) -> bool
where
    V: Visit,
    R: Visitor<Id = V::VisitorId>,
    A: AvailabilityProvider<VisitorId = V::VisitorId>,
{
    if route.visits.len() < 3 {
        return false;
    }

    let current_cost = route.total_travel_time;
    let n = route.visits.len();

    for i in 0..n - 1 {
        for j in i + 2..n {
            // Reverse segment [i+1..=j]
            let mut candidate = route.visits.clone();
            candidate[i + 1..=j].reverse();

            let candidate_route = RouteState {
                visitor: route.visitor,
                visits: candidate,
                estimated_windows: Vec::new(),
                total_travel_time: 0,
            };

            if let Some((windows, cost)) = compute_schedule(
                service_date,
                &candidate_route,
                availability,
                matrix,
                index,
                options,
            ) {
                if cost < current_cost {
                    route.visits[i + 1..=j].reverse();
                    route.estimated_windows = windows;
                    route.total_travel_time = cost;
                    return true;
                }
            }
        }
    }

    false
}

/// Relocate: Move a visit from one route to another (or within the same route).
/// Returns true if an improvement was made.
fn relocate_improve<'a, V, R, A>(
    routes: &mut [RouteState<'a, V, R>],
    service_date: i64,
    availability: &A,
    matrix: &[Vec<i32>],
    index: &HashMap<String, usize>,
    options: &SolveOptions,
) -> bool
where
    V: Visit,
    R: Visitor<Id = V::VisitorId>,
    A: AvailabilityProvider<VisitorId = V::VisitorId>,
{
    let total_cost: i32 = routes.iter().map(|r| r.total_travel_time).sum();

    // Try moving each visit from each route to every other position
    for from_route_idx in 0..routes.len() {
        let from_route_len = routes[from_route_idx].visits.len();
        if from_route_len == 0 {
            continue;
        }

        for visit_idx in 0..from_route_len {
            let visit = routes[from_route_idx].visits[visit_idx];

            // Check if visit is pinned to current visitor
            let is_pinned_to_visitor = matches!(
                visit.pin_type(),
                VisitPinType::Visitor | VisitPinType::VisitorAndDate
            );

            // Try inserting into every route (including same route, different position)
            for to_route_idx in 0..routes.len() {
                // Skip moving pinned visits to different routes
                if is_pinned_to_visitor && to_route_idx != from_route_idx {
                    continue;
                }

                let to_route_len = routes[to_route_idx].visits.len();
                let insert_positions = if from_route_idx == to_route_idx {
                    to_route_len // same route: can insert at 0..len (excluding current position)
                } else {
                    to_route_len + 1 // different route: can insert at 0..=len
                };

                for insert_pos in 0..insert_positions {
                    // Skip if same route and same or adjacent position (no change)
                    if from_route_idx == to_route_idx {
                        if insert_pos == visit_idx || insert_pos == visit_idx + 1 {
                            continue;
                        }
                    }

                    // Check capability match for target route
                    let required = visit.required_capabilities();
                    if !required.is_empty() {
                        let available = routes[to_route_idx].visitor.capabilities();
                        if !required.iter().all(|cap| available.contains(cap)) {
                            continue;
                        }
                    }

                    // Build candidate routes
                    let mut from_candidate = routes[from_route_idx].visits.clone();
                    from_candidate.remove(visit_idx);

                    let mut to_candidate = if from_route_idx == to_route_idx {
                        from_candidate.clone()
                    } else {
                        routes[to_route_idx].visits.clone()
                    };

                    let actual_insert_pos = if from_route_idx == to_route_idx && insert_pos > visit_idx {
                        insert_pos - 1
                    } else {
                        insert_pos
                    };
                    to_candidate.insert(actual_insert_pos, visit);

                    // Compute new schedules
                    let from_route_state = RouteState {
                        visitor: routes[from_route_idx].visitor,
                        visits: if from_route_idx == to_route_idx {
                            to_candidate.clone()
                        } else {
                            from_candidate
                        },
                        estimated_windows: Vec::new(),
                        total_travel_time: 0,
                    };

                    let from_schedule = compute_schedule(
                        service_date,
                        &from_route_state,
                        availability,
                        matrix,
                        index,
                        options,
                    );

                    if from_schedule.is_none() {
                        continue;
                    }

                    if from_route_idx == to_route_idx {
                        // Same route: just the new cost
                        let (windows, cost) = from_schedule.unwrap();
                        let other_cost: i32 = routes
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| *i != from_route_idx)
                            .map(|(_, r)| r.total_travel_time)
                            .sum();

                        if cost + other_cost < total_cost {
                            routes[from_route_idx].visits = to_candidate;
                            routes[from_route_idx].estimated_windows = windows;
                            routes[from_route_idx].total_travel_time = cost;
                            return true;
                        }
                        continue;
                    } else {
                        // Different routes: compute both
                        let to_route_state = RouteState {
                            visitor: routes[to_route_idx].visitor,
                            visits: to_candidate.clone(),
                            estimated_windows: Vec::new(),
                            total_travel_time: 0,
                        };

                        let to_schedule = compute_schedule(
                            service_date,
                            &to_route_state,
                            availability,
                            matrix,
                            index,
                            options,
                        );

                        if to_schedule.is_none() {
                            continue;
                        }

                        let (from_windows, from_cost) = from_schedule.unwrap();
                        let (to_windows, to_cost) = to_schedule.unwrap();

                        let other_cost: i32 = routes
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| *i != from_route_idx && *i != to_route_idx)
                            .map(|(_, r)| r.total_travel_time)
                            .sum();

                        if from_cost + to_cost + other_cost < total_cost {
                            // Apply the move
                            routes[from_route_idx].visits.remove(visit_idx);
                            routes[from_route_idx].estimated_windows = from_windows;
                            routes[from_route_idx].total_travel_time = from_cost;

                            routes[to_route_idx].visits.insert(insert_pos, visit);
                            routes[to_route_idx].estimated_windows = to_windows;
                            routes[to_route_idx].total_travel_time = to_cost;
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

/// Run local search improvement until no more improvements or max iterations reached.
fn local_search<'a, V, R, A>(
    routes: &mut [RouteState<'a, V, R>],
    service_date: i64,
    availability: &A,
    matrix: &[Vec<i32>],
    index: &HashMap<String, usize>,
    options: &SolveOptions,
)
where
    V: Visit,
    R: Visitor<Id = V::VisitorId>,
    A: AvailabilityProvider<VisitorId = V::VisitorId>,
{
    for _ in 0..options.local_search_iterations {
        let mut improved = false;

        // Try 2-opt on each route
        for route in routes.iter_mut() {
            if two_opt_improve(
                route,
                service_date,
                availability,
                matrix,
                index,
                options,
            ) {
                improved = true;
            }
        }

        // Try relocate moves between routes
        if relocate_improve(
            routes,
            service_date,
            availability,
            matrix,
            index,
            options,
        ) {
            improved = true;
        }

        if !improved {
            break;
        }
    }
}
