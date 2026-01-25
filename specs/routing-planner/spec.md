# Routing Planner Spec (10k ft)

## Mission

Build a small, deterministic VRP engine that assigns Visits to RoutePlans and sequences them while honoring hard constraints and minimizing travel cost. The engine should be domain-agnostic and reusable via traits.

## Scope

In scope:
- Single-day planning (multi-day out of scope for v1).
- Multiple visitors/vehicles with individual availability windows.
- Hard constraints: pins, committed windows, availability, required capabilities.
- Soft constraints: target time, keep-same-visitor preference, workload balance.
- Distance model progression: OSRM matrix by default; Haversine fallback only if OSRM is unavailable.

Out of scope (v1):
- Real-time reoptimization during the day.
- Dynamic orders arriving mid-route.
- Multi-depot fleet mixing.
- Capacities (optional v2).

## Ontology

- **Visit**: a single service occurrence to be routed.
- **Visitor**: a worker/vehicle that performs visits.
- **RoutePlan**: a visitor + service date container for ordered visits.
- **FieldOffice**: a planning region tied to a Geofabrik map area (used to select OSRM data).
- **Pin**: a hard constraint (visitor/date) that must not change.
- **Committed window**: customer-facing window that must be respected.
- **Target time**: preference that may be violated with penalty.
- **Estimated window**: solver output used for sequencing and UI display.

## System Boundaries

- The planner does not create Visits; it only assigns and sequences them.
- Availability is provided by an external provider (trait-based).
- Distances/time come from a matrix provider (trait-based).
- OSRM datasets are provisioned per FieldOffice; servers auto-download/preprocess if missing.

## Approach

1. Build a daily problem instance from Visits and Visitors.
2. Apply hard constraints and feasibility checks.
3. Construct an initial solution (fast constructive heuristic).
4. Improve via local search.
5. Persist RoutePlans, sequences, and estimated windows.

## Heuristic Strategy (v1)

- Construction: cheapest insertion with a simple seeding strategy.
- Local search: relocate + 2-opt (intra- and inter-route).
- Optional: ruin & recreate for diversification (off by default).

## Determinism

- All randomization uses a seeded RNG.
- Same inputs must produce the same output.

## Distance Model

- Default: OSRM sidecar (HTTP) providing travel time matrices.
- Fallback: Haversine (only if OSRM is unavailable and explicitly enabled).

## OSRM-First Flow (Tiered)

1. **Small N**: build full OSRM matrix and run construction + local search.
2. **Medium N**: build full Haversine matrix, solve, then OSRM-refine only final routes.
3. **Large N**: pre-cluster (k-means or grid), solve per cluster with Haversine, then OSRM-refine per cluster.

Clusters are **seed-only**; inter-route moves may cross cluster boundaries.

## Defaults (Initial)

- Thresholds: `N_small = 150`, `N_medium = 400` (tune with benchmarks).
- Time budget: 10s per run (configurable; allow 5–10s).
- Cost units: OSRM travel time in seconds (primary), distance is secondary metric.
- OSRM endpoint: `table` only for planning.
- Stability: soft penalty (do not hard-freeze assignments except for pins).
- Breaks: fixed duration, flexible start within a window.
 - Adaptive thresholds: optionally adjust `N_small`/`N_medium` based on observed runtime to stay within the time budget.

## Open Questions

- Thresholds for switching from full-matrix OSRM to clustered approximation.
- How to model stability penalties (avoid reshuffling already accepted plans).
- Initial objective weights for balancing cost vs. SLA compliance.
