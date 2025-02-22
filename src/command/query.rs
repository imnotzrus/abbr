use crate::command::Execute;
use crate::database::{Database, DbOperation};
use crate::util;
use anyhow::{Context, Result};
use clap::Parser;
use hashbrown::HashMap;
use std::io::{self, Write};

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Parser)]
#[command(disable_help_flag = true, disable_help_subcommand = true)]
pub struct Query {
  #[arg(required = true)]
  pub alias: String,
  pub params: Vec<String>,
}

impl Execute for Query {
  fn execute(&self) -> Result<()> {
    let db = Database::open()?;
    let order = self.build_order(&db)?;
    let handle = &mut io::stdout();
    write!(handle, "{}", order.join(" "))?;

    Ok(())
  }
}

impl Query {
  fn build_order<'s>(
    &'s self,
    db: &'s impl DbOperation,
  ) -> Result<Vec<&'s str>> {
    let record = db
      .records()
      .iter()
      .find(|record| record.alias == self.alias)
      .context(format!("alias `{}` not found", self.alias))?;

    let mut order = Vec::with_capacity(self.params.len() + 1);
    order.push(record.origin.as_ref());

    let mut mappings = HashMap::with_capacity(record.mappings.len());
    for (alias, origin) in record.mappings.iter() {
      mappings.insert(alias.as_ref(), origin.as_ref());
    }

    let mut idx = 0;
    while idx < self.params.len() {
      let param = &self.params[idx];
      match mappings.get(param.as_str()) {
        Some(origin) => order.push(*origin),
        None => match util::param_to_pair(param) {
          Some((param, amount)) => {
            match mappings.get(param) {
              Some(origin) => order.push(*origin),
              None => order.push(param),
            }
            for i in 1..=amount {
              order.push(&self.params[idx + i]);
            }
            idx += amount;
          }
          None => order.push(param),
        },
      }
      idx += 1;
    }

    Ok(order)
  }
}

#[cfg(test)]
mod tests {
  use crate::command::query::Query;
  use crate::database::{test::DummyDatabase, DbOperation};

  #[test]
  fn test_echo() {
    let db = DummyDatabase::open().unwrap();
    let query = Query {
      alias: "e".to_string(),
      params: vec!["hello".to_string(), "world".to_string()],
    };

    let order = query.build_order(&db).unwrap();
    assert_eq!(order.join(" "), String::from("echo hello world"));
  }

  #[test]
  fn test_git_tag() {
    let db = DummyDatabase::open().unwrap();
    let query = Query {
      alias: "gtd".to_string(),
      params: vec!["tag1,tag2,tag3".to_string()],
    };

    let order = query.build_order(&db).unwrap();
    assert_eq!(order.join(" "), String::from("git tag -d tag1,tag2,tag3"));
  }

  #[test]
  fn test_docker_compose_up_d() {
    let db = DummyDatabase::open().unwrap();
    let query = Query {
      alias: "dk".to_string(),
      params: vec!["cmp".to_string(), "ud".to_string()],
    };

    let order = query.build_order(&db).unwrap();
    assert_eq!(order.join(" "), String::from("docker compose up -d"));
  }

  #[test]
  fn test_docker_image_ls() {
    let db = DummyDatabase::open().unwrap();
    let query = Query {
      alias: "dk".to_string(),
      params: vec!["i".to_string(), "l".to_string()],
    };

    let order = query.build_order(&db).unwrap();
    assert_eq!(order.join(" "), String::from("docker image ls"));
  }

  #[test]
  fn test_cargo_add() {
    let db = DummyDatabase::open().unwrap();
    let query = Query {
      alias: "ca".to_string(),
      params: vec![
        "a/3".to_string(),
        "serde".to_string(),
        "tokio".to_string(),
        "clap".to_string(),
      ],
    };

    let order = query.build_order(&db).unwrap();
    assert_eq!(order.join(" "), String::from("cargo add serde tokio clap"));
  }
}
