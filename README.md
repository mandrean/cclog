# cclog

A [conventional changelog][convention] for the rest of us.

`cclog` is a fork of [clog-lib](https://github.com/clog-tool/clog-lib) and
[clog-cli](https://github.com/clog-tool/clog-cli), merged into a single
Cargo workspace and modernized for Rust 2024 edition.

| Crate | Description |
|-------|-------------|
| [`cclog`](lib/) | Library for generating changelogs from git metadata |
| [`cclog-cli`](cli/) | Command-line tool wrapping the library |

## How it works

Every time you make a commit, ensure your subject line follows the
[conventional commit][convention] format:

```
type(component): message
```

Then run `cclog` to generate a changelog from your git history.

Supported commit types include `feat`, `fix`, `perf`, and
[custom sections](#custom-sections). Empty components are also supported:
`feat: message` or `feat(): message`.

## CLI

### Install

```sh
cargo install cclog-cli
```

Or build from source:

```sh
cargo build --release -p cclog-cli
```

### Usage

```
Usage: cclog [OPTIONS]

Options:
  -r, --repository <URL>  Repository URL for commit/issue links
  -f, --from <COMMIT>     Starting commit (e.g. 12a8546)
  -t, --to <COMMIT>       Ending commit [default: HEAD]
  -T, --format <STR>      Output format: markdown or json [default: markdown]
  -M, --major             Increment major version (sets minor and patch to 0)
  -m, --minor             Increment minor version (sets patch to 0)
  -p, --patch             Increment patch version
      --setversion <VER>  Set version explicitly (e.g. 1.0.1)
  -F, --from-latest-tag   Use latest tag as start (instead of --from)
  -o, --outfile <PATH>    Write changelog to file (defaults to stdout)
  -i, --infile <PATH>     Append old changelog data after new entries
  -C, --changelog <PATH>  Read and prepend to existing changelog
  -c, --config <FILE>     Config file [default: .clog.toml]
  -s, --subtitle <STR>    Release subtitle
  -g, --git-dir <PATH>    Path to .git directory
  -w, --work-tree <PATH>  Path to git working tree
  -l, --link-style <STR>  Link style: github, gitlab, stash, cgit [default: github]
  -h, --help              Print help
  -V, --version           Print version
```

### Examples

Generate a changelog from the latest tag:

```sh
cclog -r https://github.com/user/repo --from-latest-tag --changelog CHANGELOG.md
```

Bump patch version and write to stdout:

```sh
cclog -r https://github.com/user/repo --patch --from-latest-tag
```

### Configuration

Create a `.clog.toml` in your repository root:

```toml
[clog]
repository = "https://github.com/user/repo"
changelog = "CHANGELOG.md"
link-style = "github"
from-latest-tag = true
```

See [`lib/examples/clog.toml`](lib/examples/clog.toml) for all available options.

### Custom sections

By default, three sections are shown: **Features**, **Performance**, and
**Bug Fixes**. Add more via `.clog.toml`:

```toml
[sections]
"Breaking Changes" = ["break", "breaking"]
Refactoring = ["refactor", "ref"]
```

Then commits like `break(parser): remove deprecated API` will appear under
"Breaking Changes".

## Library

Add `cclog` to your `Cargo.toml`:

```toml
[dependencies]
cclog = "0.12"
```

```rust
use cclog::Clog;

fn main() {
    let clog = Clog::with_git_work_tree(".")
        .unwrap()
        .repository("https://github.com/user/repo")
        .subtitle("My Release")
        .changelog("CHANGELOG.md")
        .from("v0.1.0")
        .version("0.2.0");

    clog.write_changelog().unwrap();
}
```

See the [API docs](https://docs.rs/cclog) for full details.

## License

MIT — see [LICENSE](LICENSE) for details.

[convention]: https://github.com/ajoslin/conventional-changelog/blob/a5505865ff3dd710cf757f50530e73ef0ca641da/conventions/angular.md
