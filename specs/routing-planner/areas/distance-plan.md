# Distance Plan

## Goal

Define a distance/time matrix strategy that scales from Haversine to OSRM.

## Deliverables

- Matrix provider interface.
- OSRM HTTP integration plan (default).
- Haversine reference implementation (fallback).
- Matrix caching strategy.

## Steps

1. Define matrix provider API (inputs/outputs, units).
2. Document OSRM sidecar setup and API calls.
3. Implement Haversine matrix generator (fallback).
4. Define caching policy (per planning run or per day).
5. Clarify OSRM `table` endpoint usage for planning.
