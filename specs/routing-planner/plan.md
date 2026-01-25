# Routing Planner Plan

## Phase 1: Ontology + Data Model ✓

- [x] Define core domain types (visit, route plan, visitor, availability).
- [x] Ensure job supports committed window, target time, estimated window.
- [x] Add pin fields and enums to represent worker/date pinning.
- [x] Define capability matching (superset - visitor must have ALL required).
- [x] Add `current_visitor_id()` to Visit trait for stability.
- FieldOffice region mapping deferred to integration phase.

## Phase 2: Solver Strategy Decision ✓

- [x] Revisit VRP research summary and confirm solver approach.
- [x] **DECIDED**: Internal solver (Option C) inspired by vrp-core.
- [x] Document final decision and rationale.

## Phase 3: Solver Interface + Tests ✓

- [x] Define planner interface inputs/outputs (data structs).
- [x] Validate OSRM sidecar integration via testcontainers.
- [x] Build a unit-test suite with fixture problems.
- [x] Add tests for stability, capabilities, windows.
- [x] Mock availability service.
- [x] Add scale tests (50/100 visits performance validation).

## Phase 4: Implementation ✓

- [x] Cheapest insertion construction.
- [x] Auto-provision OSRM datasets (download + preprocess).
- [x] Relocate operator (local search).
- [x] 2-opt operator (local search).
- [x] Stability penalty in cost function.
- [x] Unassigned reason codes.
- [x] Haversine fallback distance matrix provider.
- Persistence deferred to integration phase.

## Phase 5: Integration (Future)

- Implement adapters in properlydone-platform.
- FieldOffice region mapping for OSRM dataset selection.
- Persist RoutePlans and assignments.
- Store RouteOptimizationRun metadata.

## Phase 6: UI Integration (Later)

- Operator tools: pin job to worker/date/time.
- Visualize estimated windows and committed windows.
- Manual override and re-optimization.
