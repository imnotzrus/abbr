use std::env;
use std::path::PathBuf;

use anyhow::{ensure, Context, Result};

pub const DATA_DIR: &str = "_ABBR_DATA_DIR";

pub fn data_dir() -> Result<PathBuf> {
  let dir = match env::var_os(DATA_DIR) {
    Some(dir) => PathBuf::from(dir),
    None => dirs::data_local_dir()
      .context(format!(
        "failed to find data directory, please set {DATA_DIR} manually"
      ))?
      .join("abbreviator"),
  };

  ensure!(dir.is_absolute(), format!("{DATA_DIR} must be an absolute path"));
  Ok(dir)
}
