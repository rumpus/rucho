# Usage Examples

Practical examples for testing with rucho using **curl**, **Python** (`requests`), and **JavaScript** (`fetch`).

All examples assume rucho is running at `http://localhost:8080` (the default).

## Prerequisites

- **curl** (included on most systems)
- **Python**: `pip install requests`
- **Node.js**: `fetch` is built-in (v18+)

---

## Table of Contents

- [Basic Echo Endpoints](#basic-echo-endpoints)
- [Request Inspection](#request-inspection)
- [Status Code Testing](#status-code-testing)
- [Wildcard Endpoint](#wildcard-endpoint)
- [Redirect Testing](#redirect-testing)
- [Delay & Timeout Testing](#delay--timeout-testing)
- [Cookie Management](#cookie-management)
- [Chaos Engineering](#chaos-engineering)
- [Health Checks & Monitoring](#health-checks--monitoring)

---

## Basic Echo Endpoints

### GET /get

Inspect your request details — method, headers, and timing.

**curl:**

```bash
curl http://localhost:8080/get
```

**Python:**

```python
import requests

resp = requests.get("http://localhost:8080/get")
print(resp.json())
```

**JavaScript:**

```javascript
const resp = await fetch("http://localhost:8080/get");
const data = await resp.json();
console.log(data);
```

**Sample response:**

```json
{
  "method": "GET",
  "headers": {
    "host": "localhost:8080",
    "user-agent": "curl/8.7.1",
    "accept": "*/*"
  },
  "timing": {
    "duration_ms": 0.123
  }
}
```

### POST /post

Echo a JSON body back.

**curl:**

```bash
curl -X POST http://localhost:8080/post \
  -H "Content-Type: application/json" \
  -d '{"username": "alice", "role": "admin"}'
```

**Python:**

```python
import requests

resp = requests.post("http://localhost:8080/post", json={
    "username": "alice",
    "role": "admin"
})
print(resp.json())
```

**JavaScript:**

```javascript
const resp = await fetch("http://localhost:8080/post", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ username: "alice", role: "admin" }),
});
const data = await resp.json();
console.log(data);
```

**Sample response:**

```json
{
  "method": "POST",
  "headers": {
    "host": "localhost:8080",
    "content-type": "application/json",
    "content-length": "38"
  },
  "body": {
    "username": "alice",
    "role": "admin"
  },
  "timing": {
    "duration_ms": 0.456
  }
}
```

### PUT /put and PATCH /patch

Same structure as POST — body is echoed back.

**curl:**

```bash
# PUT
curl -X PUT http://localhost:8080/put \
  -H "Content-Type: application/json" \
  -d '{"id": 42, "name": "updated"}'

# PATCH
curl -X PATCH http://localhost:8080/patch \
  -H "Content-Type: application/json" \
  -d '{"name": "patched"}'
```

**Python:**

```python
import requests

# PUT
resp = requests.put("http://localhost:8080/put", json={"id": 42, "name": "updated"})
print(resp.json())

# PATCH
resp = requests.patch("http://localhost:8080/patch", json={"name": "patched"})
print(resp.json())
```

### DELETE /delete

Returns method, headers, and body (body is `null` when none is sent).

**curl:**

```bash
# Without body
curl -X DELETE http://localhost:8080/delete

# With body
curl -X DELETE http://localhost:8080/delete \
  -H "Content-Type: application/json" \
  -d '{"id": 42}'
```

**Python:**

```python
import requests

resp = requests.delete("http://localhost:8080/delete")
data = resp.json()
print(data["body"])  # null
```

---

## Request Inspection

### GET /headers

Send custom headers and see them reflected.

**curl:**

```bash
curl http://localhost:8080/headers \
  -H "X-Custom-Header: my-value" \
  -H "Authorization: Bearer token123"
```

**Python:**

```python
import requests

resp = requests.get("http://localhost:8080/headers", headers={
    "X-Custom-Header": "my-value",
    "Authorization": "Bearer token123"
})
for name, value in resp.json()["headers"].items():
    print(f"{name}: {value}")
```

**JavaScript:**

```javascript
const resp = await fetch("http://localhost:8080/headers", {
  headers: {
    "X-Custom-Header": "my-value",
    "Authorization": "Bearer token123",
  },
});
const data = await resp.json();
console.log(data.headers);
```

**Sample response:**

```json
{
  "headers": {
    "host": "localhost:8080",
    "x-custom-header": "my-value",
    "authorization": "Bearer token123",
    "accept": "*/*"
  },
  "timing": {
    "duration_ms": 0.089
  }
}
```

### GET /user-agent

Test how your client identifies itself.

**curl:**

```bash
curl http://localhost:8080/user-agent -H "User-Agent: MyApp/2.0"
```

**Python:**

```python
import requests

resp = requests.get("http://localhost:8080/user-agent", headers={
    "User-Agent": "MyApp/2.0"
})
print(resp.json())
# {"user-agent": "MyApp/2.0", "timing": {"duration_ms": 0.1}}
```

### GET /ip

Client IP detection. Respects `X-Forwarded-For` and `X-Real-IP` for proxy scenarios.

**curl:**

```bash
# Direct connection
curl http://localhost:8080/ip

# Simulating a request through a proxy
curl http://localhost:8080/ip -H "X-Forwarded-For: 203.0.113.50"
```

**Python:**

```python
import requests

# Simulate proxy forwarding
resp = requests.get("http://localhost:8080/ip", headers={
    "X-Forwarded-For": "203.0.113.50"
})
print(resp.json())
# {"origin": "203.0.113.50", "timing": {"duration_ms": 0.1}}
```

### GET /uuid

Generate a unique request ID.

**curl:**

```bash
curl http://localhost:8080/uuid
```

**Sample response:**

```json
{
  "uuid": "550e8400-e29b-41d4-a716-446655440000",
  "timing": {
    "duration_ms": 0.05
  }
}
```

---

## Status Code Testing

Test how your client handles different HTTP status codes.

### Basic usage

**curl:**

```bash
# Success
curl -i http://localhost:8080/status/200

# Not Found
curl -i http://localhost:8080/status/404

# I'm a Teapot
curl -i http://localhost:8080/status/418

# Service Unavailable
curl -i http://localhost:8080/status/503
```

**Python:**

```python
import requests

for code in [200, 404, 418, 503]:
    resp = requests.get(f"http://localhost:8080/status/{code}")
    print(f"/status/{code} → {resp.status_code}")
```

**JavaScript:**

```javascript
for (const code of [200, 404, 418, 503]) {
  const resp = await fetch(`http://localhost:8080/status/${code}`);
  console.log(`/status/${code} → ${resp.status}`);
}
```

### Scenario: testing error handling in a client library

```python
import requests

def fetch_data(url):
    """Example client function with error handling."""
    resp = requests.get(url)
    resp.raise_for_status()
    return resp.text

# Test that your client raises on 5xx errors
try:
    fetch_data("http://localhost:8080/status/503")
except requests.HTTPError as e:
    print(f"Caught expected error: {e.response.status_code}")
```

---

## Wildcard Endpoint

`/anything` echoes any method, path, query string, headers, and body.

**curl:**

```bash
curl -X POST "http://localhost:8080/anything/my/custom/path?debug=true&level=5" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer secret" \
  -d '{"action": "test"}'
```

**Python:**

```python
import requests

resp = requests.post(
    "http://localhost:8080/anything/my/custom/path",
    params={"debug": "true", "level": "5"},
    headers={"Authorization": "Bearer secret"},
    json={"action": "test"}
)
data = resp.json()
print(f"Method: {data['method']}")
print(f"Path:   {data['path']}")
print(f"Query:  {data['query']}")
print(f"Body:   {data['body']}")
```

**JavaScript:**

```javascript
const resp = await fetch(
  "http://localhost:8080/anything/my/custom/path?debug=true&level=5",
  {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: "Bearer secret",
    },
    body: JSON.stringify({ action: "test" }),
  }
);
const data = await resp.json();
console.log(data);
```

**Sample response:**

```json
{
  "method": "POST",
  "path": "/anything/my/custom/path",
  "query": "debug=true&level=5",
  "headers": {
    "host": "localhost:8080",
    "content-type": "application/json",
    "authorization": "Bearer secret"
  },
  "body": "{\"action\":\"test\"}",
  "timing": {
    "duration_ms": 0.234
  }
}
```

### Scenario: testing a client that hits arbitrary paths

```python
import requests

# Your API client under test might construct URLs dynamically
base = "http://localhost:8080/anything"
endpoints = ["/users/123", "/orders/456/items", "/search?q=rucho"]

for path in endpoints:
    resp = requests.get(f"{base}{path}")
    data = resp.json()
    print(f"{data['path']} (query: {data['query'] or 'none'})")
```

---

## Redirect Testing

### Follow a redirect chain

**curl:**

```bash
# Follow all redirects (-L) through a 3-hop chain
curl -L http://localhost:8080/redirect/3

# See each redirect step (-v shows headers)
curl -v http://localhost:8080/redirect/3
```

**Python:**

```python
import requests

# Follow redirects (default behavior)
resp = requests.get("http://localhost:8080/redirect/3")
print(f"Final URL: {resp.url}")
print(f"Status: {resp.status_code}")
print(f"Hops: {len(resp.history)}")
for i, r in enumerate(resp.history):
    print(f"  Hop {i+1}: {r.status_code} → {r.headers['Location']}")
```

**JavaScript:**

```javascript
// Follow redirects (default behavior)
const resp = await fetch("http://localhost:8080/redirect/3", {
  redirect: "follow",
});
console.log(`Final URL: ${resp.url}`);
console.log(`Redirected: ${resp.redirected}`);
```

### Single redirect

A single redirect (`/redirect/1`) goes straight to `/get`:

```bash
curl -L http://localhost:8080/redirect/1
# Returns the /get JSON response after one 302 hop
```

### Scenario: testing client redirect limits

Rucho supports up to 20 redirect hops. Test how your client handles long chains:

```python
import requests

# Most HTTP clients default to a max of 30 redirects
# Test with a long chain
resp = requests.get("http://localhost:8080/redirect/15")
print(f"Followed {len(resp.history)} redirects")

# Disable redirects to inspect the raw 302
resp = requests.get("http://localhost:8080/redirect/3", allow_redirects=False)
print(f"Status: {resp.status_code}")
print(f"Location: {resp.headers['Location']}")
# Status: 302
# Location: /redirect/2
```

---

## Delay & Timeout Testing

### Basic delay

**curl:**

```bash
# 5-second delay
curl http://localhost:8080/delay/5
# Response: "Response delayed by 5 seconds"
```

**Python:**

```python
import requests

resp = requests.get("http://localhost:8080/delay/2")
print(resp.text)  # "Response delayed by 2 seconds"
```

### Scenario: test that a client timeout fires correctly

Set a short timeout and verify it triggers against a slow endpoint:

**Python:**

```python
import requests

# Client timeout (2s) is shorter than the delay (5s) → should fail
try:
    resp = requests.get("http://localhost:8080/delay/5", timeout=2)
except requests.Timeout:
    print("Timeout fired as expected!")

# Client timeout (10s) is longer than the delay (3s) → should succeed
resp = requests.get("http://localhost:8080/delay/3", timeout=10)
print(f"Success: {resp.text}")
```

**JavaScript:**

```javascript
// AbortController with a 2-second timeout against a 5-second delay
const controller = new AbortController();
const timeoutId = setTimeout(() => controller.abort(), 2000);

try {
  await fetch("http://localhost:8080/delay/5", {
    signal: controller.signal,
  });
} catch (err) {
  console.log("Timeout fired as expected:", err.name); // "AbortError"
} finally {
  clearTimeout(timeoutId);
}
```

**curl:**

```bash
# curl --max-time sets the total timeout in seconds
curl --max-time 2 http://localhost:8080/delay/5
# curl: (28) Operation timed out

curl --max-time 10 http://localhost:8080/delay/3
# Response delayed by 3 seconds
```

---

## Cookie Management

### Set and inspect cookies

**curl:**

```bash
# Set cookies via query params (follows redirect to /cookies)
curl -L -c - http://localhost:8080/cookies/set?session=abc123&theme=dark
```

**Sample response (after redirect to /cookies):**

```json
{
  "cookies": {
    "session": "abc123",
    "theme": "dark"
  },
  "timing": {
    "duration_ms": 0.15
  }
}
```

### Full cookie roundtrip: set → inspect → delete → verify

**curl:**

```bash
# 1. Set cookies and save to a cookie jar file
curl -c cookies.txt -L http://localhost:8080/cookies/set?session=abc123&theme=dark

# 2. Inspect cookies using the jar
curl -b cookies.txt http://localhost:8080/cookies

# 3. Delete the "theme" cookie
curl -b cookies.txt -c cookies.txt -L http://localhost:8080/cookies/delete?theme

# 4. Verify it's gone
curl -b cookies.txt http://localhost:8080/cookies
# {"cookies": {"session": "abc123"}, ...}
```

**Python:**

```python
import requests

# Use a session to automatically manage cookies
session = requests.Session()

# 1. Set cookies (follows redirect to /cookies automatically)
resp = session.get("http://localhost:8080/cookies/set", params={
    "session": "abc123",
    "theme": "dark"
})
print("After set:", resp.json()["cookies"])
# {"session": "abc123", "theme": "dark"}

# 2. Inspect cookies
resp = session.get("http://localhost:8080/cookies")
print("Inspect:", resp.json()["cookies"])
# {"session": "abc123", "theme": "dark"}

# 3. Delete the "theme" cookie
resp = session.get("http://localhost:8080/cookies/delete", params={"theme": ""})
print("After delete:", resp.json()["cookies"])
# {"session": "abc123"}

# 4. Verify via the session cookie jar
print("Session cookies:", dict(session.cookies))
```

**JavaScript:**

```javascript
// Note: fetch doesn't handle cookies automatically in Node.js.
// Use a library like undici or node-fetch with a cookie jar,
// or pass cookies manually:

// Set cookies and inspect the Set-Cookie headers
const setResp = await fetch(
  "http://localhost:8080/cookies/set?session=abc123&theme=dark",
  { redirect: "manual" }
);
console.log("Set-Cookie:", setResp.headers.getSetCookie());

// Send cookies back manually
const inspectResp = await fetch("http://localhost:8080/cookies", {
  headers: { Cookie: "session=abc123; theme=dark" },
});
const data = await inspectResp.json();
console.log("Cookies:", data.cookies);
```

---

## Chaos Engineering

Chaos mode injects random failures, delays, and response corruption. It's configured via environment variables when starting the server.

### Failure injection

Randomly return 500/503 on 30% of requests:

```bash
# Start rucho with chaos enabled
RUCHO_CHAOS_MODE=failure \
RUCHO_CHAOS_FAILURE_RATE=0.3 \
RUCHO_CHAOS_FAILURE_CODES=500,503 \
./target/release/rucho start
```

Test it:

```bash
# Make several requests and check for injected failures
for i in $(seq 1 10); do
  code=$(curl -s -o /dev/null -w '%{http_code}' http://localhost:8080/get)
  echo "Request $i: HTTP $code"
done
```

When a failure is injected, the response includes an `X-Chaos` header and a JSON body:

```json
{
  "error": "Chaos failure injected",
  "chaos": {
    "type": "failure",
    "status_code": 503
  }
}
```

### Delay injection

Add random delays (up to 3s) on 50% of requests:

```bash
RUCHO_CHAOS_MODE=delay \
RUCHO_CHAOS_DELAY_RATE=0.5 \
RUCHO_CHAOS_DELAY_MS=random \
RUCHO_CHAOS_DELAY_MAX_MS=3000 \
./target/release/rucho start
```

```python
import requests
import time

# Measure response times to observe injected delays
for i in range(10):
    start = time.time()
    resp = requests.get("http://localhost:8080/get")
    elapsed = time.time() - start
    chaos = resp.headers.get("x-chaos", "none")
    print(f"Request {i+1}: {elapsed:.2f}s (chaos: {chaos})")
```

### Response corruption

Truncate response bodies on 20% of requests:

```bash
RUCHO_CHAOS_MODE=corruption \
RUCHO_CHAOS_CORRUPTION_RATE=0.2 \
RUCHO_CHAOS_CORRUPTION_TYPE=truncate \
./target/release/rucho start
```

```python
import requests

for i in range(10):
    resp = requests.get("http://localhost:8080/get")
    chaos = resp.headers.get("x-chaos", "none")
    try:
        data = resp.json()
        print(f"Request {i+1}: valid JSON (chaos: {chaos})")
    except requests.exceptions.JSONDecodeError:
        print(f"Request {i+1}: corrupted response (chaos: {chaos})")
```

### Combined chaos mode

Enable all chaos types at once for thorough resilience testing:

```bash
RUCHO_CHAOS_MODE=failure,delay,corruption \
RUCHO_CHAOS_FAILURE_RATE=0.1 \
RUCHO_CHAOS_FAILURE_CODES=500,502,503 \
RUCHO_CHAOS_DELAY_RATE=0.2 \
RUCHO_CHAOS_DELAY_MS=random \
RUCHO_CHAOS_DELAY_MAX_MS=5000 \
RUCHO_CHAOS_CORRUPTION_RATE=0.05 \
RUCHO_CHAOS_CORRUPTION_TYPE=empty \
./target/release/rucho start
```

---

## Health Checks & Monitoring

### GET /healthz

Simple health check — returns `200 OK` with body `OK`.

**curl:**

```bash
curl http://localhost:8080/healthz
# OK
```

**Python:**

```python
import requests

resp = requests.get("http://localhost:8080/healthz")
assert resp.status_code == 200
assert resp.text == "OK"
```

### GET /metrics

Request statistics (must be enabled with `RUCHO_METRICS_ENABLED=true`).

**curl:**

```bash
curl http://localhost:8080/metrics
```

**Sample response:**

```json
{
  "all_time": {
    "total_requests": 150,
    "successes": 140,
    "failures": 10,
    "endpoint_hits": {
      "/get": 80,
      "/post": 40,
      "/status/:code": 20,
      "/healthz": 10
    }
  },
  "last_hour": {
    "total_requests": 50,
    "successes": 48,
    "failures": 2,
    "endpoint_hits": {
      "/get": 30,
      "/post": 15,
      "/status/:code": 5
    }
  }
}
```

### GET /endpoints

API discovery — list all available endpoints.

**curl:**

```bash
curl http://localhost:8080/endpoints
```

### Scenario: Docker/Kubernetes health check

**Docker Compose:**

```yaml
services:
  rucho:
    image: rumpus/rucho:latest
    ports:
      - "8080:8080"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/healthz"]
      interval: 10s
      timeout: 5s
      retries: 3
      start_period: 5s
```

**Kubernetes liveness/readiness probes:**

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: rucho
spec:
  containers:
    - name: rucho
      image: rumpus/rucho:latest
      ports:
        - containerPort: 8080
      livenessProbe:
        httpGet:
          path: /healthz
          port: 8080
        initialDelaySeconds: 3
        periodSeconds: 10
      readinessProbe:
        httpGet:
          path: /healthz
          port: 8080
        initialDelaySeconds: 3
        periodSeconds: 5
```
