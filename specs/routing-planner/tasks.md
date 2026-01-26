# Routing Planner Tasks

## Immediate - COMPLETE ✓

- [x] Define core domain types (visit, route plan, visitor, availability).
- [x] Define pin enum + fields (visitor/date/both).
- [x] Confirm committed vs target time window fields.
- [x] Define estimated window output schema (seconds from midnight, start/end pairs).
- [x] Add `current_visitor_id()` to Visit trait for stability.
- [x] Add unassigned reason codes to solver output.

## Research - COMPLETE ✓

- [x] Finalize solver approach (internal solver, Option C).
- [x] Review vrp-core algorithms to scope minimal internal solver.
- [x] Define distance model progression (OSRM-first with optional Haversine fallback).

## Tests - COMPLETE ✓

- [x] Build fixtures (3-10 vehicles, 20-200 jobs).
- [x] Add tests for pinned worker/date, committed window, target time.
- [x] Add tests for stability penalty (reassignment).
- [x] Add tests for capability matching (superset).
- [x] Mock availability service responses.
- [x] Add scale tests (50/100/140/200 visits, performance validation).
- [x] Add quality benchmarks (local search improvement, geographic efficiency).
- [x] Add composite real-world scenario tests.
- [x] Add OSRM sidecar integration test (MLD).
- [x] Add realistic routing tests with Las Vegas coordinates via OSRM (3 tests).
- [ ] Add benchmark cases comparing Haversine vs map-based matrix.

## Implementation - COMPLETE ✓

- [x] Cheapest insertion construction.
- [x] 2-opt operator (local search).
- [x] Relocate operator (local search).
- [x] Stability penalty in cost function.
- [x] Capability filtering at route level.
- [x] Pinned visit protection in local search.
- [x] Add OSRM HTTP client adapter.
- [x] Auto-provision OSRM datasets (download + preprocess).
- [x] Haversine fallback distance matrix provider.

## Integration - COMPLETE ✓

### Model Changes (properlydone-platform-routing) - COMPLETE ✓

- [x] Add `target_time: Option<SecondsFromMidnight>` to Visit model.
- [x] Add `osrm_region: Option<String>` to FieldOffice.

### Trait Adapters (properlydone-platform-routing) - COMPLETE ✓

Located in `server/src/routing/`:

- [x] `VisitAdapter` implementing `vrp_planner::traits::Visit`
  - Dereference Property for `location()` → `(lat, lng)`
  - Prefetch `VisitCapabilityRequirement` for `required_capabilities()`
  - Map `Visit.technician` to `current_visitor_id()`
  - Map `Visit.pin_type` enum (Tech→Visitor, TechAndDate→VisitorAndDate)
- [x] `VisitorAdapter` implementing `vrp_planner::traits::Visitor`
  - Join `User` + `EmployeeWorkSchedule` for `start_location()`
  - Prefetch `TechnicianCapability` for `capabilities()`
- [x] `AvailabilityAdapter` implementing `vrp_planner::traits::AvailabilityProvider`
  - Wrap `availability_for_user_date()` from utils
  - Merge multiple windows to outer bounds: `(first.start, last.end)`
  - Return `None` if no windows (user unavailable)

### OSRM Region Selection - IN PROGRESS

- [x] Add `osrm_region: Option<String>` to FieldOffice (Geofabrik path).
- [ ] Map FieldOffice → OSRM dataset in solver invocation.
- [x] Fallback to Haversine if no region configured.

### Persistence - COMPLETE ✓

- [x] Create/update RoutePlan records from solver output.
- [x] Update Visit.route_plan, sequence_order, estimated_window_start/end.
- [x] Set Visit.unassigned_reason for unassigned visits.
- [x] Create RouteOptimizationRun with status, metrics, timing.

### Testing - IN PROGRESS

- [x] Integration test: properlydone models → vrp-planner → result application.
- [ ] Add benchmark cases comparing Haversine vs OSRM matrix.

## v2 Features (Future)

- [ ] Priority/urgency handling for emergency visits.
- [ ] VIP customer preferred scheduling.
- [ ] Break handling (lunch breaks, mid-day unavailability).
- [ ] Working hours limits (max hours per day, overtime avoidance).

## UI (Later)

- [ ] Pin job to worker/date/time.
- [ ] Visualize estimated vs committed windows.
- [ ] Manual resequencing with lock option.

## Bug Fixes Applied

- [x] Capability filtering bug: now checks at route level, not just global.
- [x] NoCapableVisitor reason: correctly identifies when no *available* visitor has capability.
- [x] Relocate operator: now respects pinned visits (was moving them between routes).
