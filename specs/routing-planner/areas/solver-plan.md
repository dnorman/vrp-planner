# Solver Plan

## Goal

Define the algorithmic pipeline and heuristics for v1, with clear upgrade paths.

## Deliverables

- Construction heuristic definition.
- Local search operators.
- Objective function/penalties.
- Determinism strategy (seed handling).

## Steps

1. Specify problem representation for solver core.
2. Choose construction heuristic (cheapest insertion).
3. Define local search operators (relocate, 2-opt).
4. Define objective weights and penalty structure.
5. Define deterministic seed handling.
6. Define OSRM-first routing flow with clustering fallback thresholds.
7. Define time budget and stability penalty defaults.
