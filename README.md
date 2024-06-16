# SQLite Studio

Single binary, single command SQLite database explorer.

```bash
sqlite-studio <sqlite_db>
```

## Features

- Overview page with common metadata.
- Tables page with each table's metadata, including the disk size being used by each table.
- Infinite scroll rows view.
- A custom query page that gives you more access to your db.

More features available on the [releases page](https://hub.docker.com/r/frectonz/sqlite-studio).

## How To Run It

### Nix

If you are using [Nix](https://nixos.org/), to build it from source.

```bash
nix shell github:frectonz/sqlite-studio
sqlite-studio <sqlite_db>
```

### Pre-Built Binaries

You can find pre-built binaries for the following targets on the [releases](https://github.com/frectonz/sqlite-studio/releases) page.

- Linux `sqlite-studio_<release>_x86_64-unknown-linux-musl.zip`
- Windows `sqlite-studio_<release>_x86_64-pc-windows-gnu.zip`
- MacOS x86 `sqlite-studio_<release>_x86_64-apple-darwin.zip`

After downloading the ZIP archive, you can extract it and get the binary.

## Contributing

Before executing `cargo run` you need to build the UI because the rust app statically embedded the UI files in the binary.

```bash
git clone git@github.com:frectonz/sqlite-studio.git
cd sqlite-studio
nix develop # if you use nix
cd ui
npm install
npm run build
cd ..
cargo run <sqlite_db>
```
