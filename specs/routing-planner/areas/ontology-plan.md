# Ontology Plan

## Goal

Define the domain model and trait surface for the routing engine in a reusable, domain-agnostic way.

## Deliverables

- Core traits: Visit, Visitor, RoutePlan, AvailabilityProvider, DistanceMatrixProvider.
- Pin semantics and window definitions.
- Minimal solution/output struct schema.

## Steps

1. Draft trait interfaces and pin enums.
2. Define required vs optional fields for each trait.
3. Specify invariants (e.g., pin requires pinned_* present).
4. Document serialization considerations (deterministic input/output).
