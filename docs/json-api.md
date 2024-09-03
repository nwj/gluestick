# JSON API Documentation

This document outlines the endpoints and usage of the Gluestick JSON API.

## Base URL

All API requests in the examples below use the demo instance URL:

```
https://pastes.nwj.sh/api/v1
```

**Note:** The base URL will vary depending on your Gluestick instance. Replace `https://pastes.nwj.sh` with the appropriate URL for your specific Gluestick instance.

## Authentication

All requests must include an API key in the `X-GLUESTICK-API-KEY` header:

```
X-GLUESTICK-API-KEY: your_api_key_here
```

## Endpoints

### List Pastes

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

### Create Paste

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

### Show Paste

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

### Show Raw Paste

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

### Update Paste

Updates an existing paste.

- **URL:** `/pastes/:id`
- **Method:** PATCH

**Example Request:**
```bash
curl -X PATCH \
  -H "X-GLUESTICK-API-KEY: your_api_key_here" \
  -H "Content-Type: application/json" \
  -d '{"filename":"updated-paste.txt","description":"This paste was updated","body":"An updated paste body"}' \
  https://pastes.nwj.sh/api/v1/pastes/00000000-0000-0000-0000-000000000000
```

**Example Response:**
- **Status Code:** 200 OK
- **Body:** Empty

### Delete Paste

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

## Error Handling

The API returns appropriate HTTP status codes along with JSON error messages for various error scenarios. Some common error responses include:

### 400 Bad Request
```json
{
    "status": 400,
    "error": "Bad Request",
    "message": "Filename may not contain the following characters: < > : \" / \\ | ? *"
}
```

### 401 Unauthorized
```json
{
    "status": 401,
    "error": "Unauthorized",
    "message": "Invalid authentication credentials."
}
```

### 403 Forbidden
```json
{
    "status": 403,
    "error": "Forbidden",
    "message": "Insufficient privileges"
}
```

### 404 Not Found
```json
{
    "status": 404,
    "error": "Not Found",
    "message": "Resource not found."
}
```
