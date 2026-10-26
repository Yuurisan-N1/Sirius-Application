<div align="center">

<img width="100%" alt="header" src="https://capsule-render.vercel.app/api?type=waving&height=210&text=Sirius%20Bot&fontAlign=50&fontAlignY=36&fontSize=56&desc=Mining%20%7C%20Quests%20%7C%20Inviter%20Bind%20%7C%20Auto%20Enroll%20%7C%20Multi-Account&descAlign=50&descAlignY=58"/>

<img alt="typing" src="https://readme-typing-svg.demolab.com?font=Inter&size=18&duration=3000&pause=650&center=true&vCenter=true&width=900&lines=Identity-Based+Auth+%7C+No+Email+No+Password;Auto+Enroll+%7C+Create+Fresh+Identities+in+Bulk;Auto+Bind+Inviter+%7C+Required+Before+Mining;Auto+Claim+Quests+%7C+All+Open+Tasks;Auto+Mining+%7C+Open+Round+When+None+Runs;Proxy+Support+%7C+Multi-Account"/>

<p>
  <img alt="rust" src="https://img.shields.io/badge/Rust-2021-f74c00?logo=rust&logoColor=white"/>
  <img alt="platform" src="https://img.shields.io/badge/Platform-Sirius%20App-111111"/>
  <img alt="multi-account" src="https://img.shields.io/badge/Multi--Account-Supported-111111"/>
  <img alt="build" src="https://github.com/Yuurisan-N1/Sirius-Application/actions/workflows/build.yml/badge.svg"/>
  <img alt="author" src="https://img.shields.io/badge/by-Yuurisandesu-111111"/>
</p>

<p>
  <b>Sirius Bot</b> is a mining, mission and referral automation client for the Sirius app.<br/>
  Sirius accounts have no email and no password. Each account is a 64-character identity string. The bot can create identities in bulk with the enroll command, or you can copy them from the app. On every cycle it signs in, reads the profile, binds the inviter when missing, claims every open quest, and opens a mining round when none is running, all across multiple accounts with proxy support.<br/>
  Built and distributed by <b>Yuurisandesu</b>.
</p>

</div>

---

## Table of Contents

