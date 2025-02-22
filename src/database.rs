use crate::{config, util};
use anyhow::{bail, Context, Result};
use bincode::Options;
use hashbrown::HashMap;
use ouroboros::self_referencing;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub const EXCLUDED_CHARS: &[char] = &['\r', '\n'];
type Str<'s> = Cow<'s, str>;

pub trait DbOperation {
  fn open() -> Result<Self>
  where
    Self: Sized;
  fn save(&mut self) -> Result<()>;
  fn add_record(&mut self, new: Record<'static>);
  fn add_params<S, P>(&mut self, alias: S, params: P)
  where
    S: AsRef<str> + Into<String>,
    P: Iterator<Item = (S, S)>;
  fn rem_params<S, P>(&mut self, alias: S, params: P)
  where
    S: AsRef<str>,
    P: Iterator<Item = S>;
  fn del_record<S>(&mut self, alias: S)
  where
    S: AsRef<str>;
  fn records(&self) -> &[Record];
}

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Serialize, Deserialize)]
pub struct Record<'r> {
  pub origin: Str<'r>,
  pub alias: Str<'r>,
  pub mappings: Vec<(Str<'r>, Str<'r>)>,
}

#[self_referencing]
pub struct Database {
  path: PathBuf,
  bytes: Vec<u8>,
  #[borrows(bytes)]
  #[covariant]
  records: Vec<Record<'this>>,
}

impl DbOperation for Database {
  fn open() -> Result<Self> {
    let data_dir = config::data_dir()?;

    Self::open_dir(data_dir)
  }

  fn save(&mut self) -> Result<()> {
    let bytes = Self::serialize(self.records())?;
    util::write(self.borrow_path(), bytes)
      .context("failed to write database")?;
    Ok(())
  }

  fn add_record(&mut self, new: Record<'static>) {
    self.with_records_mut(move |records| {
      if let Some(r) =
        records.iter_mut().find(|record| record.alias == new.alias)
      {
        *r = new;
      } else {
        records.push(new);
      }
    });
  }

  fn add_params<S, P>(&mut self, alias: S, params: P)
  where
    S: AsRef<str> + Into<String>,
    P: Iterator<Item = (S, S)>,
  {
    let alias = alias.as_ref();

    self.with_records_mut(|records| {
      if let Some(record) = records.iter_mut().find(|r| r.alias == alias) {
        let mut map = record.mappings.drain(..).collect::<HashMap<_, _>>();
        for (origin, alias) in params {
          map.insert(alias.into().into(), origin.into().into());
        }
        record.mappings = map.into_iter().collect();
      }
    });
  }

  fn rem_params<S, P>(&mut self, alias: S, params: P)
  where
    S: AsRef<str>,
    P: Iterator<Item = S>,
  {
    self.with_records_mut(|records| {
      if let Some(record) =
        records.iter_mut().find(|e| e.alias == alias.as_ref())
      {
        let mut map = record.mappings.drain(..).collect::<HashMap<_, _>>();
        for alias in params {
          map.remove(alias.as_ref());
        }
        record.mappings = map.into_iter().collect()
      }
    });
  }

  fn del_record<S>(&mut self, alias: S)
  where
    S: AsRef<str>,
  {
    if let Some(idx) =
      self.records().iter().position(|record| record.alias == alias.as_ref())
    {
      self.swap_remove(idx);
    }
  }

  fn records(&self) -> &[Record] {
    self.borrow_records()
  }
}

impl Database {
  fn open_dir(data_dir: impl AsRef<Path>) -> Result<Self> {
    let data_dir = data_dir.as_ref();
    let file_path = data_dir.join("db.abb");
    let path = fs::canonicalize(&file_path).unwrap_or(file_path);

    match fs::read(&path) {
      Ok(bytes) => Self::try_new(path, bytes, |bytes| Self::deserialize(bytes)),
      Err(e) if e.kind() == io::ErrorKind::NotFound => {
        // Create data directory, but don't create any file yet. The file will be
        // created later by [`Database::save`] if any data is modified.
        fs::create_dir_all(data_dir).with_context(|| {
          format!("unable to create data directory: {}", data_dir.display())
        })?;
        Ok(Self::new(path, Vec::new(), |_| Vec::new()))
      }
      Err(e) => Err(e).with_context(|| {
        format!("failed to read from database: {}", path.display())
      }),
    }
  }

