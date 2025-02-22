use crate::command::Execute;
use crate::database::{Database, DbOperation, EXCLUDED_CHARS};
use crate::util;
use anyhow::Result;
use clap::{Parser, Subcommand};

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone, Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum Action {
  /// Add new parameter alias(es), format: `<origin>=<alias>`
  Add { params: Vec<String> },
  /// Remove parameter alias(es)
  Rmv { params: Vec<String> },
  /// Remove command alias
  Del,
}

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Parser)]
pub struct Edit {
  #[arg(required = true)]
  /// Alias to edit
  pub alias: String,
  #[clap(subcommand)]
  /// Action to perform
  pub action: Action,
}

impl Execute for Edit {
  fn execute(&self) -> Result<()> {
    let mut db = Database::open()?;

    match &self.action {
      Action::Add { params } => {
        let mut mappings = Vec::with_capacity(params.len());
        for param in params {
          if param.contains(EXCLUDED_CHARS) {
            continue;
          }
          let (origin, alias) = util::alias_to_pair(param)?;
          mappings.push((origin, alias));
        }
        db.add_params(self.alias.as_str(), mappings.into_iter())
      }
      Action::Rmv { params } => {
        db.rem_params(self.alias.as_str(), params.iter().map(String::as_str))
      }
      Action::Del => db.del_record(self.alias.as_str()),
    }

    Ok(())
  }
}
