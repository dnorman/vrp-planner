# Distance Plan

## Goal

Define a distance/time matrix strategy that defaults to OSRM (MLD) with optional Haversine fallback.

## Deliverables

- Matrix provider interface.
- OSRM HTTP integration plan (default, MLD).
- OSRM dataset provisioning strategy (auto-download + preprocess).
- Haversine reference implementation (fallback).
- Matrix caching strategy.

## Steps

1. Define matrix provider API (inputs/outputs, units).
2. Document OSRM sidecar setup and API calls.
3. Define dataset lifecycle (Geofabrik region selection, cache root).
4. Implement Haversine matrix generator (fallback).
5. Define caching policy (per planning run or per day).
6. Clarify OSRM `table` endpoint usage for planning.
