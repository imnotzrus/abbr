mod command;
mod config;
mod database;
mod error;
mod shell;
mod util;

use crate::command::{Command, Execute};
use crate::error::SilentExit;
use clap::Parser;
use std::process::ExitCode;
use std::{
  env,
  io::{self, Write},
};

fn main() -> ExitCode {
  env::remove_var("RUST_LIB_BACKTRACE");
  env::remove_var("RUST_BACKTRACE");

  if let Err(err) = Command::parse().execute() {
    return match err.downcast::<SilentExit>() {
      Ok(SilentExit { code }) => code.into(),
      Err(err) => {
        _ = writeln!(io::stderr(), "abbr: {err:?}");
        ExitCode::FAILURE
      }
    };
  }

  ExitCode::SUCCESS
}
