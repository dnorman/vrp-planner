# Routing Planner Research

## Summary (Current)

Motivation: existing Rust VRP crates either lack long-term support, are large and opaque, or do not give the level of control we need over constraints and debugging. The goal is a small, readable solver we fully own.

Key findings:
- `vrp-pragmatic` is the only mature pure-Rust solver, but it is large (~35k LOC), single-maintainer, and marked "permanently experimental".
- Tests on a realistic service-routing problem passed (skills, time windows, breaks respected).
- A focused in-house solver could be ~500-2,000 LOC for the constraints we need.
- Recommendation so far: build an internal solver inspired by `vrp-core` (Option C).

## Service Routing Needs (Domain-Agnostic)

Typical characteristics:
- Recurring service is the norm; one-off jobs are the exception.
- Strong worker-customer affinity (keep the same worker whenever possible).
- Day-of-week preferences often matter more than exact arrival time for routine stops.
- Time windows exist for appointments, but the majority of visits are flexible.
- Routes are geographically clustered, minimizing drive time.
- Shifts are well-defined; breaks are required but flexible within a window.
- Travel distance optimization matters, but not at the expense of constraint violations.
- Problem size is small-to-medium (3–10 vehicles, 20–100 jobs/day).
- Operators expect stability (avoid reshuffling already confirmed routes).

Implications for the solver:
- Hard constraints: pinned worker/date, committed window, availability, breaks, required capabilities.
- Soft constraints: target time, keep-same-worker preference, compact routes, workload balance.
- Solve for a single day at a time; multi-day optimization is not required.
- Deterministic output is preferred (operators need stable results).

## Distance Model and Map Data

Current baseline:
- Assume OSRM sidecar is available for travel time matrices.
- Haversine is an explicit fallback only when OSRM is unavailable.

Progression path:
- Introduce a pluggable distance/time matrix provider.
- Start with a local OSRM instance using OpenStreetMap extracts.
- Precompute a matrix for each planning run (depots + jobs).

Notes:
- OSM data is free to download (no subscription), but hosting/routing is on us.
- OSRM needs preprocessed map data (regional extracts are manageable; full-planet is heavy).
- Matrix computation scales with N^2; for 100 jobs + 10 vehicles, ~110^2 ~= 12k pairs.
- For small N, on-demand A* can work, but a matrix is simpler and deterministic.

## Practical Implications

- The routing planner should own domain types and constraints, not import them from a large external framework.
- The architecture should allow provider swapping (internal vs external) without rewriting downstream systems.

## Benchmarking Approach (Using vrp-pragmatic)

Use `vrp-pragmatic` for comparative benchmarking only:
- Convert a day's jobs + availability into the pragmatic JSON format.
- Run solver and capture total travel time, feasibility, and constraint violations.
- Compare against internal solver output on the same input.
- Do not depend on runtime integration in production.

## OSRM Crate Options (Rust)

Snapshot from crates.io (for evaluation only):
- `osrm_interface` (MIT): supports both native C++ bindings and HTTP API (features: `native`, `remote`).
- `osrm_client` (MIT): lightweight HTTP client for OSRM services.
- `rs_osrm` (MIT): wrapper for OSRM (FFI oriented).
- `osrm-binding` (MIT): low-level unsafe FFI bindings.
- `osrm_parser` (MIT): request builder/response parser.
- `osrm` (MIT): minimal bindings (very early version).

Recommendation for v1:
- Prefer HTTP integration (OSRM as sidecar service) using `osrm_client` or plain `reqwest`.
- Avoid FFI bindings unless we have a strong need to embed OSRM directly.

Decision:
- Use a sidecar OSRM service (HTTP) as the default distance/time source.

## Library Strategy: Use vs Extract vs Rebuild

Recommended approach:
- **Do not depend on `vrp-pragmatic` in production** (maintenance risk, reported constraint bugs).
- **Use `vrp-pragmatic` as a benchmark/reference** for solution quality and constraint behavior.
- **Implement a minimal internal solver** using well-known heuristics:
  - Construction: nearest neighbor or cheapest insertion
  - Local search: relocate + 2-opt
  - Optional: basic ruin & recreate

Reference extraction guidance:
- `vrp-core` skills constraint is small and readable; use it as a behavioral reference.
- Reimplement algorithms using our own domain types to keep the codebase small and debuggable.
- Maintain a deterministic seed for reproducibility.
- Apache-2.0 license in `vendor/vrp` allows reimplementation/inspiration with attribution if needed.

## Method Selection (Current POV)

- **v1**: construction + local search (cheapest insertion, relocate, 2-opt) for speed and simplicity.
- **v2+**: consider ALNS/LNS once constraints and data models stabilize.
- **Routing costs**: use OSRM matrix as default; avoid embedded routing in the solver core.

## Reference Notes from `vrp-core`

- Skills constraint uses three sets: `all_of`, `one_of`, `none_of`.
  - `all_of` must be subset of vehicle skills.
  - `one_of` requires at least one overlap.
  - `none_of` must be disjoint.
- Breaks are treated as activities with time windows; routes can insert breaks where feasible.
- Break policy options include skipping when there is no intersection with the shift window or when arrival is before break window end.
- The approach is modular but tightly coupled to internal types (`InsertionContext`, `RouteContext`).

## Open Research Tasks

- Validate solver quality on larger inputs (50-100 visits, 5-10 techs).
- Confirm how `vrp-core` models skills/time windows/breaks for reference heuristics.
- Identify minimal set of heuristics that deliver acceptable quality:
  - Construction: nearest neighbor / cheapest insertion
  - Local search: relocate + 2-opt
  - Optional: ruin & recreate

## Next Research Steps (Concrete)

- Create a 50–100 job synthetic service-routing case using existing test structure.
- Benchmark `vrp-pragmatic` vs a baseline greedy solver for quality/feasibility.
- Document which constraints are truly required for v1 vs can be deferred.

## Decision Log

- **DECIDED**: Internal solver (Option C) - build a minimal solver inspired by vrp-core.
- Rationale: Full control over constraints, debugging, and maintenance. ~500-2,000 LOC for v1 constraints.
- v1 scope: pins, committed windows, availability, required capabilities (superset match), target time, stability penalty.
- Breaks deferred to v2.

## References (Starting Point)

- Solomon (1987): VRPTW benchmark instances and construction heuristics.
- Clarke–Wright Savings (1964): classic construction heuristic for VRP.
- Savelsbergh (1992): local search in routing (relocate, swap, 2-opt, Or-opt).
- Shaw (1998): large neighborhood search (ruin & recreate).
- Pisinger & Ropke (2007): adaptive large neighborhood search (ALNS).
