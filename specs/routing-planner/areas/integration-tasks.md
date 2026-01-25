# Integration Tasks

Integration happens AFTER vrp-planner is solid with cargo tests.

## Adapter Layer (Future)

- [ ] Define adapter interface for Visits/Visitors (maps ProperlyDone models to traits).
- [ ] Define mapping rules for pins and windows.
- [ ] Define mapping for solver outputs (sequence, estimates, unassigned reasons).
- [ ] Provide example adapter stub.

## OSRM Region Selection (Future)

- [ ] Add `geofabrik_region` field to FieldOffice in properlydone-platform.
- [ ] Adapter uses FieldOffice region to select OSRM dataset.

## Notes

- vrp-planner traits are domain-agnostic.
- properlydone-platform will implement adapters that satisfy those traits.
- Test with mocks in vrp-planner; integrate later.
