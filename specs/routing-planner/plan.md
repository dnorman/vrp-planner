# Routing Planner Plan

## Phase 1: Ontology + Data Model ✓ COMPLETE

- [x] Define core domain types (visit, route plan, visitor, availability).
- [x] Ensure job supports committed window, target time, estimated window.
- [x] Add pin fields and enums to represent worker/date pinning.
- [x] Define capability matching (superset - visitor must have ALL required).
- [x] Add `current_visitor_id()` to Visit trait for stability.
- FieldOffice region mapping deferred to integration phase.

## Phase 2: Solver Strategy Decision ✓ COMPLETE

- [x] Revisit VRP research summary and confirm solver approach.
- [x] **DECIDED**: Internal solver (Option C) inspired by vrp-core.
- [x] Document final decision and rationale.

## Phase 3: Solver Interface + Tests ✓ COMPLETE

- [x] Define planner interface inputs/outputs (data structs).
- [x] Validate OSRM sidecar integration via testcontainers.
- [x] Build comprehensive test suite (70 tests including OSRM integration and unit tests).
- [x] Add tests for stability, capabilities, windows, pinning.
- [x] Mock availability service.
- [x] Add scale tests (50/100/140/200 visits).
- [x] Add quality benchmarks (local search, geographic efficiency).
- [x] Add composite real-world scenario tests.

## Phase 4: Implementation ✓ COMPLETE

- [x] Cheapest insertion construction.
- [x] Auto-provision OSRM datasets (download + preprocess).
- [x] Relocate operator (local search).
- [x] 2-opt operator (local search).
- [x] Stability penalty in cost function.
- [x] Unassigned reason codes.
- [x] Haversine fallback distance matrix provider.
- [x] Capability filtering at route level (bug fix).
- [x] Pinned visit protection in local search (bug fix).
- Persistence deferred to integration phase.

## Phase 5: Integration (Future)

- Implement adapters in properlydone-platform.
- FieldOffice region mapping for OSRM dataset selection.
- Persist RoutePlans and assignments.
- Store RouteOptimizationRun metadata.

## Phase 6: v2 Features (Future)

- Priority/urgency handling for emergency visits.
- VIP customer preferred scheduling.
- Break handling (lunch breaks, mid-day unavailability).
- Working hours limits (max hours per day, overtime avoidance).

## Phase 7: UI Integration (Later)

- Operator tools: pin job to worker/date/time.
- Visualize estimated windows and committed windows.
- Manual override and re-optimization.

---

## Performance Summary

| Problem Size | Time | Assignment Rate |
|-------------|------|-----------------|
| 50 visits / 5 techs | ~50ms | 100% |
| 100 visits / 10 techs | ~230ms | 100% |
| 140 visits / 14 techs | ~444ms | 100% |
| 200 visits / 20 techs | ~928ms | 100% |

## Quality Metrics

- Local search improvement: 9.3% travel time reduction
- Geographic efficiency: 100% correct clustering
- Solution determinism: Consistent results across runs
