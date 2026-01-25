# Distance Tasks

## Done

- [x] Add OSRM HTTP client adapter.
- [x] Auto-download + preprocess OSRM datasets (MLD).
- [x] Implement Haversine matrix generator (fallback).
- [x] Specify units: travel time in seconds.

## Future

- [ ] Define OSRM dataset cache root and eviction policy.
- [ ] Add matrix caching (keyed by date + locations hash).
- [ ] Add benchmark comparing OSRM vs Haversine matrix.
- [ ] Document table-only planning (no stored routing geometry).
