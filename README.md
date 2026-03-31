# Secure Web Gateway

## Overview
**Secure Web Gateway** is a desktop-based browser wrapper designed to enforce navigation security policies and restrict unsafe web behavior. The project focuses on implementing a controlled browsing environment with URL validation, sandboxed navigation, and protocol filtering.

The gateway intercepts navigation requests, validates URLs, and enforces a whitelist-based sandbox policy before allowing content to load.

> [!IMPORTANT]
> This project is currently under active development. The current implementation focuses on secure navigation handling and protocol enforcement.

---

## Current Features

### 1. Controlled Navigation Pipeline
Navigation requests follow a structured validation flow:
**User Input** → **Input Sanitization** → **IPC Request** → **URL Resolution** → **Sandbox Policy Check** → **WebView Navigation**

All navigation attempts pass through this pipeline before rendering.

### 2. Custom IPC Navigation Channel
The gateway uses a controlled IPC endpoint: `http://ipc.localhost/navigate`. This isolates frontend input from backend navigation logic and prevents direct navigation from JavaScript.

**Supported Commands:**
* `Maps`
* `ping`
* *Unknown commands are rejected.*

### 3. URL Resolution and Normalization
User input is processed using structured URL parsing:
* Accepts valid `HTTP`/`HTTPS` URLs.
* Converts bare domains to `HTTPS`.
* Converts invalid input into search queries.
* Blocks unsupported schemes.

| Input | Result |
| :--- | :--- |
| `example.com` | `https://example.com` |
| `javascript:alert(1)` | Search query |
| `invalid input` | Search query |

### 4. Navigation Sandbox
Navigation is restricted using a whitelist-based scheme filter.

* **Allowed Schemes:** `http`, `https`, `ws`, `wss`, `blob`, `about`, `asset`, `ipc`
* **Restricted / Blocked Schemes:** `javascript`, `file`, `ftp`, `view-source`, `mailto`, `tel`, `sms`, `geo`

*Unknown schemes are denied by default.*

### 5. Navigation Interception
All navigation attempts are intercepted, including:
* `iframe` navigation
* Redirects
* JavaScript-driven navigation
* Manual user navigation

*Blocked navigation loads a local **blocked page**.*

### 6. Popup Control
New window requests are intercepted and validated before being allowed. Blocked popups are denied and redirected to a blocked page.

### 7. Download Restrictions
Download requests are restricted based on scheme:
* **Allowed:** `https`, `blob`
* **Blocked:** `http`, `file`, `ftp`

### 8. Custom Asset Protocol
Local UI files are served through a custom protocol: `asset://`.
* **Allowed assets:** `index.html`, `blocked.html`, `favicon.ico`
* **Other files:** Rejected.

### 9. Content Security Policy (CSP)
The gateway applies CSP headers to restrict resource loading:
* Restricts script execution.
* Controls frame sources.
* Limits resource loading.

### 10. Event-Based Navigation Control
Navigation is handled through event-based messaging between the **Frontend**, **IPC handler**, and **WebView controller**. This separates UI logic from navigation execution.

---

## Architecture

```text
main.rs
 ├── Event loop
 ├── Navigation events
 └── WebView injection

webview.rs
 ├── Custom protocols
 ├── Navigation handlers
 ├── Download handlers
 └── Asset serving

navigation.rs
 └── URL parsing and normalization

sandbox.rs
 └── Navigation policy enforcement

index.html
 └── UI and IPC trigger
```

---

## Technologies Used
* **Rust** (Core Logic)
* **Wry** (WebView rendering)
* **Tao** (Window/Event Loop management)
* **HTML / JavaScript** (UI)
* **url** (Crate for parsing)
* **percent-encoding** (Crate for URI safety)

---

## Security Controls Implemented
* Input sanitization
* URL normalization
* Scheme whitelist enforcement
* Navigation interception
* Popup blocking
* Download restriction
* Custom protocol handling
* Content Security Policy
* Event-based navigation control

---

## Tests
Unit tests are implemented for:
* URL resolution logic
* Scheme filtering
* Blocked navigation cases
* Invalid input handling

---

## Current Limitations
The project is still under development. The following features are not yet implemented:
* Domain allow/block lists
* Certificate validation controls
* Request filtering
* Network policy enforcement
* Navigation history
* Permission handling

---

## Future Work
* Domain filtering
* Policy configuration
* Logging and auditing
* Navigation isolation
* Security telemetry
* Per-site permission controls

---

## Running the Project

### Requirements
* Rust (stable)
* Cargo

### Build & Run
```bash
# Build the project
cargo build

# Run the project
cargo run

# as ipc is currently blocked by chromium and is yet to be resolved,
# can test the navigation and sandbox rules using
cargo test
```

---

## Project Status
* **Status:** Active development
* **Current Focus:** Secure navigation enforcement and sandbox policy implementation.

## Purpose
This project is intended as a secure browsing gateway to study:
1.  Navigation security
2.  Browser isolation concepts
3.  Protocol filtering
4.  Secure desktop webview architecture
