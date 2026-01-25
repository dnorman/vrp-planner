# Routing Planner Tasks

## Immediate

- [x] Define core domain types (visit, route plan, visitor, availability).
- [x] Define pin enum + fields (visitor/date/both).
- [x] Confirm committed vs target time window fields.
- [x] Define estimated window output schema (seconds from midnight, start/end pairs).
- [ ] Add `current_visitor_id()` to Visit trait for stability.
- [ ] Add unassigned reason codes to solver output.

## Research

- [x] Finalize solver approach (internal solver, Option C).
- [x] Review vrp-core algorithms to scope minimal internal solver.
- [x] Define distance model progression (OSRM-first with optional Haversine fallback).

## Tests

- [ ] Build fixtures (3-10 vehicles, 20-100 jobs).
- [ ] Add tests for pinned worker/date, committed window, target time.
- [ ] Add tests for stability penalty (reassignment).
- [ ] Add tests for capability matching (superset).
- [ ] Mock availability service responses.
- [ ] Add benchmark cases comparing Haversine vs map-based matrix.
- [x] Add OSRM sidecar integration test (MLD).

## Implementation

- [x] Cheapest insertion construction.
- [x] 2-opt operator (local search).
- [x] Relocate operator (local search).
- [ ] Stability penalty in cost function.
- [ ] RoutePlan creation flow (lazy creation on optimization runs).
- [ ] Assignment + sequencing persistence.
- [ ] RouteOptimizationRun metrics and error handling.
- [x] Add OSRM HTTP client adapter.
- [x] Auto-provision OSRM datasets (download + preprocess).

## UI (Later)

- [ ] Pin job to worker/date/time.
- [ ] Visualize estimated vs committed windows.
- [ ] Manual resequencing with lock option.
