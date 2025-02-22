mod edit;
mod init;
mod new;
mod query;

use anyhow::Result;
use clap::Parser;

pub trait Execute {
  fn execute(&self) -> Result<()>;
}

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Parser)]
#[command(disable_help_subcommand = true)]
pub enum Command {
  /// Initialize tool
  Init(init::Init),
  /// New command alias
  New(new::New),
  /// Edit command alias
  Edit(edit::Edit),
  Query(query::Query),
}

impl Execute for Command {
  fn execute(&self) -> Result<()> {
    match self {
      Command::Init(init) => init.execute(),
      Command::New(new) => new.execute(),
      Command::Edit(edit) => edit.execute(),
      Command::Query(query) => query.execute(),
    }?;
    Ok(())
  }
}
