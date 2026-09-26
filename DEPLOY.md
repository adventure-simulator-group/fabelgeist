# Deployment

One command, one VPS, two things it can put there:

```bash
just deploy example.com          # the showcase: art demo and Texture Studio
just deploy example.com game     # the playable game (strategic web, tactical servers, SpacetimeDB)
```

Both run from a checkout of this repository on your machine. Deploys are
manual and immediate: nothing on the box polls or auto-updates.

## What the operator needs

Python 3.11 or newer, `ssh`, and `scp`. OpenSSH ships with Windows 10+,
macOS, and Linux.

The showcase is built locally before upload, so it also needs the tools
`just showcase` uses: cargo (the pinned nightly installs itself from
`rust-toolchain.toml` on first use), the `wasm32-unknown-unknown` target, and
the `wasm-bindgen` CLI at the exact version in `Cargo.lock`. The game target
needs nothing else locally; every compiler runs on the VPS.

`just` is optional. Every recipe is one line that calls a script under
`scripts/`, and the commands below show both forms.

## Settings

Copy `.env.example` to `.env` (gitignored) and fill in what you use. Real
environment variables take precedence. Everything is optional:

```bash
HCLOUD_TOKEN=...               # absent: deploy to an existing box instead
HCLOUD_LOCATION=ash
HCLOUD_SERVER_TYPE=cpx11       # default per target: cpx11 showcase, cpx31 game
HCLOUD_IMAGE=ubuntu-24.04
HCLOUD_SERVER_NAME=...         # reuse a server that exists under another name
HCLOUD_SSH_CIDR=203.0.113.10/32    # absent: your detected public IP, else anywhere
HCLOUD_SSH_KEY_PATH=~/.ssh/id_ed25519.pub
DEPLOY_HOST=203.0.113.20       # deploy to this box; skips Hetzner entirely
DEPLOY_USER=root
DEPLOY_ROOT=/opt/fabelgeist
SERVICE_USER=fabelgeist        # game only
DEPLOY_REPO=https://github.com/...   # game only: build a fork
```

With `HCLOUD_TOKEN` set, `just deploy` creates or reuses a Hetzner server
named `fabelgeist-<domain>`, registers `~/.ssh/id_ed25519.pub` (generated if
missing), and applies a firewall that allows SSH from your IP plus public 80
and 443. Point the domain's A record at the IP it prints; Caddy issues the
certificate itself. `www` can be a CNAME to the root.

Without the token, the script connects to `DEPLOY_HOST` (or the domain) as
`DEPLOY_USER` with whatever key ssh would use for that host.

## Showcase target

This is the default, and what goes public first. It is plain files:

- `/` a landing page linking the two
- `/art-demo` the procedural art demo
- `/texture-studio/` the material editor
- `/tactical/assets/...` the asset tree the art demo reads

The deploy runs `scripts/showcase.py --build-only` locally, hashes the site
tree plus the asset roots, asks the box for its manifest of what it already
has, and sends only the difference as one tar stream over ssh. The box applies
the delta to a hard-linked copy of the live tree and renames it into place, so
the site is never half-updated. The previous tree is kept for rollback.

```bash
just deploy example.com
python scripts/deploy.py example.com               # same thing without just
python scripts/deploy.py example.com --skip-build  # upload the existing build
python scripts/deploy.py example.com --plan        # build and hash only, no network
just rollback example.com                          # back to the previous upload
```

The first upload is large: about 950 files and 1.85 GB, almost all of it the
asset tree. Later deploys send only what changed, typically the rebuilt
bundles. `--plan` prints the exact numbers before anything leaves your machine.

Under `/opt/fabelgeist` on the box: `site/` (served by Caddy), `site.prev/`,
`manifest.json`, `manifest.prev.json`. `site/deploy.json` records the commit
and time of the current deploy. The box runs Caddy and nothing else, so the
smallest Hetzner size is plenty.

### Build cost

The showcase build compiles three Bevy applications to wasm in release. What
that costs on a machine that has only done debug builds so far:

| | Size |
| --- | --- |
| Pinned nightly toolchain with the wasm target | about 1.5 GB |
| Release wasm build directory for all three bundles | about 0.8 GB |
| Output site (`target/showcase/site`) | about 0.35 GB |

