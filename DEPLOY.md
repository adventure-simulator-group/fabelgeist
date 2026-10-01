# Deployment

One command, one VPS, two things it can put there:

```bash
just deploy example.com          # the showcase: art demo and Texture Studio
just deploy example.com game     # the playable game (strategic web, tactical servers, SpacetimeDB)
```

Both run from a checkout of this repository. The showcase is deployed manually.
The private test game can deploy automatically after the Rust quality workflow
passes on `main`; the server does not poll the repository.

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
HCLOUD_SERVER_TYPE=cx43        # game default: cx43 in fsn1; showcase: cpx11 in ash
HCLOUD_IMAGE=ubuntu-24.04
HCLOUD_SERVER_NAME=...         # reuse a server that exists under another name
HCLOUD_SSH_CIDR=203.0.113.10/32    # absent: your detected public IP, else anywhere
HCLOUD_SSH_KEY_PATH=~/.ssh/id_ed25519.pub
DEPLOY_HOST=203.0.113.20       # deploy to this box; skips Hetzner entirely
DEPLOY_USER=root
DEPLOY_ROOT=/opt/fabelgeist
SERVICE_USER=fabelgeist        # game only
DEPLOY_REPO=https://github.com/...   # game only: build a fork
TEST_USER=tester                   # game only: one private tester
TEST_PASSWORD_HASH=...             # required: caddy hash-password output
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

## Private test game

`test.fabelgeist.com` runs on a dedicated x86-64 Debian/Ubuntu server. The
recommended inexpensive starting point is a Hetzner CX43 in Falkenstein
(`HCLOUD_SERVER_TYPE=cx43`, `HCLOUD_LOCATION=fsn1`). Its shared CPUs and European
location suit functional testing; they are not a performance or latency promise.
The setup installs the pinned Rust and SpacetimeDB versions, the Wasm build tools,
and Caddy. The first build can take substantially longer than later builds.

The three services are `fabelgeist-stdb`, `fabelgeist-web`, and
`fabelgeist-dispatcher`. All backends listen on loopback. Caddy requires the
single tester's HTTP Basic credentials on every page, asset and mission socket.
It terminates HTTPS and forwards `/t6001` through `/t6999` to mission listeners.
Strategic-web translates private mission addresses into the configured secure
origin; the dispatcher stops allocating when this port range is exhausted.

Each deployment publishes and seeds a **new development database**. The old
world remains in SpacetimeDB and is never migrated, reset or deleted. Restarting
services keeps the currently configured world. Changing the deployed version
starts with a new development world; active tactical missions are disconnected
when their dispatcher is stopped. This is a private single-tester sandbox, not a
multi-user player-account system. Previous worlds occupy disk until an operator
explicitly approves their removal.

The build prepares native executables, Wasm, game assets, the schema, and the
verified pinned terrain/map bundle before stopping the live gateway. The new
database is seeded before the service swap. A failed build or failed seed leaves
the existing game services running. A failure during the swap requires operator
recovery; automatic rollback of binaries and service units is not implemented.

Create a tester password and generate its bcrypt hash with `caddy hash-password`
(the command prompts for the password). Put the hash in `TEST_PASSWORD_HASH`;
never commit the password, its hash, cloud tokens or SSH keys. The deployer mints
a local SpacetimeDB identity and session-signing secret and retains them in the
server's restricted environment file. No SpacetimeDB account is required.

```bash
python3 scripts/deploy.py test.fabelgeist.com --target game
# Default build revision is origin/main. A commit SHA selects an exact revision.
python3 scripts/deploy.py test.fabelgeist.com --target game --ref <commit-sha>
```

Add a Porkbun `A` record with host `test` and value equal to the server's public
IPv4. Leave the existing root and wiki records intact. Only add an `AAAA` record
if IPv6 is configured and reachable. Caddy obtains the domain certificate once
DNS points to the server and ports 80 and 443 are reachable.

### Automatic deployment

The `Deploy test game` workflow runs after `Rust quality` passes for a commit on
`main`. It deploys that exact commit, verifies HTTPS and compares the served
commit stamp. It also supports a manual run against `main`. Deploy jobs are
serialized, and pull-request or fork workflow runs cannot invoke deployment.

Create the GitHub environment **test-game** with these values:

| Kind | Name | Value |
| --- | --- | --- |
| Variable | `TEST_DEPLOY_HOST` | Dedicated server IPv4 or SSH hostname |
| Variable | `TEST_DEPLOY_USER` | SSH operator, defaults to `root` |
| Secret | `TEST_DEPLOY_SSH_KEY` | Dedicated deployment key whose public key is authorized on the server |
| Secret | `TEST_DEPLOY_KNOWN_HOSTS` | Server SSH host-key record verified during initial setup |
| Secret | `TEST_PASSWORD_HASH` | Tester's bcrypt password hash |
| Secret | `TEST_PASSWORD` | Same tester password, for the HTTPS verification request |

The workflow uses `/opt/fabelgeist-test` and service user `fabelgeist-test`.
Use those values on the first manual deployment too. To let hosted GitHub runners
reach SSH, set `HCLOUD_SSH_CIDR=0.0.0.0/0` during provisioning or configure an
appropriate firewall rule. Setup disables SSH password authentication. Actions
requires a verified known-hosts entry and refuses unknown SSH host keys.
Cloud provisioning credentials stay on the operator's machine; automated deploys
connect to the existing server and do not need a Hetzner API token.

Merge the deployment changes before enabling this workflow. Add the environment
credentials and DNS record, run the first deployment, then test login, renderer
startup, mission entry, movement/combat, and returning to the strategic view in a
WebGPU-capable browser. The automated HTTPS check does not establish WebGPU
rendering or successful combat.

The deployed revision is available at `/static/deploy.json`. A separate
`/static/browser-check.html` page reports the secure context, Wasm availability,
and whether WebGPU can supply a GPU adapter.

## Verification

```bash
curl -I https://example.com/                     # HTTP/2 200
curl -sI https://example.com/tactical/wasm/art-demo_bg.wasm | grep -i content-type   # application/wasm
curl -s https://example.com/deploy.json          # showcase commit; game: /static/deploy.json
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
