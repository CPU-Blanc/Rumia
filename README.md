# Rumia

<a href="https://github.com/CPU-Blanc/Rumia/actions/workflows/CI.yml?query=branch%3Amaster">
<img alt="CI" src="https://github.com/CPU-Blanc/Rumia/actions/workflows/CI.yml/badge.svg?branch=master"/>
</a>
<a href="https://github.com/CPU-Blanc/Rumia/releases">
<img alt="Current Release" src="https://img.shields.io/github/release/CPU-Blanc/Rumia.svg?color=blue"/>
</a>
<a href="https://github.com/CPU-Blanc/Rumia/blob/master/LICENSE.md">
<img alt="AGPL 3.0 License" src="https://img.shields.io/github/license/CPU-Blanc/Rumia.svg"/>
</a>

## Overview
Rumia is a lightweight file server written in Rust, designed with simplicity in mind. No fancy AI tools, AWS S3, or even resizing; just post a request, store the file, and pull it down when you need it.

### Available Image Tags
- `latest` - The latest stable release
- `dev` - The latest build, includes pre-releases as well as stable releases - Whichever is newest
- Semver (ie `0.2`, `0.2.1` etc)

## Running
Rumia can be run inside a Docker container (recommended), or a CLI application

### Running the binary (CLI)
The binary runs as a normal CLI application (there is no GUI).\
You can provide the required arguments via the [environmental variables](#variables).
Use `rumia --help` for a full list of available options.\
You *must* select your storage type. Currently only `file-system` is supported, others such as AWS S3 may be supported in the future.

### Running with Docker Compose
Example compose:

```yaml
services:
  rumia:
    image: ghcr.io/cpu-blanc/rumia:latest
    container_name: rumia
    environment:
      - RUMIA_API_KEY=api_key  #replace with a key you wish to use
      - RUMIA_URL=https://files.mydomain.com #replace with your public url exposing the service
      - RUMIA_PORT=10032
      - RUMIA_STORAGE=FILE #use the local filesystem for storage. Only option for now 
    ports:
      - "10032:10032"
    volumes:
      - files:/filestore
    restart: unless-stopped

volumes:
  files:
```

### Running with Docker Create
```
docker create \
    --name rumia \
    -e RUMIA_API_KEY=api_key \
    -e RUMIA_URL=https://files.mydomain.com \
    -v files:/filestore
    -p 10032:10032 \
    --restart unless-stopped \
    ghcr.io/cpu-blanc/rumia:latest
```

### Variables
| Env var         | CLI arg          | Type         | Default          | Info                                                                                                                               |
|-----------------|------------------|--------------|------------------|------------------------------------------------------------------------------------------------------------------------------------|
| `RUMIA_API_KEY` | `-a`,`--api-key` | `String`     | **Required**     | API key to use for authorisation                                                                                                   |
| `RUMIA_PORT`    | `-p`,`--port`    | `Int`        | 10032            | Port to bind to. You cannot set this with the Docker version, instead set it using Docker's port mapping                           |
| `RUMIA_URL`     | `-u`,`--url`     | `String`     | http://localhost | URL which your instance is available at                                                                                            |
| `RUMIA_VERBOSE` | `-v`,`--verbose` | `Bool`       | `false`          | Verbose logging                                                                                                                    |
| `RUMIA_IP`      | `-i`,`--ip`      | `Ipv4Addr`   | `0.0.0.0`        | IP address to bind to                                                                                                              |
| `RUMIA_STORAGE` | N/A              | enum: `file` | `file`           | What storage system to use. `file` (filesystem) is currently the only supported<br>Storage type is selected from subcommand on CLI |

#### File storage settings:
| Env var                 | CLI arg  | Type     | Default                                   | Info                             |
|-------------------------|----------|----------|-------------------------------------------|----------------------------------|
| `RUMIA_FILESYSTEM_PATH` | `--path` | `String` | CLI: **Required**<br>Docker: `/filestore` | Filesystem path to save files to | 

## Endpoints

All endpoints marked with '🔒' are protected and require authorisation by providing the `x-api-key` header with your chosen API key.

---

### `GET /health`
#### Responses
| Code     | Body |
|----------|------|
| 200 - OK | "ok" |

---

### `GET /attachment/<filepath>`
#### Responses
| Code             | Info                                  |
|------------------|---------------------------------------|
| 200 - OK         | Returns the file bytes                |
| 400 - BadRequest | The provided filepath is malformed    |
| 404 - NotFound   | The file does not exist on the server |

---

### 🔒 `DELETE /attachment/<filepath>`
#### Responses
| Code               | Info                                                |
|--------------------|-----------------------------------------------------|
| 200 - OK           | The file was successfully deleted                   |
| 400 - BadRequest   | The provided filepath is malformed                  |
| 401 - Unauthorised | The provided API key is either missing or incorrect |
| 404 - NotFound     | The file does not exist on the server               | 


---

### 🔒 `POST /api/upload/file` 
#### Request Type: Multipart Form
| Field    | Value       | Required |
|----------|-------------|----------|
| file     | binary data | ✅        |
| filename | string      | ✅        |
#### Responses
| Code                       | Info                                                |
|----------------------------|-----------------------------------------------------|
| 200 - OK                   | Returns the full URL path of the uploaded file      |
| 400 - BadRequest           | Required fields are either missing, or malformed    |
| 401 - Unauthorised         | The provided API key is either missing or incorrect |
| 415 - UnsupportedMediaType | The file type is blacklisted (eg .exe, .dll)        |

---

### 🔒 `POST /api/upload/<url>`
#### Request Type: URL Path
`url` - Must be a url-encoded link to a raw resource
#### Responses
| Code                       | Info                                                                            |
|----------------------------|---------------------------------------------------------------------------------|
| 200 - OK                   | Returns the full URL path of the uploaded file                                  |
| 400 - BadRequest           | The URL could not be parsed                                                     |
| 401 - Unauthorised         | The provided API key is either missing or incorrect                             |
| 415 - UnsupportedMediaType | The file type is blacklisted (eg .exe, .dll)                                    |
| 424 - FailedDependency     | The upstream server did not respond with binary data                            |
| 502 - BadGateway           | Unable to connect to the upstream server                                        |
| Other                      | Any error codes generated by the upstream server will be forwarded and returned | 

---

### Example requests:
Request:
```
curl --request POST \
    --url http://localhost:10032/api/upload/file \
    --header 'x-api-key:your_key' \
    --form file=@/home/NAME/Pictures/cat.jpg \
    --form filename=cat.jpg
```
Response:
```
status: 200
body: http://localhost:10032/attachment/9206667b-869d-4fba-8dee-aa44c0facbd6/cat.jpg
```
Request:
```
curl --request POST \
    -H 'x-api-key:your_key' \
    http://localhost:10032/api/upload/https%3A%2F%2Fraw.githubusercontent.com%2FCPU-Blanc%2FRumia%2Frefs%2Fheads%2Fmaster%2Frumia%2Fresources%2Ftest%2Ftest.jpg
```
Response:
```
status: 200
body: http://localhost:10032/attachment/9206667b-869d-4fba-8dee-aa44c0facbd6/test.jpg
```