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

## Integration (Future)

- [ ] RoutePlan creation flow (lazy creation on optimization runs).
- [ ] Assignment + sequencing persistence.
- [ ] RouteOptimizationRun metrics and error handling.
- [ ] Implement adapters in properlydone-platform.
- [ ] FieldOffice region mapping for OSRM dataset selection.

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
