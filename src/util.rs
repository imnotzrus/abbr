use anyhow::{anyhow, Context, Result};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub const ALIAS_DELIMITER: char = '=';
pub const PARAM_DELIMITER: char = '/';

pub fn write(path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<()> {
  let path = path.as_ref();
  let contents = contents.as_ref();
  let dir = path.parent().unwrap();

  // Create a junk_file.
  let (mut junk_file, junk_path) = junk_file(dir)?;
  let result = (|| {
    // Write to the junk_file.
    _ = junk_file.set_len(contents.len() as u64);
    junk_file.write_all(contents).with_context(|| {
      format!("failed to write file: {}", junk_path.display())
    })?;

    // Set the owner of the junk_file (UNIX only).
    #[cfg(unix)]
    if let Ok(metadata) = path.metadata() {
      use std::os::unix::fs::MetadataExt;
      use std::os::unix::io::AsRawFd;

      use nix::unistd::{self, Gid, Uid};

      let uid = Uid::from_raw(metadata.uid());
      let gid = Gid::from_raw(metadata.gid());
      _ = unistd::fchown(junk_file.as_raw_fd(), Some(uid), Some(gid));
    }

    // Close and rename the junk_file.
    drop(junk_file);
    rename(&junk_path, path)
  })();
  // In case of an error, delete the junk_file.
  if result.is_err() {
    _ = fs::remove_file(&junk_path);
  }
  result
}

fn junk_file(dir: impl AsRef<Path>) -> Result<(File, PathBuf)> {
  const MAX_ATTEMPTS: usize = 5;
  const TMP_NAME_LEN: usize = 16;
  let dir = dir.as_ref();

  let mut attempts = 0;
  loop {
    attempts += 1;

    // Generate a random name for the junk_file.
    let mut name = String::with_capacity(TMP_NAME_LEN);
    name.push_str("junk_");
    while name.len() < TMP_NAME_LEN {
      name.push(fastrand::alphanumeric());
    }
    let path = dir.join(name);

    // Atomically create the junk_file.
    match OpenOptions::new().write(true).create_new(true).open(&path) {
      Ok(file) => break Ok((file, path)),
      Err(e)
        if e.kind() == io::ErrorKind::AlreadyExists
          && attempts < MAX_ATTEMPTS => {}
      Err(e) => {
        break Err(e).with_context(|| {
          format!("failed to create file: {}", path.display())
        });
      }
    }
  }
}

fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
  let from = from.as_ref();
  let to = to.as_ref();

  const MAX_ATTEMPTS: usize = if cfg!(windows) { 5 } else { 1 };
  let mut attempts = 0;

  loop {
    match fs::rename(from, to) {
      Err(e)
        if e.kind() == io::ErrorKind::PermissionDenied
          && attempts < MAX_ATTEMPTS =>
      {
        attempts += 1
      }
      result => {
        break result.with_context(|| {
          format!(
            "failed to rename file: {} -> {}",
            from.display(),
            to.display()
          )
        });
      }
    }
  }
}

pub fn alias_to_pair(alias: &str) -> Result<(&str, &str)> {
  // Format: <origin>=<alias>
  alias
    .split_once(ALIAS_DELIMITER)
    .ok_or(anyhow!("invalid alias format, must be `<origin>=<alias>`"))
}

pub fn param_to_pair(param: &str) -> Option<(&str, usize)> {
  param.split_once(PARAM_DELIMITER).and_then(|(param, amount)| {
    amount.parse::<usize>().ok().map(|amount| (param, amount))
  })
}
