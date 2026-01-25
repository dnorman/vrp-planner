# Routing Planner Tasks

## Immediate

- [ ] Define core domain types (job, route plan, vehicle/worker, availability).
- [ ] Define pin enum + fields (worker/date/both).
- [ ] Confirm committed vs target vs estimated time window fields.

## Research

- [ ] Finalize solver approach (internal vs vendor).
- [ ] Review vrp-core algorithms to scope minimal internal solver.
- [ ] Define distance model progression (Haversine -> OSRM matrix).

## Tests

- [ ] Build fixtures (3-10 vehicles, 20-100 jobs).
- [ ] Add tests for pinned worker/date, committed window, target time.
- [ ] Mock availability service responses.
- [ ] Add benchmark cases comparing Haversine vs map-based matrix.

## Implementation

- [ ] RoutePlan creation flow (lazy creation on optimization runs).
- [ ] Assignment + sequencing persistence.
- [ ] RouteOptimizationRun metrics and error handling.

## UI (Later)

- [ ] Pin job to worker/date/time.
- [ ] Visualize estimated vs committed windows.
- [ ] Manual resequencing with lock option.
