<div align="center">

<img width="100%" alt="header" src="https://capsule-render.vercel.app/api?type=waving&height=210&text=Sirius&fontAlign=50&fontAlignY=36&fontSize=56&desc=Identity%20%7C%20Mining%20Round%20%7C%20Task%20Board%20%7C%20Rust%20CLI&descAlign=50&descAlignY=58"/>

<img alt="typing" src="https://readme-typing-svg.demolab.com?font=Inter&size=18&duration=3000&pause=650&center=true&vCenter=true&width=900&lines=Identity+Only%2C+No+Email%2C+No+Password;Mining+Round+Reopened+When+It+Ends;Every+Task+Claimed+In+One+Pass;Multi+Account+%7C+Proxy+Support"/>

<p>
  <img alt="rust" src="https://img.shields.io/badge/Rust-2021-f74c00?logo=rust&logoColor=white"/>
  <img alt="platform" src="https://img.shields.io/badge/Platform-Sirius-111111"/>
  <img alt="wire" src="https://img.shields.io/badge/Wire-JWE%20Envelope-111111"/>
  <img alt="build" src="https://github.com/Yuurisan-N1/Sirius/actions/workflows/build.yml/badge.svg"/>
  <img alt="author" src="https://img.shields.io/badge/by-Yuurisandesu-111111"/>
</p>

<p>
  <b>Sirius</b> is a CLI client for the Sirius mining app.<br/>
  It signs in with the identity string the app stores on the phone, seals every request in the JWE envelope the app uses, then walks the account every cycle: profile, inviter, task board, mining round.<br/>
  Built and distributed by <b>Yuurisandesu</b>.
</p>

</div>

---

## Table of Contents

- [What It Does](#what-it-does)
- [Requirements](#requirements)
- [Install](#install)
- [Identity File](#identity-file)
- [First Run](#first-run)
- [Modes](#modes)
- [Configuration](#configuration)
- [Proxy](#proxy)
- [Installation Cache](#installation-cache)
- [How A Cycle Runs](#how-a-cycle-runs)
- [Troubleshooting](#troubleshooting)
- [Disclaimer](#disclaimer)

---

## What It Does

| Area | Behaviour |
| --- | --- |
| Auth | identity string plus an installation id, no email, no password, no code |
| Wire | every request sealed with RSA-OAEP-256 and A256GCM, reply opened with the one time key |
| Session | access token kept for the cycle, renewed by signing in again |
| Inviter | bound once per account, mining is refused while it is missing |
| Task board | every open task claimed in one pass, one log line each |
| Mining | a round is opened when none runs, the settled reward is reported each cycle |
| Multi account | one identity per line in `data.txt`, one proxy per line in `proxy.txt` |

The mining length, the settlement interval, the reward rate and the task list are read from the server, so a change on their side does not need a new build.

## Requirements

- Rust 1.75 or newer, https://rustup.rs
- Linux, Windows, or macOS
- A Sirius identity, copied from the app, or the human check solver for `enroll`
- Optional: a proxy that allows HTTPS CONNECT

## Install

```bash
git clone https://github.com/Yuurisan-N1/Sirius.git
cd Sirius
cargo build --release
```

The binary lands in `target/release/sirius-bot`. The wrapper `./run.sh` builds it for you when the source is newer.

## Identity File

`data.txt`, one identity per line:

```
0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
```

That is the whole account. Copy it from the app: **Profile → Identity Backup → copy**. Blank lines and lines starting with `#` are ignored. The file is gitignored and should be `chmod 600`.

An installation id may follow the identity after a pipe, but it is optional: the bot generates one per identity and remembers it in `state/`.

## First Run

```bash
# 1. put the inviter code in config.json, mining is refused without it
# 2. copy the identity from the app into data.txt
# 3. look at the server answer without claiming anything
./run.sh probe

# 4. start the loop
./run.sh run        # foreground
./run.sh bg         # background, log to sirius-bot.log
./run.sh screen     # detached screen session
./run.sh stop       # stop the background or screen run
```

No identity yet? The bot can register one:

```bash
./run.sh enroll 3
```

That asks the solver for a human check token, registers three identities, and appends them to `data.txt`.

## Modes

| Command | What it does |
| --- | --- |
| `./run.sh run` | full automation loop, one cycle per `sleep_seconds` |
| `./run.sh probe` | read only pass: registration config and region answer, no claim |
| `./run.sh selftest` | offline checks on the envelope, the identity reader and the money lines |
| `./run.sh enroll <count>` | register fresh identities and append them to `data.txt` |
| `./run.sh build` | release build only |

`probe` is the safe way to see what the server reports before letting the loop claim anything.

## Configuration

`config.json` — the canon block, nothing else:

```json
{
  "settings": {
    "sleep_seconds": 3600,
    "referral_code": "EghAsP"
  }
}
```

| Key | Meaning |
| --- | --- |
| `sleep_seconds` | pause between cycles, clamped to 60 at the low end |
| `referral_code` | your own user code, bound to every account before mining starts |

An empty or missing `referral_code` falls back to the built in default, so the bot still binds an inviter out of the box. Change the value in `config.json` when you want your own code bound instead.

## Proxy

`proxy.txt`, one proxy per line, same order as `data.txt`. Blank file means direct.

```
http://user:pass@host:port
socks5://user:pass@host:port
```

A proxy without a port falls back to 1080 for socks5 and 8080 for http. Special characters in the user or password have to be percent encoded.

## Installation Cache

Each identity gets `state/account-<prefix>.json`, mode `0600`, gitignored:

| Field | Meaning |
| --- | --- |
| `identity_id` | the account |
| `installation_id` | the installation the server saw on the first sign in |

The installation id is generated once and reused, so the account keeps the same device fingerprint. Tokens are never written to disk and never printed.

## How A Cycle Runs

1. Sign in with the identity and the cached installation id.
2. Read the profile, print the user code, the region, and the join date.
3. Bind the inviter when the account has none.
4. Read the task board, claim every open task, then read the summary.
5. Mining: report the running round, or open a new one when the last one ended.
6. Sleep `sleep_seconds` and repeat.

A task or a round that fails is logged and skipped, it never stops the cycle.

## Troubleshooting

| Line in the log | Meaning |
| --- | --- |
| `the identity is 32 characters instead of 64` | the line in `data.txt` is not an identity |
| `the server refused the identity` | the identity does not exist on that server |
| `the region is closed` | the address is blocked, run the account behind a proxy |
| `the inviter was skipped` | the account already carries an inviter |
| `the server asks every new identity for a human check` | `enroll` needs the solver up |
| `the solver could not be reached` | the solver is down |
| `the reply was not sealed` | the server answered outside the envelope, check the base url |

## Disclaimer

For personal use on your own account only. Automating a service can break its terms, and the account risk is yours. No warranty, no refund, use at your own pace.