- [Requirements](#requirements)
- [Install Rust](#install-rust)
- [Installation](#installation)
- [Configuration](#configuration)
- [Getting Identities](#getting-identities)
- [First-Time Setup](#first-time-setup)
- [Running the Bot](#running-the-bot)
- [Run Modes](#run-modes)
- [Download Prebuilt Binary](#download-prebuilt-binary)
- [Features](#features)
- [Troubleshooting](#troubleshooting)
- [File Structure](#file-structure)
- [Disclaimer](#disclaimer)

---

## Requirements

- Rust `1.70+` (includes `cargo`, only needed if building from source)
- Git

---

## Install Rust

### Linux / macOS

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Verify:

```bash
rustc --version
```

### Windows

Download and run the installer from https://rustup.rs, then reopen the terminal and verify:

```cmd
rustc --version
```

### Termux (Android)

```bash
pkg update && pkg install proot-distro
proot-distro install ubuntu
proot-distro login ubuntu
```

Then inside Ubuntu:

```bash
apt update && apt install -y curl git build-essential pkg-config libssl-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

---

## Installation

**Clone the repository:**

```bash
git clone https://github.com/Yuurisan-N1/Sirius-Application.git
cd Sirius-Application
```

**Build the binary:**

```bash
cargo build --release
# or
./run.sh build
```

---

## Configuration

### 1. Identities (data.txt)

Sirius accounts have no email and no password. Each account is a 64-character identity string. Fill `data.txt` with one identity per line. Blank lines and lines starting with `#` are ignored. Keep the file private:

```
a1b2c3d4e5f6...64characteridentitystringhere
another64characteridentitystringhere...
```

```bash
chmod 600 data.txt
```

See [Getting Identities](#getting-identities) below for how to obtain them.

### 2. Referral Code (config.json)

Mining is refused by the server until an inviter is bound to the account. Open `config.json` and set `referral_code` to your own user code. Your code is on the Profile screen of the Sirius app:

```json
{
  "settings": {
    "sleep_seconds": 3600,
    "referral_code": "YourCode"
  }
}
```

Leaving `referral_code` empty causes the bot to skip the inviter step, and the server will refuse to open a mining round.

### 3. Proxy (proxy.txt)

Fill `proxy.txt` with proxies, one per line, in the same order as `data.txt` (optional, leave empty to run without proxy):

```
http://user:pass@host:port
socks5://user:pass@host:port
```

---

## Getting Identities

There are two ways to get identity strings for `data.txt`.

**Option A - Copy from the app:**

Open the Sirius app on your phone, go to Profile, then Identity Backup, and copy the 64-character string. Paste it into `data.txt`.

**Option B - Auto-create with enroll:**

The enroll command registers fresh identities automatically and appends them to `data.txt`. Build the binary first, then run:

```bash
./run.sh enroll 3
```

Replace `3` with however many identities you want to create. The bot asks a human check solver for a token, registers each identity, and appends the strings to `data.txt`. The installation cache for each new identity is saved to `state/` automatically.

---

## First-Time Setup

Before running the main loop, verify the setup looks correct:

```bash
./run.sh probe
```

This reads the registration config and the region answer without claiming anything. When the output looks right, start the loop.

---

## Running the Bot

Make `run.sh` executable first:

```bash
chmod +x run.sh
```

Then choose a mode:

```bash
./run.sh run            # build if needed and run in foreground
./run.sh bg             # build if needed and run in background, logs saved to sirius-bot.log
./run.sh screen         # build if needed and run in a detached screen session
./run.sh stop           # stop the background or screen session
./run.sh probe          # read registration config and region without claiming
./run.sh selftest       # offline checks, no network
./run.sh enroll <count> # create and register fresh identities
./run.sh build          # build only, do not run
```

For background mode, tail the log anytime:

```bash
tail -f sirius-bot.log
```

For screen mode, attach anytime:

```bash
screen -r sirius-bot
```

Press `Ctrl+C` once to stop the bot cleanly when running in foreground.

### Using make (Linux / macOS)

```bash
make release    # optimized release build
make start      # build release and run immediately
make run        # build debug and run
make check      # check for errors without building
make fmt        # format the code
make clean      # remove all build artifacts
make size       # show release binary size
```

### Manual run

```bash
# Linux / macOS / Termux
./target/release/sirius-bot

# Windows
.\target\release\sirius-bot.exe
```

---

## Run Modes

| Mode | Command | Description |
|---|---|---|
| run | `./run.sh run` | Full cycle: sign in, inviter, quests, mining, sleep |
| bg | `./run.sh bg` | Same as run in background with log file |
| bgprobe | `./run.sh bgprobe` | Probe in background |
| screen | `./run.sh screen` | Same as run in a detached screen session |
| probe | `./run.sh probe` | Read registration config and region without claiming |
| selftest | `./run.sh selftest` | Offline checks, no network |
| enroll | `./run.sh enroll <count>` | Create and register N fresh identities |
| stop | `./run.sh stop` | Stop background or screen session |
| build | `./run.sh build` | Build only |

---

## Download Prebuilt Binary

Prebuilt binaries are automatically compiled on every push to main via GitHub Actions across 3 platforms.

Download the latest binaries from the Actions page:
https://github.com/Yuurisan-N1/Sirius-Application/actions/workflows/build.yml

Open the latest successful run and scroll to the Artifacts section.

| Artifact | Platform |
|---|---|
| `sirius-bot-linux-x86_64` | Linux x86_64 |
| `sirius-bot-windows-x86_64` | Windows x86_64 |
| `sirius-bot-macos-aarch64` | macOS Apple Silicon |

**Linux / macOS after downloading:**

```bash
chmod +x sirius-bot-linux-x86_64
./sirius-bot-linux-x86_64
```

**Windows:**

```bash
.\sirius-bot-windows-x86_64.exe
```

Place the binary in the same folder as `data.txt`, `proxy.txt`, and `config.json` before running.

---

## Features

### Identity-Based Authentication
Sirius uses a 64-character identity string instead of email and password. Each identity is a device-bound key. The bot generates a fresh installation ID on first use per identity and caches it in `state/` (mode 0600, gitignored). Tokens are never printed to the log. Delete the `state/` folder to force a fresh installation ID.

### Auto Enroll
The enroll command creates and registers fresh identities in bulk. It requests a human check token, registers each identity with the server, and appends the identity strings to `data.txt` automatically.

### Inviter Bind
Mining rounds cannot be opened until an inviter code is bound to the account. The bot checks whether the inviter is already set on each account and binds the code from `referral_code` in `config.json` when it is missing. Accounts that already have an inviter skip this step silently.

### Auto Quests
The bot fetches all open quests for each account and claims them in order. Each claimed quest logs its reward. The quest list is empty between settlement intervals and the bot skips the step with a note.

### Auto Mining
The bot checks whether a mining round is currently running. If no round is active, it opens one. The mining length, settlement interval, and reward rate are read live from the server and are not configurable locally. If a round is already running, the remaining time is logged and mining is left to continue.

### Installation Cache
After the first enroll or sign-in per identity, the installation ID and session state are saved to `state/`. Subsequent runs load from the cache and skip re-registration. Deleting a file in `state/` forces a fresh start for that identity.

### Multi Account
All identities in `data.txt` are processed sequentially within every cycle. Each identity uses its own proxy from the pool and its own cache file in `state/`.

### Proxy Support
Proxies are loaded from `proxy.txt` and paired to identities by index. HTTP and SOCKS5 are both supported. Proxy credentials are masked in log output. Running without proxies is fully supported.

### Auto Countdown
After all accounts complete a cycle, the bot displays a live `HH:MM:SS` countdown until the next cycle starts.

---

## Troubleshooting

| Message | Cause | Fix |
|---|---|---|
| `the identity is 32 characters instead of 64` | The line in `data.txt` is not a full identity | Copy the full 64-character string from the app |
| `the server refused the identity` | The identity does not exist on the server | Run `./run.sh enroll 1` or copy from the app |
| `the region is closed` | The IP address is blocked | Use a proxy |
| `the inviter was skipped` | The account already has an inviter bound | No action needed |
| `the server asks every new identity for a human check` | Enroll requires a solver token | The solver must be reachable |
| `the solver could not be reached` | The human check solver is down | Retry later |

---

## File Structure

```text
Sirius-Application/
├── .github/
│   └── workflows/
│       └── build.yml                    # CI: 3-platform matrix build
├── src/
│   ├── main.rs                          # Entry point, Ctrl+C handler, mode dispatch
│   ├── cli/
│   │   ├── dispatch.rs                  # Mode selector
│   │   ├── enroll.rs                    # Enroll CLI: create and register identities
│   │   ├── probe.rs                     # Probe CLI: read without claiming
│   │   ├── selftest.rs                  # Offline checks
│   │   └── mod.rs
│   ├── job/
│   │   ├── account.rs                   # Account model: identity string, installation ID
│   │   ├── loop.rs                      # Multi-account loop and countdown
│   │   └── mod.rs
│   ├── net/
│   │   ├── client.rs                    # HTTP client with cookie jar
│   │   ├── egress.rs                    # Proxy pool loader and masker
│   │   ├── gate.rs                      # Gate request builder
│   │   └── mod.rs
│   ├── step/
│   │   ├── cycle.rs                     # Per-account cycle: inviter, quests, mining
│   │   ├── enroll.rs                    # Identity registration and cache write
│   │   ├── invite.rs                    # Inviter bind
│   │   ├── quest.rs                     # Quest list fetch and claim
│   │   └── mod.rs
│   ├── support/
│   │   ├── banner.rs                    # ASCII banner
│   │   ├── cfg/                         # Config loader and schema
│   │   ├── consts/                      # App and network constants
│   │   ├── error.rs                     # Error types
│   │   ├── log/                         # Log init, colored format, countdown helper
│   │   └── mod.rs
│   └── wire/
│       ├── endpoint.rs                  # API endpoint definitions
│       ├── seal.rs                      # Request seal and encryption (RSA + AES-GCM)
│       ├── shape.rs                     # Request and response shape definitions
│       └── mod.rs
├── assets/
│   ├── icon.ico                         # Windows binary icon
│   └── icon.png                         # App icon
├── state/                               # Installation ID and session cache per identity (auto-generated, gitignored)
├── Cargo.toml                           # Project manifest and dependencies
├── Cargo.lock
├── build.rs                             # Build script (embeds icon into Windows binary)
├── Makefile                             # make targets: build, run, release, start, clean, size
├── run.sh                               # Run helper: run, bg, screen, probe, enroll, selftest, stop, build
├── config.json                          # Sleep duration and referral code
├── data.txt                             # Identity strings, one per line
├── proxy.txt                            # Proxy list (optional)
├── HOWTO.txt                            # Quick start guide
└── LICENSE                              # GNU AGPL v3.0
```

---

## Disclaimer

This tool is built for educational and technical exploration purposes. Use it wisely and at your own responsibility.

---

<div align="center">
<img width="100%" alt="footer" src="https://capsule-render.vercel.app/api?type=waving&height=120&section=footer"/>
</div>