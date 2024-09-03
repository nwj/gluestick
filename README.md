# Gluestick

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![CircleCI](https://dl.circleci.com/status-badge/img/circleci/VM8enYtRd7z5ktxpDSNF3i/AqwWXTi1cJw4FVs2Pt1g2Z/tree/main.svg?style=svg)](https://dl.circleci.com/status-badge/redirect/circleci/VM8enYtRd7z5ktxpDSNF3i/AqwWXTi1cJw4FVs2Pt1g2Z/tree/main)

### A self-hosted pastebin that makes it easy to share code, notes, and text snippets.

- [Features](#features)
- [Installation](#installation)
- [Administration](#installation)
- [API Documentation](#api-documentation)

## Disclaimer

- ⚠️ The project is under active development and has not yet reached a stable release.
- ⚠️ Expect bugs and breaking changes. Especially breaking changes.
- ⚠️ Do not use the app as the only way to store your data. Keep multiple, independent backups of everything you store in the app.

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

## API Documentation

Here are the endpoints and usage examples for the Gluestick JSON API.

### Base URL

All API requests in the examples below use the demo instance URL:

```
https://pastes.nwj.sh/api/v1
```

**Note:** The base URL will vary depending on your Gluestick instance. Replace `https://pastes.nwj.sh` with the appropriate URL for your specific Gluestick instance.

### Authentication

All requests must include an API key in the `X-GLUESTICK-API-KEY` header:

```
X-GLUESTICK-API-KEY: your_api_key_here
```

### Endpoints

#### List Pastes

Retrieves a list of pastes.

- **URL:** `/pastes`
- **Method:** GET

**Example Request:**
```bash
curl -H "X-GLUESTICK-API-KEY: your_api_key_here" https://pastes.nwj.sh/api/v1/pastes
```

**Example Response:**
```json
{
    "pastes": [
        {
            "id": "00000000-0000-0000-0000-000000000000",
            "user_id": "00000000-0000-0000-0000-000000000001",
            "filename": "example-paste.txt",
            "description": "An example paste",
            "body": "This is an example paste.",
            "visibility": "public",
            "created_at": "2024-01-01T01:01:01.001Z",
            "updated_at": "2024-01-02T01:01:01.001Z"
        },
        // ... more pastes ...
    ],
    "pagination": {
        "prev_page": null,
        "next_page": "00000000-0000-0000-0000-000000000002"
    }
}
```

#### Create Paste

Creates a new paste.

- **URL:** `/pastes`
- **Method:** POST

**Example Request:**
```bash
curl -X POST \
  -H "X-GLUESTICK-API-KEY: your_api_key_here" \
  -H "Content-Type: application/json" \
  -d '{"filename":"api_created.txt","description":"Example description","body":"Example body","visibility":"public"}' \
  https://pastes.nwj.sh/api/v1/pastes
```

**Example Response:**
```json
// The UUID of the newly created paste
"00000000-0000-0000-0000-000000000000"
```

#### Show Paste

Retrieves a specific paste.

- **URL:** `/pastes/:id`
- **Method:** GET

**Example Request:**
```bash
curl -H "X-GLUESTICK-API-KEY: your_api_key_here" https://pastes.nwj.sh/api/v1/pastes/00000000-0000-0000-0000-000000000000
```

**Example Response:**
```json
{
    "id": "00000000-0000-0000-0000-000000000000",
    "user_id": "00000000-0000-0000-0000-000000000001",
    "filename": "example-paste.txt",
    "description": "An example paste",
    "body": "This is an example paste.",
    "visibility": "secret",
    "created_at": "2024-01-01T01:01:01.001Z",
    "updated_at": "2024-01-02T01:01:01.001Z"
}
```

#### Show Raw Paste

Retrieves the raw content of a specific paste.

- **URL:** `/pastes/:id/raw`
- **Method:** GET

**Example Request:**
```bash
curl -H "X-GLUESTICK-API-KEY: your_api_key_here" https://pastes.nwj.sh/api/v1/pastes/00000000-0000-0000-0000-000000000000/raw
```

**Example Response:**
```
This is an example paste.
```

#### Update Paste

Updates an existing paste.

- **URL:** `/pastes/:id`
- **Method:** PATCH

**Example Request:**
```bash
curl -X PATCH \
  -H "X-GLUESTICK-API-KEY: your_api_key_here" \
  -H "Content-Type: application/json" \
  -d '{"filename":"updated-paste.txt","description":"This paste was updated","body":"An updated paste body"}' \
  https://pastes.nwj.sh/api/v1/pastes/00000000-0000-0000-0000-000000000000```

**Example Response:**
- **Status Code:** 200 OK
- **Body:** Empty

#### Delete Paste

Deletes a specific paste.

- **URL:** `/pastes/:id`
- **Method:** DELETE

**Example Request:**
```bash
curl -X DELETE \
  -H "X-GLUESTICK-API-KEY: your_api_key_here" \
  https://pastes.nwj.sh/api/v1/pastes/00000000-0000-0000-0000-000000000000
```

**Example Response:**
- **Status Code:** 200 OK
- **Body:** Empty

###  Error Handling

The API returns appropriate HTTP status codes along with JSON error messages for various error scenarios. Some common error responses include:

#### 400 Bad Request
```json
{
    "status": 400,
    "error": "Bad Request",
    "message": "Filename may not contain the following characters: < > : \" / \\ | ? *"
}
```

#### 401 Unauthorized
```json
{
    "status": 401,
    "error": "Unauthorized",
    "message": "Invalid authentication credentials."
}
```

#### 403 Forbidden
```json
{
    "status": 403,
    "error": "Forbidden",
    "message": "Insufficient privileges"
}
```

#### 404 Not Found
```json
{
    "status": 404,
    "error": "Not Found",
    "message": "Resource not found."
}
```
