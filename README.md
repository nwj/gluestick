# Gluestick

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CircleCI](https://dl.circleci.com/status-badge/img/circleci/VM8enYtRd7z5ktxpDSNF3i/AqwWXTi1cJw4FVs2Pt1g2Z/tree/main.svg?style=svg)](https://dl.circleci.com/status-badge/redirect/circleci/VM8enYtRd7z5ktxpDSNF3i/AqwWXTi1cJw4FVs2Pt1g2Z/tree/main)

### A self-hosted pastebin that makes it easy to share code, notes, and text snippets.

## Disclaimer

- ⚠️ The project is under active development and has not yet reached a stable release.
- ⚠️ Expect bugs and breaking changes. Especially breaking changes.
- ⚠️ Do not use the app as the only way to store your data. Keep multiple, independent backups of everything you store in the app.

- [Features](#features)
- [Installation](#installation)
- [Administration](#installation)

## Features

- Upload and share code, notes, and other text snippets (called "pastes")
- "Secret" pastes (accessible to whoever has the URL, but unindexed)
- Syntax highlighting
- One-click paste copying
- One-click paste file downloads
- Raw text views of all pastes
- Multi-user support (with invite-only sign ups)
- JSON API (requires authentication via API key)

## Installation

Gluestick is written in Rust and currently must be compiled from source. I have tested that it builds without issue on MacOS Sonoma and Ubuntu 24.04 LTS, but it will likely build in any UNIX-like environment supported by the Rust compiler.

To build:

1. [Install Rust](https://www.rust-lang.org/tools/install), if you don't already have it.
2. Clone the Gluestick git repo: `git clone git@github.com:nwj/gluestick.git`
3. Step into the repo folder: `cd gluestick`
4. Run the compiler: `cargo build --release`

If successful, the Gluestick executable will be at `./target/release/gluestick`

## Administration

### Configuring and running the app server

Gluestick ships as a single binary executable. Simply execute it to run the app server: `./target/release/gluestick`.

The app server can be configured via the following environment variables:

- `GLUESTICK_PORT`: The port that the server will listen for TCP connections on. Defaults to `3000`.
- `GLUESTICK_DB_PATH`: The relative file path for the SQLite database file that the server will read from and write to. If no database file is present at the specified path, the server will create and migrate a new database at that path. Defaults to `gluestick.db`. 

Additionally, Gluestick will attempt to read these environment variables out of a `.env` file, when such a file is present.

### Backups

Gluestick stores all of its data in a SQLite database file, located by default at `gluestick.db` within the file directory where the app server is executed.

To take a manual backup: `sqlite3 gluestick.db ".backup 'backup.db'"`. Then backup the resulting `backup.db` file.

Backup can be further improved by automating and scheduling the manual backup process above, or by using [Litestream](https://litestream.io/) to continuously replicate the database to external file storage.
