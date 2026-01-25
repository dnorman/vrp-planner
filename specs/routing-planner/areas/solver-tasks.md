# Solver Tasks

## Done

- [x] Implement cheapest insertion construction.
- [x] Define solution cost components (travel time, target deviation, stability penalty).
- [x] Define time budget and stability penalty defaults (see spec.md Cost Weights).

## In Progress

- [ ] Implement relocate operator (inter/intra-route moves).
- [ ] Implement 2-opt operator (segment reversal).
- [ ] Add stability penalty to cost function (requires `current_visitor_id()` on Visit).

## Future (v2)

- [ ] Add optional ruin & recreate pass.
- [ ] Add deterministic RNG seed control (needed when randomization is added).
- [ ] Define OSRM-first flow with clustering fallback thresholds.
- [ ] Workload balance penalty.
