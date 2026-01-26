# Ontology Tasks

## Done

- [x] Finalize trait names (Visit, Visitor, RoutePlan).
- [x] Define pin semantics and required fields.
- [x] Define time window semantics (committed vs target).
- [x] Define estimated window output semantics (seconds from midnight, start/end pairs).
- [x] Define solution output schema (`PlannerResult` with `RouteResult` per visitor).
- [x] Define capability matching semantics (superset - visitor must have ALL required).
- [x] Add `current_visitor_id()` to Visit trait for stability penalty.
- [x] Add unassigned reason enum to solver output (WrongDate, MissingPinnedVisitor, NoCapableVisitor, NoFeasibleWindow).

## Future

- [ ] Document invariants and validation rules.

## Deferred to Integration

- [ ] Define FieldOffice region mapping for OSRM datasets (handled by adapter layer).
