#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Opts<'a> {
  pub cmd: Option<&'a str>,
}

macro_rules! make_template {
  ($name:ident, $path:expr) => {
    #[cfg_attr(debug_assertions, derive(::std::fmt::Debug))]
    #[derive(::rinja::Template)]
    #[template(path = $path)]
    pub struct $name<'a>(pub &'a self::Opts<'a>);

    impl<'a> ::std::ops::Deref for $name<'a> {
      type Target = self::Opts<'a>;
      fn deref(&self) -> &Self::Target {
        self.0
      }
    }
  };
}

make_template!(Bash, "bash");
make_template!(Zsh, "zsh");
