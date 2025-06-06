# Project Roadmap for Rucho

## Introduction

This document outlines the development roadmap for Rucho, our HTTP echo server. It provides insights into our current status, short-term objectives, and long-term vision for the project. We aim to keep this roadmap updated as the project evolves.

## Current Status

Rucho's core echo server functionality is stable and operational. Key features such as comprehensive HTTP method support, dynamic status code simulation (`/status/:code`), response delays (`/delay/:n`), and detailed request echoing (`/anything`, `/get`, `/post`, etc.) are implemented.

Furthermore, essential developer tools and production-readiness features are in place:
*   **OpenAPI/Swagger Documentation**: Interactive API documentation is available via `/swagger-ui`, with the specification at `/api-docs/openapi.json`.
*   **Docker Support**: Rucho can be easily deployed using Docker, with a non-root user for enhanced security.
*   **Configuration**: Flexible configuration options are available through files and environment variables.
*   **HTTPS**: Support for HTTPS via Rustls is implemented.
*   **CORS**: Permissive Cross-Origin Resource Sharing (CORS) is supported.

The project has successfully completed its initial core improvements and developer utility endpoint implementation phases.

## Short-Term Goals

Our immediate focus is on enhancing Rucho's robustness and deployability for production-like environments. The following features are planned for the near future:

*   **JSON Structured Server Logs**: Implement structured logging (e.g., JSON format) for easier parsing, searching, and integration with log management systems.
*   **Panic Recovery Middleware**: Introduce middleware to gracefully handle panics within request handlers, preventing the server from crashing and returning appropriate error responses.
*   **Request/Response Size Metrics**: Add functionality to track and expose metrics related to request and response sizes, which can be useful for monitoring and performance analysis.
*   **Helm Chart**: Develop a Helm chart to simplify deployment and management of Rucho on Kubernetes clusters.

## Long-Term Goals

Looking further ahead, we envision Rucho evolving into a more comprehensive and versatile HTTP toolkit. Potential future developments include:

*   **Expanded Protocol Support**:
    *   WebSocket echo support for testing real-time applications.
    *   gRPC echo server capabilities.
*   **New Utility Endpoints**:
    *   `/uuid`: Generate and return random UUIDs.
    *   `/ip`: Return the requester's IP address.
    *   `/user-agent`: Echo the User-Agent header from the request.
    *   `/headers`: Specifically echo back all request headers.
    *   `/redirect/:n`: Simulate chained HTTP redirects.
    *   `/stream/:n`: Stream multiple JSON objects in the response.
*   **Enhanced Observability & Monitoring**:
    *   Expose a `/metrics` endpoint for Prometheus to scrape, providing detailed operational metrics.
*   **Development & Deployment Automation**:
    *   Set up GitHub Actions for robust CI/CD automation (testing, building, releasing).
    *   Provide Terraform scripts for provisioning cloud infrastructure to run Rucho.
*   **Advanced Request Handling**:
    *   Implement request size limiting to prevent abuse or oversized payloads.
    *   Add rate limiting middleware.
    *   More granular CORS configuration options.
*   **Security & Configuration**:
    *   Introduce authentication/authorization middleware (e.g., JWT, Basic Auth, or OAuth2).
    *   Support for environment-based configuration using `.env` files.
*   **Advanced Features**:
    *   A request replay feature to re-send captured requests.
    *   A plugin system (e.g., using Lua or Wasm) to allow users to extend Rucho's functionality with custom logic.

## Contributing

We welcome contributions from the community! Whether it's reporting bugs, suggesting new features, or submitting code changes, your help is appreciated. Please feel free to fork the repository, create a new branch for your changes, and submit a pull request. If you have ideas or questions, don't hesitate to open an issue.

## License

Rucho is licensed under the MIT License. You can find the full license text in the `LICENSE.md` file in the repository.
