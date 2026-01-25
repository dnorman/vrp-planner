# Routing Planner Spec (10k ft)

## Mission

Build a small, deterministic VRP engine that assigns Visits to RoutePlans and sequences them while honoring hard constraints and minimizing travel cost. The engine should be domain-agnostic and reusable via traits.

## Scope

In scope (v1):
- Single-day planning.
- Multiple visitors/vehicles with individual availability windows.
- Hard constraints: pins, committed windows, availability, required capabilities.
- Soft constraints: target time, keep-same-visitor preference (stability), workload balance.
- Distance model: OSRM matrix by default; Haversine fallback only if OSRM is unavailable.
- Capability matching: visitor must have ALL required capabilities (superset match).

Out of scope (v1):
- Real-time reoptimization during the day.
- Dynamic orders arriving mid-route.
- Multi-depot fleet mixing.
- Capacities (v2).
- Break handling (v2 - fixed duration, flexible start within a window).

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
- Adaptive thresholds: optionally adjust `N_small`/`N_medium` based on observed runtime to stay within the time budget.

## Cost Weights (Defaults)

These are initial values; tune based on real-world feedback.

| Component | Weight | Notes |
|-----------|--------|-------|
| Travel time | 1.0 | Primary objective (seconds) |
| Target time deviation | 0.5 | Penalty per second of deviation from target |
| Reassignment penalty | 300 | Penalty for changing visitor (stability) |
| Workload imbalance | 0.3 | Penalty for uneven distribution (v2) |

## Stability

- Visits track their current visitor assignment via `current_visitor_id()`.
- Reassigning a visit to a different visitor incurs a soft penalty.
- Pins are hard constraints and override stability preferences.

## Open Questions

- Thresholds for switching from full-matrix OSRM to clustered approximation (needs benchmarking).
- Workload balance metric definition (total time? visit count?).
