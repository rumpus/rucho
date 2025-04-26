# Echo Server (Rust, Axum)

Simple HTTP echo server built using Rust and Axum.  
It demonstrates basic request handling, header echoing, and request tracing.

## 🛠 Tech Stack

- [Rust](https://www.rust-lang.org/)
- [Axum](https://docs.rs/axum/latest/axum/)
- [Tokio](https://tokio.rs/)
- [Tower-HTTP](https://docs.rs/tower-http/latest/tower_http/)
- [Hyper](https://hyper.rs/)

## 📂 Project Structure

```bash
src/
├── main.rs          # Application entrypoint
├── lib.rs           # Library module definitions
└── handlers/
    └── echo.rs      # HTTP request handlers
```

## 🚀 Getting Started

1. **Clone the repository**
   ```bash
   git clone <your-repo-url>
   cd echo-server
   ```

2. **Build the project**
   ```bash
   cargo build
   ```

3. **Run the server**
   ```bash
   cargo run
   ```

Server will start on:

```
http://localhost:8080
```

## 📜 Available Endpoints

| Method | Path   | Description                     |
|:------:|:------:|:--------------------------------:|
| GET    | `/`    | Returns a simple welcome message |
| GET    | `/get` | Echoes back request headers as JSON |


## 📝 Notes

- `TraceLayer` is applied for basic request logging.
- `serde` and `serde_json` are used to format JSON responses.
- `.gitignore` excludes `target/`, `*.rs.bk` backup files, and `Cargo.lock`.


## 📢 License

This project is open-sourced under the MIT License.
