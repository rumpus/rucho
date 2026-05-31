# Security Policy

## Scope

Rucho is an HTTP echo server and request inspector built for **testing and
debugging** — and as a controllable upstream behind Kong Gateway / Kong Mesh.
It is **not hardened for production**, is not intended to serve untrusted public
traffic, and should not handle secrets. Run it in trusted test / CI / lab
environments.

Some deliberate behaviors follow from that purpose and are **features, not
vulnerabilities**: it echoes request data back verbatim, can be told to inject
failures / delays / corruption (chaos mode, off by default), and can emit
arbitrary response headers (`/response-headers`). Don't expose an instance with
those enabled to untrusted callers.

## Supported versions

This is a single-maintainer project; only the **latest release** receives fixes.

| Version        | Supported |
|----------------|-----------|
| latest release | ✅        |
| older          | ❌        |

## Reporting a vulnerability

Please report security issues **privately** via GitHub's
[private vulnerability reporting](https://github.com/rumpus/rucho/security/advisories/new)
— the **"Report a vulnerability"** button on the repository's **Security** tab —
rather than opening a public issue.

Acknowledgement is best-effort: this is a spare-time project, not a product with
a response SLA.
