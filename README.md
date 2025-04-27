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
│   ├── delete.rs
│   ├── get.rs
│   ├── options.rs
│   ├── patch.rs
│   ├── post.rs
│   ├── put.rs
│   └── status.rs
└── utils/
    └── json_response.rs  # Shared JSON response helper
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
| GET      | `/`                | Welcome message ("Hello, World!")                |
| GET      | `/get`             | Echo request headers as JSON                    |
| POST     | `/post`            | Echo request body as JSON                       |
| PUT      | `/put`             | Echo request body as JSON                       |
| PATCH    | `/patch`           | Echo request body as JSON                       |
| DELETE   | `/delete`          | Echo request body as JSON                       |
| OPTIONS  | `/options`         | Returns allowed HTTP methods                   |
| GET      | `/status/:code`    | Responds with requested HTTP status code        |

---

## 🧹 Features

- 📜 Clean JSON response formatting with newline.
- 📈 Automatic request tracing and logging using `TraceLayer`.
- 🔥 Support for all major HTTP methods (GET, POST, PUT, PATCH, DELETE, OPTIONS).
- ⚡ Dynamic HTTP status simulation (`/status/200`, `/status/503`, etc).
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

## 📝 Notes

- `TraceLayer` provides request/response logging automatically.
- JSON formatting is consistent across all echo endpoints.
- `.gitignore` excludes `target/`, `*.rs.bk` backups, and `Cargo.lock`.
- Project structure follows best practices for Rust Axum services.

---

## 📢 License

This project is licensed under the [MIT License](LICENSE).

---