  fn swap_remove(&mut self, idx: usize) {
    self.with_records_mut(|elements| elements.swap_remove(idx));
  }
}

impl Database {
  const VERSION: u32 = 8;

  fn serialize(records: &[Record]) -> Result<Vec<u8>> {
    (|| -> bincode::Result<_> {
      // Preallocate buffer with combined size of sections.
      let buffer_size = bincode::serialized_size(&Self::VERSION)?
        + bincode::serialized_size(&records)?;
      let mut buffer = Vec::with_capacity(buffer_size as usize);

      // Serialize sections into buffer.
      bincode::serialize_into(&mut buffer, &Self::VERSION)?;
      bincode::serialize_into(&mut buffer, &records)?;

      Ok(buffer)
    })()
    .context("failed to serialize database")
  }

  fn deserialize(bytes: &[u8]) -> Result<Vec<Record>> {
    // Assume a maximum size for the database. This prevents bincode from throwing
    // strange errors when it encounters invalid data.
    const MAX_SIZE: u64 = 32 << 20; // 32 MiB
    let deserializer =
      &mut bincode::options().with_fixint_encoding().with_limit(MAX_SIZE);

    // Split bytes into sections.
    let version_size = deserializer.serialized_size(&Self::VERSION)? as _;
    if bytes.len() < version_size {
      bail!("failed to deserialize database - data corrupted");
    }
    let (bytes_version, bytes_elements) = bytes.split_at(version_size);

    // Deserialize sections.
    let version = deserializer.deserialize(bytes_version)?;
    let elements = match version {
      Self::VERSION => deserializer
        .deserialize(bytes_elements)
        .context("failed to deserialize database")?,
      version => {
        bail!("unsupported version (got {version}, supports {})", Self::VERSION)
      }
    };

    Ok(elements)
  }
}

#[cfg(test)]
pub mod test {
  use crate::database::{DbOperation, Record};
  use hashbrown::HashMap;

  #[derive(Debug)]
  pub struct DummyDatabase {
    pub records: Vec<Record<'static>>,
  }

  impl DbOperation for DummyDatabase {
    fn open() -> anyhow::Result<Self>
    where
      Self: Sized,
    {
      let mut records = Vec::new();

      records.push(Record {
        origin: "echo".into(),
        alias: "e".into(),
        mappings: Vec::new(),
      });

      records.push(Record {
        origin: "git tag -d".into(),
        alias: "gtd".into(),
        mappings: Vec::new(),
      });

      records.push(Record {
        origin: "docker".into(),
        alias: "dk".into(),
        mappings: vec![
          ("cmp".into(), "compose".into()),
          ("ud".into(), "up -d".into()),
          ("i".into(), "image".into()),
          ("l".into(), "ls".into()),
        ],
      });

      records.push(Record {
        origin: "cargo".into(),
        alias: "ca".into(),
        mappings: vec![("a".into(), "add".into())],
      });

      Ok(Self { records })
    }

    fn save(&mut self) -> anyhow::Result<()> {
      Ok(())
    }

    fn add_record(&mut self, new: Record<'static>) {
      if let Some(r) = self.records.iter_mut().find(|r| r.alias == new.alias) {
        *r = new;
      } else {
        self.records.push(new);
      }
    }

    fn add_params<S, P>(&mut self, alias: S, params: P)
    where
      S: AsRef<str> + Into<String>,
      P: Iterator<Item = (S, S)>,
    {
      if let Some(record) =
        self.records.iter_mut().find(|r| r.alias == alias.as_ref())
      {
        let mut map = record.mappings.drain(..).collect::<HashMap<_, _>>();
        for (origin, alias) in params {
          map.insert(alias.into().into(), origin.into().into());
        }
        record.mappings = map.into_iter().collect();
      }
    }

    fn rem_params<S, P>(&mut self, alias: S, params: P)
    where
      S: AsRef<str>,
      P: Iterator<Item = S>,
    {
      if let Some(record) =
        self.records.iter_mut().find(|e| e.alias == alias.as_ref())
      {
        let mut map = record.mappings.drain(..).collect::<HashMap<_, _>>();
        for alias in params {
          map.remove(alias.as_ref());
        }
        record.mappings = map.into_iter().collect()
      }
    }

    fn del_record<S>(&mut self, alias: S)
    where
      S: AsRef<str>,
    {
      self.records.retain(|record| record.alias != alias.as_ref());
    }

    fn records(&self) -> &[Record] {
      self.records.as_slice()
    }
  }
}
