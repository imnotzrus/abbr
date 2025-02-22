use crate::command::Execute;
use crate::error::BrokenPipeHandler;
use crate::shell::{Bash, Opts, Zsh};
use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use rinja::Template;
use std::io::{self, Write};

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(ValueEnum, Clone)]
#[value(rename_all = "kebab-case")]
pub enum InitShell {
  Bash,
  Zsh,
}

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Parser)]
pub struct Init {
  /// Current shell to initialize
  #[arg(value_enum)]
  pub shell: InitShell,
  /// Set alias for this tool
  #[arg(long, default_value = "ab")]
  pub alias: String,
}

impl Execute for Init {
  fn execute(&self) -> Result<()> {
    let opts = Opts { cmd: Some(self.alias.as_str()) };
    let source = match self.shell {
      InitShell::Bash => Bash(&opts).render(),
      InitShell::Zsh => Zsh(&opts).render(),
    }
    .context("failed to render template")?;
    writeln!(io::stdout(), "{source}").pipe_exit("stdout")?;
    Ok(())
  }
}
