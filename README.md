# 🚀 Echo Server (Rust + Axum)

Simple, fast, and scalable HTTP echo server built using Rust and Axum.  
Designed for testing, debugging, and simulating various HTTP behaviors.

---

## 🛠 Tech Stack

- [Rust](https://www.rust-lang.org/)
- [Axum](https://docs.rs/axum/latest/axum/)
- [Tokio](https://tokio.rs/)
- [Tower-HTTP](https://docs.rs/tower-http/latest/tower_http/)
- [Hyper](https://hyper.rs/)

---

## 📂 Project Structure

```bash
src/
├── main.rs             # Application entrypoint
├── lib.rs              # Library module declarations
├── routes/             # HTTP route handlers
│   ├── core_routes.rs  # Core echo and utility endpoints
│   ├── delay.rs        # Delay endpoint
│   ├── healthz.rs      # Health check endpoint
│   └── mod.rs          # Routes module declaration
└── utils/              # Utility modules
    ├── config.rs       # Configuration loading
    ├── error_response.rs # Standardized error responses
    ├── json_response.rs  # Standardized JSON responses
    ├── mod.rs          # Utils module declaration
    ├── request_models.rs # Request model structs (e.g., query params)
    └── server_config.rs # Server listener and TLS configuration
```

---

## 🚀 Getting Started

1. **Clone the repository**
   ```bash
   git clone https://github.com/SheriffTwinkie/rust-echo.git
   cd rust-echo
   ```

2. **Build the project**
   ```bash
   cargo build
   ```

3. **Run the server**
   ```bash
   cargo run
   ```

Server will start at:

```bash
http://localhost:8080
```

---

## 📜 Available Endpoints

| Method   | Path              | Description                                      |
|:--------:|:------------------:|:------------------------------------------------:|
| GET      | `/`                | Welcome message                                  |
| GET      | `/get`             | Echoes request details for GET                   |
| HEAD     | `/get`             | Responds with headers for GET query              |
| POST     | `/post`            | Echoes request details for POST, expects JSON body |
| PUT      | `/put`             | Echoes request details for PUT, expects JSON body  |
| PATCH    | `/patch`           | Echoes request details for PATCH, expects JSON body|
| DELETE   | `/delete`          | Echoes request details for DELETE                |
| OPTIONS  | `/options`         | Responds with allowed HTTP methods               |
| ANY      | `/status/:code`    | Returns the specified HTTP status code           |
| ANY      | `/anything`        | Echoes request details for any HTTP method       |
| ANY      | `/anything/*path`  | Echoes request details for any HTTP method under a specific path |
| GET      | `/delay/:n`        | Delays response by `n` seconds                   |
| GET      | `/healthz`         | Performs a health check, returns "OK"            |
| GET      | `/endpoints`       | Lists all available API endpoints                |
| GET      | `/swagger-ui`      | Displays OpenAPI/Swagger UI documentation        |

---

## OpenAPI/Swagger Documentation

Rucho includes OpenAPI (Swagger) documentation for its API endpoints.
You can access the Swagger UI by navigating to `/swagger-ui` in your browser when the server is running.

Example: `http://localhost:8080/swagger-ui`

The OpenAPI specification is available at `/api-docs/openapi.json`.

---

## 🧹 Features

- 📜 Clean JSON response formatting with newline (optional pretty-printing via `?pretty=true`).
- 📈 Automatic request tracing and logging using `TraceLayer`.
- 🔥 Support for all major HTTP methods (GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD, ANY).
- ⚡ Dynamic HTTP status simulation (`/status/200`, `/status/503`, etc).
- ⏱️ Configurable response delay endpoint (`/delay/:n`).
- ❤️ Health check endpoint (`/healthz`).
- 📖 Self-documenting API with OpenAPI/Swagger UI (`/swagger-ui`).
- 🗄️ Endpoint listing (`/endpoints`).
- ⚙️ Flexible configuration via files and environment variables.
- 🔒 Optional HTTPS support via Rustls.
- 🐳 Docker support with a non-root user.
- 🧹 Organized modular structure for easy expansion and maintenance.

---

## 🛠 Example Usage

### Basic curl examples:

```bash
# Simple GET
curl -s http://localhost:8080

# GET headers echoed as JSON
curl -s http://localhost:8080/get | jq

# POST body echoed
curl -s -X POST http://localhost:8080/post -H "Content-Type: application/json" -d "{\"test\": \"value\"}" | jq

# Simulate a 503 response
curl -i http://localhost:8080/status/503
```

✅ Output is always clean, newline-separated, and JSON-formatted where appropriate.

---

## Running with Docker

**User:** The application inside the Docker container runs as the non-root `rucho` user for enhanced security.

**Configuration:**
*   The Docker image comes with a default configuration file located at `/etc/rucho/rucho.conf`.
*   You can customize the configuration by providing your own `rucho.conf` and mounting it to `/etc/rucho/rucho.conf` in the container. For example:
    ```bash
    docker run -v ./my-rucho.conf:/etc/rucho/rucho.conf <image_name>
    ```
*   Alternatively, you can use environment variables as described in the "Configuration" section to override specific settings.
*   A clean sample configuration file (`rucho.conf.default`) is also available within the image at `/usr/share/doc/rucho/examples/rucho.conf.default` if you wish to use it as a template. You can copy it out using:
    ```bash
    docker cp <container_name_or_id>:/usr/share/doc/rucho/examples/rucho.conf.default ./rucho.conf.default
    ```

---

## Configuration

Rucho can be configured through configuration files and environment variables.

### Parameters

The following parameters can be configured:

*   `prefix`: The installation prefix or base directory for data.
    *   Default: `/usr/local/rucho`
    *   Config file key: `prefix`
    *   Environment variable: `RUCHO_PREFIX`
*   `log_level`: The logging verbosity.
    *   Default: `notice`
    *   Supported values (case-insensitive): `TRACE`, `DEBUG`, `INFO`, `NOTICE`, `WARN`, `ERROR`
    *   Config file key: `log_level`
    *   Environment variable: `RUCHO_LOG_LEVEL`
*   `server_listen_primary`: The primary listen address and port for the server.
    *   Default: `0.0.0.0:8080`
    *   Config file key: `server_listen_primary`
    *   Environment variable: `RUCHO_SERVER_LISTEN_PRIMARY`
*   `server_listen_secondary`: The secondary listen address and port for the server.
    *   Default: `0.0.0.0:9090`
    *   Config file key: `server_listen_secondary`
    *   Environment variable: `RUCHO_SERVER_LISTEN_SECONDARY`

### Configuration Loading Order

Configuration values are loaded in the following order of precedence (each step overrides the previous):

1.  **Hardcoded Defaults**: The application starts with built-in default values for all parameters.
2.  **System-wide Configuration File**: If `/etc/rucho/rucho.conf` exists, it is read and its values override the defaults.
3.  **Local Configuration File**: If `rucho.conf` exists in the current working directory from which Rucho is launched, it is read. Its values override both the defaults and any values from the system-wide configuration file.
4.  **Environment Variables**: Any environment variables starting with `RUCHO_` (e.g., `RUCHO_PREFIX`, `RUCHO_LOG_LEVEL`) will override values from all other sources.

### Using the Sample Configuration File

A sample configuration file, `rucho.conf.default`, is provided in the `config_samples/` directory of the source repository.
You can use this as a template:

*   For a **system-wide configuration**, copy it to `/etc/rucho/rucho.conf`:
    ```bash
    sudo mkdir -p /etc/rucho
    sudo cp config_samples/rucho.conf.default /etc/rucho/rucho.conf
    sudo nano /etc/rucho/rucho.conf # Edit as needed
    ```
*   For a **local configuration** (specific to a particular project or instance), copy it to the directory where you run Rucho:
    ```bash
    cp config_samples/rucho.conf.default ./rucho.conf
    nano ./rucho.conf # Edit as needed
    ```

Configuration files should use `key = value` pairs, one per line. Lines starting with `#` are treated as comments.

---

## 📝 Notes

- `TraceLayer` provides request/response logging automatically.
- JSON formatting is consistent across all echo endpoints.
- `.gitignore` excludes `target/`, `*.rs.bk` backups, and `Cargo.lock`.
- Project structure follows best practices for Rust Axum services.

---

## 📢 License

This project is licensed under the [MIT License](LICENSE).

---
