use crate::command::Execute;
use crate::database::{Database, DbOperation, Record};
use crate::util;
use anyhow::Result;
use clap::Parser;

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Parser)]
#[command(disable_help_subcommand = true)]
pub struct New {
  #[arg(required = true)]
  /// Command to alias, format: `<origin>=<alias>`
  pub command: String,
  /// Command parameter(s), format: `<origin>=<alias>`
  pub params: Vec<String>,
}

impl Execute for New {
  fn execute(&self) -> Result<()> {
    let mut db = Database::open()?;

    let (origin, alias) = util::alias_to_pair(self.command.as_ref())?;
    let mappings = self
      .params
      .iter()
      .map(|pair| {
        util::alias_to_pair(pair.as_ref()).map(|(origin, alias)| {
          (origin.to_owned().into(), alias.to_owned().into())
        })
      })
      .flatten()
      .collect::<Vec<_>>();

    let record = Record {
      origin: origin.to_owned().into(),
      alias: alias.to_owned().into(),
      mappings,
    };

    db.add_record(record);
    db.save()?;

    Ok(())
  }
}
