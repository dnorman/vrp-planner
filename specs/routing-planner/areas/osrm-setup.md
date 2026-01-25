# OSRM Sidecar Setup (Nevada)

## Download Data

```bash
mkdir -p osrm-data/nevada
cd osrm-data/nevada
curl -L -o nevada-latest.osm.pbf \
  https://download.geofabrik.de/north-america/us/nevada-latest.osm.pbf
```

## Preprocess (MLD)

```bash
docker run -t -v "$PWD:/data" osrm/osrm-backend \
  osrm-extract -p /opt/car.lua /data/nevada-latest.osm.pbf

docker run -t -v "$PWD:/data" osrm/osrm-backend \
  osrm-partition /data/nevada-latest.osrm

docker run -t -v "$PWD:/data" osrm/osrm-backend \
  osrm-customize /data/nevada-latest.osrm
```

## Run OSRM HTTP Server

```bash
docker run -t -i -p 5000:5000 -v "$PWD:/data" osrm/osrm-backend \
  osrm-routed --algorithm mld /data/nevada-latest.osrm
```

## Sanity Check

```bash
curl "http://localhost:5000/table/v1/car/-115.1728,36.1147;-115.1580,36.1727?annotations=duration"
```

## Testcontainers Integration

Tests auto-download and preprocess into `./osrm-data` if files are missing. You can override the root path with:

```bash
export OSRM_DATA_DIR=./osrm-data
```

Then run:

```bash
cargo test --tests
```
