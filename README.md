# remove-crate-renames

Remove renames from `Cargo.toml` and the rust code. The crate parses `Cargo.toml` files looks for dependencies (and dev-depdendencies) that are renamed via `package = ` and removes them in favour of full crate name.


## Usage

```
$ cargo run <crate-path> | bash
```

This runs the command and applies the changes.

Or use:

```
$ find <workspace-path> -name Cargo.toml | xargs -n 1 cargo run -- | bash
```

To find and apply changes in batch to all crates in the workspace.
