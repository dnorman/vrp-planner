# Routing Planner Plan

## Phase 1: Ontology + Data Model

- Define core domain types (job, route plan, vehicle/worker, availability).
- Ensure job supports committed window, target time, estimated window.
- Add pin fields and enums to represent worker/date pinning.

## Phase 2: Solver Strategy Decision

- Revisit VRP research summary and confirm solver approach.
- Decide between internal solver (inspired by vrp-core) vs vendor dependency.
- Document final decision and rationale.

## Phase 3: Solver Interface + Tests

- Define planner interface inputs/outputs (data structs).
- Build a unit-test suite with fixture problems.
- Mock availability service.

## Phase 4: Implementation

- Implement planner pipeline.
- Persist RoutePlans and job assignments.
- Store RouteOptimizationRun metadata.

## Phase 5: UI Integration (Later)

- Operator tools: pin job to worker/date/time.
- Visualize estimated windows and committed windows.
- Manual override and re-optimization.
