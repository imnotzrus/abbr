<div align="center">

# Abbreviator

A tool helps you abbreviate your command and its arguments, inspired by **zoxide**

It remembers which command you want to abbreviate,
just like *alias* does but with its arguments in short also,
in your own way.

[Getting started](#getting-started) •
[Installation](#installation) •
[Configuration](#configuration)

</div>

## Getting started

```shell
ab ca a/3 serde tokio clap   # equivalents to `cargo add serde tokio clap` 
ab dk cmp ud                 # equivalents to `docker compose up -d`
ab dk i l                    # equivalents to `docker image ls`
ab e "hello world"           # equivalents to `echo "hello world"`
ab gis                       # equivalents to `git status`
```

## Installation

1. **Install binary**

   abbr runs on most major platforms. If your platform is not listed below,
   please [open an issue][issues].

   <details>
   <summary>Linux / WSL</summary>

   > The recommended way to install abbr is via the installation script:
   >
   > ```shell
   > curl -sSfL https://raw.githubusercontent.com/ajeetdsouza/zoxide/main/install.sh | sh
   > ```

   </details>

   <details>
   <summary>MacOS</summary>

   > Run this command in your terminal:
   > ```sh
   > curl -sSfL https://raw.githubusercontent.com/imnotzrus/abbr/main/install.sh | sh
   > ```

   </details>

   <details>
   <summary>Windows</summary>

   > If you're using Cygwin, Git Bash, or MSYS2, you can also use the install script:
   >
   > ```sh
   > curl -sSfL https://raw.githubusercontent.com/ajeetdsouza/zoxide/main/install.sh | sh
   > ```

   </details>

   <details>
   <summary>Android</summary>

   > Run this command in your terminal:
   > ```sh
   > curl -sS https://raw.githubusercontent.com/ajeetdsouza/zoxide/main/install.sh | bash
   > ```

   </details>

2. **Setup abbr on your shell**
   To start using abbr, add it to your shell.

   <details>
   <summary>Bash</summary>

   > Add this to the <ins>**end**</ins> of your config file (usually `~/.bashrc`):
   >
   > ```sh
   > eval "$(zoxide init bash)"
   > ```

   </details>

   <details>
   <summary>Zsh</summary>

   > Add this to the <ins>**end**</ins> of your config file (usually `~/.zshrc`):
   >
   > ```sh
   > eval "$(zoxide init zsh)"
   > ```

   </details>

<!-- 3. Install fzf <sup>(optional)</sup> (not yet supported) -->

## Configuration

### Flags

When calling `abbr init`, the following flag is available: `--alias`

- Change prefix of `abbr` command.
- `--alias aa` will change the command to `aa`.

### Environment variables

Environment variable `_ABBR_DATA_DIR` can be used for configuration. They must be set before
`abbr init` is called.

- Specifies the directory in which the database is stored.
- The default value varies across OSes:

| OS          | Path                                     | Example                                    |
|-------------|------------------------------------------|--------------------------------------------|
| Linux / BSD | `$XDG_DATA_HOME` or `$HOME/.local/share` | `/home/alice/.local/share`                 |
| macOS       | `$HOME/Library/Application Support`      | `/Users/Alice/Library/Application Support` |
| Windows     | `%LOCALAPPDATA%`                         | `C:\Users\Alice\AppData\Local`             |

[issues]: https://github.com/imnotzrus/abbr/issues/new