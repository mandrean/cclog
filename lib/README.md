# cclog

[![crates.io][crate-image]][crate-link]
[![docs.rs][docs-image]][docs-link]

A conventional changelog generator for the rest of us.

This is a fork of [clog-lib](https://github.com/clog-tool/clog-lib) and
[clog-cli](https://github.com/clog-tool/clog-cli), merged into a single
[Cargo workspace].

| Crate | Description |
|-------|-------------|
| [`cclog`](lib/) | Library for generating a [conventional changelog][convention] from git metadata |
| [`cclog-cli`](cli/) | Command-line interface wrapping the library |

## Library usage

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

## CLI usage

```sh
cargo install cclog-cli
cclog --help
```

Or build from source:

```sh
cargo build --release -p cclog-cli
```

### Configuration

`cclog` can be configured using a `.clog.toml` file. See `lib/examples/clog.toml`
for available options.

## Related projects

- [Commitizen](http://commitizen.github.io/cz-cli/) — helps you write better
  commit messages.

## License

MIT — see [LICENSE](LICENSE) for details.

[Cargo workspace]: https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html
[convention]: https://github.com/ajoslin/conventional-changelog/blob/a5505865ff3dd710cf757f50530e73ef0ca641da/conventions/angular.md
[crate-image]: https://img.shields.io/crates/v/cclog.svg
[crate-link]: https://crates.io/crates/cclog
[docs-image]: https://img.shields.io/docsrs/cclog
[docs-link]: https://docs.rs/cclog
