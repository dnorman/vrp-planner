# Integration Tasks

Integration with properlydone-platform-routing.

## Adapter Layer - COMPLETE ✓

- [x] Define adapter interface for Visits/Visitors (maps ProperlyDone models to traits)
- [x] Define mapping rules for pins and windows
- [x] Define mapping for solver outputs (sequence, estimates, unassigned reasons)
- [x] Implement `VisitAdapter`, `VisitorAdapter`, `AvailabilityAdapter`

## OSRM Region Selection - COMPLETE ✓

- [x] Add `osrm_region` field to FieldOffice in properlydone-platform
- [x] Adapter uses FieldOffice region to select OSRM dataset
- [x] Auto-prepare OSRM data on first use via `osrm-prepare` module

## Route Planner Service - COMPLETE ✓

- [x] Create `services/route-planner` crate with service architecture
- [x] Implement `RoutePlannerService` orchestration
- [x] Implement `VisitWatcher` with LiveQuery subscription for visit changes
- [x] Implement `recalculate_route_etas()` for day-of-service adjustments
- [x] Handle status transitions: InProgress, Completed, Skipped, NoAccess

## Notes

- vrp-planner traits are domain-agnostic
- properlydone-platform implements adapters that satisfy those traits
- Service crate watches for visit changes and triggers recalculation automatically