The build directory stays this small because `.cargo/config.toml` already
turns off incremental artifacts and debug info for every profile. A cold
build takes on the order of half an hour; rebuilding after a change to one
bundle takes a few minutes. Pass `--dev` to `scripts/showcase.py` for a debug
build while iterating locally, but deploy release.

## Building and deploying from Windows

The script and its build steps are plain Python and work under PowerShell.
What to install, once:

1. **Rust** via `rustup` from rustup.rs. Do not pick a toolchain; the
   repository's `rust-toolchain.toml` installs the pinned nightly and the
   wasm target the first time cargo runs inside the checkout.
2. **wasm-bindgen CLI** at the version in `Cargo.lock`:
   `cargo install wasm-bindgen-cli --version 0.2.108 --locked`. The build
   scripts refuse to run with any other version and print the one they want.
3. **Python 3.11 or newer.** The Microsoft Store build is fine. It installs the
   `py` launcher; if `python` is not on your PATH, use `py` in the commands
   below and set `PYTHON_BIN=py` for `just`.
4. **just**, optional: `cargo install just` or `winget install Casey.Just`.
5. **OpenSSH** is already part of Windows 10 and later. Create a key if you
   have none: `ssh-keygen -t ed25519`. It lands in `$HOME\.ssh\id_ed25519`,
   which is what both Hetzner and ssh will use.

Then, in PowerShell from the repository root on a local drive (not a network
share; cargo is very slow over one):

```powershell
Copy-Item .env.example .env        # then edit it
py scripts\showcase.py             # build and open the showcase on localhost
py scripts\deploy.py example.com   # build, upload, done
```

Things that differ from Linux and are already handled: there is no `rsync`
on Windows, which is why the upload is a manifest-driven tar stream over ssh;
the Store Python is invoked as `py`; and ssh host keys are accepted on first
contact so a fresh box does not stop the script with a prompt.

## Game target

Restored from the `easy-deploy` branch (commit `d6456c6e`) and not yet
exercised on top of current `main`. The box clones the public repository,
builds `strategic-web`, the tactical server and the dispatcher in release,
publishes the SpacetimeDB module, and runs three systemd units:
`fabelgeist-stdb`, `fabelgeist-web`, `fabelgeist-dispatcher`. Caddy
proxies everything to strategic-web and `/t6000`–`/t6999` to the tactical
servers the dispatcher spawns on loopback.

Before this target can work on `main`, two changes from that branch need to
land: the dispatcher's `--public-url-prefix` flag and the tactical server's
`--public-addr` flag, so mission servers advertise `wss://domain/t<port>`
instead of their bind address. The 8 GB swapfile the setup creates is for
building Bevy in release next to the running services; `cpx31` is the floor.

```bash
just deploy example.com game                    # build and deploy origin/main
python scripts/deploy.py example.com --target game --ref my-branch
python scripts/deploy.py example.com --target game --load-world   # fresh database
python scripts/deploy.py example.com --target game --break-clients
```

`--yes=delete-data` is never passed. Destroying the world stays a manual act.

## Verification

```bash
curl -I https://example.com/                     # HTTP/2 200
curl -sI https://example.com/tactical/wasm/art-demo_bg.wasm | grep -i content-type   # application/wasm
curl -s https://example.com/deploy.json          # which commit is live
ssh root@example.com 'journalctl -u caddy -n 50 --no-pager'
```

On Windows use `curl.exe`, since `curl` alone is a PowerShell alias for a
different command.

## Common failures

**`unsupported location for server type`.** Hetzner does not offer every size
in every location. Set `HCLOUD_LOCATION` and `HCLOUD_SERVER_TYPE` to a valid
pair.

**`wasm-bindgen CLI x does not match Rust y`.** Install the version it names
with `cargo install wasm-bindgen-cli --version y --locked`.

**`Permission denied (publickey)` on a reused box.** The box only knows the
key that created it. Add your `~/.ssh/id_ed25519.pub` to its
`/root/.ssh/authorized_keys` from a machine that can log in, or create a new
box.

**Domain does not load.** `dig +short example.com A` (or `nslookup` on
Windows) should return the VPS IPv4. Until DNS propagates, test the origin
directly: `curl -I --resolve example.com:443:<ip> https://example.com/`.

**HTTPS works but the demo says it needs WebGPU.** That is the browser, not
the deploy: open it in a current Chrome or Edge.
