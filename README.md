# SQLite Studio

Single binary, single command SQLite database explorer. SQLite studio supports opening both *local SQLite DB files*, *libSQL servers* and *PostgreSQL*.

### Local SQLite DB File

```bash
sqlite-studio sqlite [sqlite_db]
```

### Remote libSQL Server

```bash
sqlite-studio libsql [url] [auth_token]
```

### PostgreSQL Server

```bash
sqlite-studio postgres [url]
```

## Features

- Overview page with common metadata.
- Tables page with each table's metadata, including the disk size being used by each table.
- Infinite scroll rows view.
- A custom query page that gives you more access to your db.

More features available on the [releases page](https://github.com/frectonz/sqlite-studio/releases).

## Screenshots

### Home Page

![homepage](./screenshots/homepage.png)

### Tables Page

![tables](./screenshots/tables.png)
![infinite scroll](https://github.com/frectonz/sqlite-studio/assets/53809656/b6d8f627-4a21-46c2-bef7-8dea206b3689)

### Query Page

![query](./screenshots/query.png)
![query gif](https://github.com/frectonz/sqlite-studio/assets/53809656/3e47a890-ddd9-4c7f-be88-53e30cc23b15)

## Installation

### Install prebuilt binaries via shell script (MacOS and Linux)

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/frectonz/sqlite-studio/releases/download/0.1.8/sqlite-studio-installer.sh | sh
```

### Install prebuilt binaries via powershell script

```sh
powershell -c "irm https://github.com/frectonz/sqlite-studio/releases/download/0.1.8/sqlite-studio-installer.ps1 | iex"
```

### Updating

```bash
sqlite-studio-update
```

## Nix

```bash
nix shell github:frectonz/sqlite-studio
```

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
