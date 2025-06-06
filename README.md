# Rucho - Echo Server

Rucho is a simple, fast, and scalable HTTP echo server built using Rust and Axum. It's designed for testing, debugging, and simulating various HTTP behaviors.

## Getting started

1.  **Clone the repository**
    ```bash
    git clone https://github.com/SheriffTwinkie/rust-echo.git
    cd rust-echo
    ```

2.  **Build the project**
    ```bash
    cargo build
    ```

3.  **Run the server**
    ```bash
    cargo run
    ```

    The server will start at `http://localhost:8080` by default.

## Example usage

Here are a few basic curl examples:

```bash
# Simple GET request to the root
curl -s http://localhost:8080

# GET request, response details will be echoed as JSON
curl -s http://localhost:8080/get | jq

# POST request with a JSON body, the body will be echoed in the response
curl -s -X POST http://localhost:8080/post -H "Content-Type: application/json" -d "{\"test\": \"value\"}" | jq

# Simulate a 503 Service Unavailable response
curl -i http://localhost:8080/status/503
```

## Features

Rucho offers a variety of features to help with HTTP testing and debugging:

*   **OpenAPI/Swagger Documentation**: Interactive API documentation is available via Swagger UI. You can access it at `/swagger-ui` (e.g., `http://localhost:8080/swagger-ui`) when the server is running. The OpenAPI specification can be found at `/api-docs/openapi.json`.
*   **Key API Endpoints**:
    *   `GET /`: Displays a welcome message.
    *   `GET /get`: Echoes request details for GET requests.
    *   `POST /post`: Echoes request details for POST requests (expects JSON body).
    *   `PUT /put`: Echoes request details for PUT requests (expects JSON body).
    *   `PATCH /patch`: Echoes request details for PATCH requests (expects JSON body).
    *   `DELETE /delete`: Echoes request details for DELETE requests.
    *   `OPTIONS /options`: Responds with allowed HTTP methods for the server or a specific path.
    *   `ANY /status/:code`: Returns the specified HTTP status code (e.g., `/status/404` returns a 404 Not Found).
    *   `ANY /anything`: Echoes request details for any HTTP method.
    *   `ANY /anything/*path`: Echoes request details for any HTTP method under a specific path.
    *   `GET /delay/:n`: Delays the response by `n` seconds.
    *   `GET /healthz`: Performs a health check and returns "OK" if the server is healthy.
    *   `GET /endpoints`: Lists all available API endpoints.
*   **JSON Response Formatting**: Responses are consistently formatted as JSON (with optional pretty-printing via `?pretty=true`).
*   **Request Tracing**: Automatic request tracing and logging are enabled for observability.
*   **Comprehensive HTTP Method Support**: Handles all major HTTP methods (GET, POST, PUT, PATCH, DELETE, OPTIONS, HEAD).
*   **Dynamic Status Code Simulation**: Easily simulate any HTTP status code using the `/status/:code` endpoint.
*   **Configurable Response Delay**: Test client behavior with server-side delays using the `/delay/:n` endpoint.
*   **Health Checks**: A dedicated `/healthz` endpoint for monitoring server status.
*   **Flexible Configuration**: Configure the server using configuration files or environment variables.
*   **HTTPS Support**: Optional support for HTTPS via Rustls.
*   **Docker Support**: Easily deploy Rucho using Docker. The container runs with a non-root user for better security.

## License

This project is licensed under the MIT License. You can find the full license text in the [LICENSE.md](LICENSE.md) file.
