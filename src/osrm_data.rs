//! OSRM dataset preparation helpers (download + preprocess).

use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct GeofabrikRegion {
    /// Geofabrik region path, e.g. "north-america/us/nevada".
    pub path: String,
}

impl GeofabrikRegion {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    pub fn name(&self) -> String {
        self.path
            .split('/')
            .last()
            .unwrap_or("region")
            .to_string()
    }

    pub fn url(&self) -> String {
        format!(
            "https://download.geofabrik.de/{}-latest.osm.pbf",
            self.path
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OsrmPrepMode {
    Mld,
}

#[derive(Debug, Clone)]
pub struct OsrmDatasetConfig {
    pub region: GeofabrikRegion,
    pub data_root: PathBuf,
    pub mode: OsrmPrepMode,
}

impl OsrmDatasetConfig {
    pub fn new(region: GeofabrikRegion, data_root: impl Into<PathBuf>) -> Self {
        Self {
            region,
            data_root: data_root.into(),
            mode: OsrmPrepMode::Mld,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OsrmDataset {
    pub data_dir: PathBuf,
    pub osrm_base: PathBuf,
    pub pbf_path: PathBuf,
}

#[derive(Debug)]
pub enum OsrmDataError {
    Io(io::Error),
    Http(reqwest::Error),
    ProcessFailure(String),
}

impl From<io::Error> for OsrmDataError {
    fn from(err: io::Error) -> Self {
        OsrmDataError::Io(err)
    }
}

impl From<reqwest::Error> for OsrmDataError {
    fn from(err: reqwest::Error) -> Self {
        OsrmDataError::Http(err)
    }
}

impl OsrmDataset {
    pub fn ensure(config: &OsrmDatasetConfig) -> Result<Self, OsrmDataError> {
        let region_name = config.region.name();
        let data_root = if config.data_root.is_absolute() {
            config.data_root.clone()
        } else {
            std::env::current_dir()?.join(&config.data_root)
        };
        let data_dir = data_root.join(region_name);
        fs::create_dir_all(&data_dir)?;

        let pbf_name = format!("{}-latest.osm.pbf", config.region.name());
        let pbf_path = data_dir.join(pbf_name);
        if !pbf_path.exists() {
            download_pbf(&config.region.url(), &pbf_path)?;
        }

        let osrm_base = data_dir.join(format!("{}-latest.osrm", config.region.name()));
        if !osrm_base.exists() {
            run_docker(&[
                "osrm-extract",
                "-p",
                "/opt/car.lua",
                &format!("/data/{}", file_name(&pbf_path)),
            ], &data_dir)?;
        }

        match config.mode {
            OsrmPrepMode::Mld => {
                if !mld_ready(&osrm_base) {
                    run_docker(
                        &["osrm-partition", &format!("/data/{}", file_name(&osrm_base))],
                        &data_dir,
                    )?;
                    run_docker(
                        &["osrm-customize", &format!("/data/{}", file_name(&osrm_base))],
                        &data_dir,
                    )?;
                }
            }
        }

        Ok(Self {
            data_dir,
            osrm_base,
            pbf_path,
        })
    }
}

fn download_pbf(url: &str, dest: &Path) -> Result<(), OsrmDataError> {
    let response = reqwest::blocking::get(url)?.error_for_status()?;
    let tmp_path = dest.with_extension("tmp");
    let mut writer = BufWriter::new(File::create(&tmp_path)?);
    let bytes = response.bytes()?;
    writer.write_all(&bytes)?;
    writer.flush()?;
    fs::rename(tmp_path, dest)?;
    Ok(())
}

fn mld_ready(osrm_base: &Path) -> bool {
    let partition = osrm_base.with_extension("osrm.partition");
    let mldgr = osrm_base.with_extension("osrm.mldgr");
    let cells = osrm_base.with_extension("osrm.cells");
    osrm_base.exists() && partition.exists() && mldgr.exists() && cells.exists()
}

fn run_docker(args: &[&str], data_dir: &Path) -> Result<(), OsrmDataError> {
    let status = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-t")
        .arg("-v")
        .arg(format!("{}:/data", data_dir.display()))
        .arg("osrm/osrm-backend")
        .args(args)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(OsrmDataError::ProcessFailure(format!(
            "docker exited with status {}",
            status
        )))
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string()
}
