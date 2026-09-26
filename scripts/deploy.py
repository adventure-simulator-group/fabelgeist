#!/usr/bin/env python3
"""Deploy Fabelgeist to one VPS.

    just deploy example.com           # the showcase (default target)
    just deploy example.com game      # the playable game

Targets:

  showcase  The art demo and Texture Studio as plain files.
            Built locally by scripts/showcase.py, uploaded with the asset tree
            the art demo reads, served by Caddy. Nothing else runs on the box.
            Works from Windows: only Python, ssh and scp are needed locally.

  game      strategic-web, the tactical dispatcher and SpacetimeDB, cloned and
            built on the box from the public repository. Restored from the
            easy-deploy branch; see DEPLOY.md for what it still needs.

With HCLOUD_TOKEN set (in .env or the environment) a Hetzner server named
after the domain is created or reused, with your SSH key and a firewall.
Without it the deploy targets DEPLOY_HOST, or the domain itself, over ssh.
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import io
import ipaddress
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import socket
import subprocess
import sys
import tarfile
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
import showcase  # noqa: E402  (shares the site directory and asset overlay order)

DEPLOY_DIR = ROOT / "deploy"
TARGETS = ("showcase", "game")

# Hetzner sizes per target. The showcase is static files; the game compiles
# Bevy in release on the box and needs the memory for it.
SERVER_TYPES = {"showcase": "cpx11", "game": "cpx31"}

GAME_UNITS = ("fabelgeist-stdb", "fabelgeist-web", "fabelgeist-dispatcher")
GAME_DATABASE = "adventuresim-stdb-module"
HCLOUD_API = "https://api.hetzner.cloud/v1"


def load_env_file() -> None:
    """Read deploy settings from .env; real environment variables win.

    `just` here is pinned to `.env.tactical` for isolated dev stacks, so it
    never loads `.env` for us.
    """
    path = ROOT / ".env"
    if not path.is_file():
        return
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, _, value = line.partition("=")
        value = re.split(r"\s+#", value, maxsplit=1)[0]  # trailing comment
        os.environ.setdefault(key.strip(), value.strip().strip("'\""))


def log(message: str) -> None:
    print(message, file=sys.stderr, flush=True)


def executable(name: str) -> str:
    resolved = shutil.which(name)
    if resolved is None:
        raise SystemExit(f"Missing required executable: {name}")
    return resolved


def run(command: list[str], **kwargs) -> None:
    result = subprocess.run(command, cwd=ROOT, **kwargs)
    if result.returncode:
        raise SystemExit(f"command failed ({result.returncode}): {' '.join(command)}")


# --------------------------------------------------------------------------
# Hetzner. Skipped entirely unless HCLOUD_TOKEN is set.
# --------------------------------------------------------------------------


def hcloud(method: str, path: str, data: object = None, query: dict | None = None) -> dict:
    url = f"{HCLOUD_API}{path}"
    if query:
        url += "?" + urllib.parse.urlencode(query)
    body = None if data is None else json.dumps(data).encode()
    request = urllib.request.Request(url, data=body, method=method)
    request.add_header("Authorization", f"Bearer {os.environ['HCLOUD_TOKEN']}")
    request.add_header("Content-Type", "application/json")
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            text = response.read().decode()
            return json.loads(text) if text else {}
    except urllib.error.HTTPError as error:
        detail = error.read().decode(errors="replace")
        raise SystemExit(f"Hetzner API {method} {path} failed: {error.code} {detail}") from error


def hcloud_list(path: str, key: str, query: dict | None = None) -> list[dict]:
    items: list[dict] = []
    page = 1
    while True:
        data = hcloud("GET", path, query={**(query or {}), "page": page})
        items.extend(data.get(key, []))
        page = data.get("meta", {}).get("pagination", {}).get("next_page")
        if not page:
            return items


def wait_action(action: dict) -> None:
    action_id = action.get("id")
    if not action_id:
        return
    for _ in range(120):
        status = hcloud("GET", f"/actions/{action_id}")["action"]["status"]
        if status == "success":
            return
        if status == "error":
            raise SystemExit(f"Hetzner action {action_id} failed")
        time.sleep(2)
    raise SystemExit(f"timed out waiting for Hetzner action {action_id}")


def server_name(domain: str) -> str:
    configured = os.environ.get("HCLOUD_SERVER_NAME")
    if configured:
        return configured
    safe = re.sub(r"[^a-z0-9-]+", "-", domain.lower()).strip("-")
    return f"fabelgeist-{safe}"[:63].strip("-")


def public_key_path() -> Path:
    configured = os.environ.get("HCLOUD_SSH_KEY_PATH")
    if configured:
        return Path(configured).expanduser()
    public = Path("~/.ssh/id_ed25519.pub").expanduser()
    if public.exists():
        return public
    private = public.with_suffix("")
    private.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    run([executable("ssh-keygen"), "-t", "ed25519", "-f", str(private), "-N", "", "-C", "fabelgeist-deploy"])
    return public


def ensure_ssh_key(name: str) -> dict:
    path = public_key_path()
    public_key = path.read_text(encoding="utf-8").strip()
    for key in hcloud_list("/ssh_keys", "ssh_keys"):
        if key.get("public_key") == public_key:
            log(f"Hetzner SSH key: {key['name']}")
            return key
    log(f"Creating Hetzner SSH key from {path}")
    return hcloud("POST", "/ssh_keys", {"name": f"{name}-deploy", "public_key": public_key})["ssh_key"]


def ssh_sources() -> list[str]:
    configured = os.environ.get("HCLOUD_SSH_CIDR")
    if configured:
        return [configured]
    try:
        with urllib.request.urlopen("https://api.ipify.org", timeout=5) as response:
            address = ipaddress.ip_address(response.read().decode().strip())
        return [f"{address}/32" if address.version == 4 else f"{address}/128"]
    except Exception as error:  # noqa: BLE001 - any failure means "could not detect"
        log(f"Could not detect the public IP for the SSH firewall rule: {error}")
        log("Allowing SSH from anywhere. Set HCLOUD_SSH_CIDR to lock this down.")
        return ["0.0.0.0/0", "::/0"]


def firewall_rules() -> list[dict]:
    world = ["0.0.0.0/0", "::/0"]
    return [
        {"direction": "in", "protocol": "tcp", "port": "22", "source_ips": ssh_sources(), "description": "SSH deploy"},
        {"direction": "in", "protocol": "tcp", "port": "80", "source_ips": world, "description": "HTTP and ACME"},
        {"direction": "in", "protocol": "tcp", "port": "443", "source_ips": world, "description": "HTTPS"},
        {"direction": "in", "protocol": "icmp", "source_ips": world, "description": "ping"},
    ]


def ensure_firewall(name: str) -> dict:
    firewall_name = os.environ.get("HCLOUD_FIREWALL_NAME", f"{name}-fw")
    existing = hcloud_list("/firewalls", "firewalls", {"name": firewall_name})
    if existing:
        firewall = existing[0]
        log(f"Hetzner firewall: {firewall_name}")
        action = hcloud("POST", f"/firewalls/{firewall['id']}/actions/set_rules", {"rules": firewall_rules()})
        wait_action(action.get("action") or {})
        return hcloud("GET", f"/firewalls/{firewall['id']}")["firewall"]
    log(f"Creating Hetzner firewall: {firewall_name}")
    data = hcloud("POST", "/firewalls", {"name": firewall_name, "rules": firewall_rules()})
    for action in data.get("actions", []):
        wait_action(action)
    return data["firewall"]


def create_server(name: str, ssh_key: dict, firewall: dict, server_type: str) -> dict:
    location = os.environ.get("HCLOUD_LOCATION", "ash")
    image = os.environ.get("HCLOUD_IMAGE", "ubuntu-24.04")
    log(f"Creating Hetzner server {name}: {server_type}, {image}, {location}")
    data = hcloud(
        "POST",
        "/servers",
        {
            "name": name,
            "server_type": server_type,
            "image": image,
            "location": location,
            "ssh_keys": [ssh_key["id"]],
            "firewalls": [{"firewall": firewall["id"]}],
            "labels": {"app": "fabelgeist"},
            "public_net": {"enable_ipv4": True, "enable_ipv6": True},
        },
    )
    wait_action(data.get("action", {}))
    for action in data.get("next_actions", []):
        wait_action(action)
    return hcloud("GET", f"/servers/{data['server']['id']}")["server"]


def apply_firewall(firewall: dict, server: dict) -> None:
    applied = firewall.get("applied_to") or []
    if any(item.get("type") == "server" and item.get("server", {}).get("id") == server["id"] for item in applied):
        return
    log(f"Applying firewall to {server['name']}")
    data = hcloud(
        "POST",
        f"/firewalls/{firewall['id']}/actions/apply_to_resources",
        {"apply_to": [{"type": "server", "server": {"id": server["id"]}}]},
    )
    for action in data.get("actions", []):
        wait_action(action)


def warn_dns(domain: str, ip: str) -> None:
    try:
        resolved = {info[4][0] for info in socket.getaddrinfo(domain, None, socket.AF_INET)}
    except socket.gaierror:
        resolved = set()
    if ip not in resolved:
        log(f"DNS note: point the {domain} A record at {ip} before expecting HTTPS to work.")


def provision(domain: str, server_type: str) -> str:
    """Create or reuse the Hetzner server and return its public IPv4."""
    name = server_name(domain)
    ssh_key = ensure_ssh_key(name)
    firewall = ensure_firewall(name)
    existing = hcloud_list("/servers", "servers", {"name": name})
    if existing:
        log(f"Reusing Hetzner server {name}")
        server = existing[0]
        apply_firewall(firewall, server)
    else:
        server = create_server(name, ssh_key, firewall, server_type)
    ip = server.get("public_net", {}).get("ipv4", {}).get("ip")
    if not ip:
        raise SystemExit(f"server {name} has no public IPv4")
    warn_dns(domain, ip)
    return ip


# --------------------------------------------------------------------------
# ssh
# --------------------------------------------------------------------------

SSH_OPTIONS = ["-o", "BatchMode=yes", "-o", "StrictHostKeyChecking=accept-new", "-o", "ConnectTimeout=10"]


def ssh_command(target: str, remote: str) -> list[str]:
    return [executable("ssh"), *SSH_OPTIONS, target, remote]


def ssh_script(target: str, script: str, args: list[str] | None = None) -> None:
    """Run a bash script on the VPS, fed over stdin so nothing is quoted twice."""
    quoted = " ".join(shlex.quote(argument) for argument in args or [])
    # Bytes, not text: on Windows, text mode would rewrite every newline as
    # CRLF and the remote bash would choke on the carriage returns.
    result = subprocess.run(ssh_command(target, f"bash -s -- {quoted}"), input=script.encode("utf-8"))
    if result.returncode:
        raise SystemExit(f"remote step failed ({result.returncode})")


def ssh_output(target: str, remote: str) -> str:
    return subprocess.run(ssh_command(target, remote), capture_output=True, text=True, check=True).stdout


def ssh_input(target: str, remote: str, data: bytes) -> None:
    result = subprocess.run(ssh_command(target, remote), input=data)
    if result.returncode:
        raise SystemExit(f"remote step failed ({result.returncode}): {remote}")


def scp(local: Path, target: str, remote_path: str) -> None:
    run([executable("scp"), *SSH_OPTIONS, str(local), f"{target}:{remote_path}"])


def wait_for_ssh(target: str) -> None:
    log(f"Waiting for SSH on {target}")
    for _ in range(60):
        probe = subprocess.run(ssh_command(target, "true"), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        if probe.returncode == 0:
            return
        time.sleep(5)
    raise SystemExit(f"timed out waiting for SSH on {target}")


def render(template: Path, values: dict[str, str]) -> str:
    text = template.read_text(encoding="utf-8")
    for key, value in values.items():
        text = text.replace("{{" + key + "}}", value)
    return text


# --------------------------------------------------------------------------
# Remote steps shared by both targets.
# --------------------------------------------------------------------------

SUDO = 'if command -v sudo >/dev/null 2>&1 && [ "$(id -u)" -ne 0 ]; then sudo_cmd="sudo"; else sudo_cmd=""; fi'

BASE_SETUP_SCRIPT = r"""
set -euo pipefail
remote_root="$1"
%(sudo)s

command -v apt-get >/dev/null 2>&1 || { echo "This deploy expects a Debian/Ubuntu VPS." >&2; exit 1; }
export DEBIAN_FRONTEND=noninteractive
if ! command -v caddy >/dev/null 2>&1 || ! command -v ufw >/dev/null 2>&1; then
    ${sudo_cmd} apt-get update
    ${sudo_cmd} apt-get install -y caddy ufw
fi

${sudo_cmd} mkdir -p "${remote_root}"
${sudo_cmd} chown "$(id -u):$(id -g)" "${remote_root}"

ssh_port="${SSH_CONNECTION:-}"; ssh_port="${ssh_port##* }"
printf '%%s\n' "${ssh_port}" | grep -Eq '^[0-9]+$' || ssh_port="22"
${sudo_cmd} ufw allow "${ssh_port}/tcp"
${sudo_cmd} ufw allow 80/tcp
${sudo_cmd} ufw allow 443/tcp
${sudo_cmd} ufw default deny incoming
${sudo_cmd} ufw default allow outgoing
${sudo_cmd} ufw --force enable
""" % {"sudo": SUDO}

INSTALL_CADDY_SCRIPT = r"""
set -euo pipefail
%(sudo)s
[ -f /etc/caddy/Caddyfile ] \
    && ${sudo_cmd} cp /etc/caddy/Caddyfile "/etc/caddy/Caddyfile.backup.$(date +%%Y%%m%%d%%H%%M%%S)" || true
${sudo_cmd} install -m 0644 /tmp/fabelgeist.Caddyfile /etc/caddy/Caddyfile
${sudo_cmd} caddy validate --config /etc/caddy/Caddyfile
${sudo_cmd} systemctl enable caddy
${sudo_cmd} systemctl reload caddy || ${sudo_cmd} systemctl restart caddy
""" % {"sudo": SUDO}


def install_caddyfile(target: str, template: Path, values: dict[str, str]) -> None:
    with tempfile.TemporaryDirectory() as work:
        staged = Path(work) / "fabelgeist.Caddyfile"
        staged.write_text(render(template, values), encoding="utf-8", newline="\n")
        scp(staged, target, "/tmp/fabelgeist.Caddyfile")
    ssh_script(target, INSTALL_CADDY_SCRIPT)


# --------------------------------------------------------------------------
# Showcase target: static files, synced by content hash.
#
# rsync does not ship with Windows, so the delta is computed here: the box
# keeps a manifest of what it has, only changed files travel (as one tar
# stream over ssh) and the new tree replaces the old one atomically.
# --------------------------------------------------------------------------

SYNC_BEGIN_SCRIPT = r"""
set -euo pipefail
remote_root="$1"
cd "${remote_root}"
rm -rf site.new
# Hard links: a free copy of the live tree to apply the delta onto. tar
# unlinks before writing, so the live files are never touched in place.
if [ -d site ]; then cp -al site site.new; else mkdir site.new; fi
"""

SYNC_FINISH_SCRIPT = r"""
set -euo pipefail
remote_root="$1"
cd "${remote_root}"
find site.new -type d -empty -delete
mkdir -p site.new
chmod -R a+rX site.new
rm -rf site.prev
[ -d site ] && mv site site.prev
mv site.new site
[ -f manifest.json ] && mv manifest.json manifest.prev.json
mv manifest.json.new manifest.json
"""

ROLLBACK_SCRIPT = r"""
set -euo pipefail
remote_root="$1"
cd "${remote_root}"
[ -d site.prev ] || { echo "nothing to roll back to: no site.prev" >&2; exit 1; }
mv site site.rolled && mv site.prev site && mv site.rolled site.prev
if [ -f manifest.prev.json ]; then
    mv manifest.json manifest.rolled.json && mv manifest.prev.json manifest.json && mv manifest.rolled.json manifest.prev.json
else
    rm -f manifest.json
fi
"""


def collect_showcase_files() -> dict[str, Path]:
    """Remote path -> local file for the whole showcase tree.

    The site directory as built, plus the asset roots at /tactical/assets in
    scripts/build_wasm.py's overlay order: crate asset dirs win over assets/.
    """
    files: dict[str, Path] = {}
    for path in showcase.SITE.rglob("*"):
        if path.is_file():
            files[path.relative_to(showcase.SITE).as_posix()] = path
    for root in showcase.ASSET_ROOTS:
        for path in root.rglob("*"):
            if path.is_file():
                files.setdefault("tactical/assets/" + path.relative_to(root).as_posix(), path)
    return files


def digest(path: Path) -> str:
    sha = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            sha.update(chunk)
    return sha.hexdigest()


def plan_sync(local: dict[str, str], remote: dict[str, str]) -> tuple[list[str], list[str]]:
    """Files to upload (new or changed) and files to delete (gone locally)."""
    upload = sorted(rel for rel, sha in local.items() if remote.get(rel) != sha)
    delete = sorted(rel for rel in remote if rel not in local)
    return upload, delete


def deploy_stamp(target: str) -> bytes:
    # The commit is informational. git may be absent, or refuse a checkout on
    # a network share it considers foreign-owned; neither should stop or
    # alarm a deploy, so its stderr stays quiet and we note the gap instead.
    try:
        commit = subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True, stderr=subprocess.DEVNULL
        ).strip()
    except (OSError, subprocess.CalledProcessError):
        commit = None
        log("Note: could not read the git commit here; deploy.json will record it as unknown.")
    stamp = {"target": target, "commit": commit, "deployed_at": dt.datetime.now(dt.timezone.utc).isoformat()}
    return json.dumps(stamp, indent=2).encode()


def write_tar(stream, files: dict[str, Path], members: list[str], extra: dict[str, bytes]) -> None:
    """Stream a gzip tar of the chosen members, plus generated files, to `stream`."""
    with tarfile.open(fileobj=stream, mode="w|gz") as tar:
        for rel in members:
            tar.add(str(files[rel]), arcname=rel, recursive=False)
        for rel, data in extra.items():
            info = tarfile.TarInfo(rel)
            info.size = len(data)
            info.mtime = int(time.time())
            info.mode = 0o644
            tar.addfile(info, io.BytesIO(data))


def deploy_showcase(target: str, remote_root: str, skip_build: bool) -> None:
    if not skip_build:
        run([sys.executable, str(ROOT / "scripts" / "showcase.py"), "--build-only", "--no-open"])
    if not (showcase.SITE / "index.html").is_file():
        raise SystemExit(f"no showcase build at {showcase.SITE}; run without --skip-build")

    log("Hashing the showcase tree...")
    files = collect_showcase_files()
    local = {rel: digest(path) for rel, path in files.items()}
    try:
        remote = json.loads(ssh_output(target, f"cat {shlex.quote(remote_root)}/manifest.json 2>/dev/null || echo {{}}"))
    except subprocess.CalledProcessError:
        remote = {}
    upload, delete = plan_sync(local, remote)
    size = sum(files[rel].stat().st_size for rel in upload)
    log(f"{len(files)} files total; uploading {len(upload)} ({size / 1e6:.1f} MB), deleting {len(delete)}")

    ssh_script(target, SYNC_BEGIN_SCRIPT, [remote_root])
    extract = ssh_command(target, f"tar xzf - --unlink-first -C {shlex.quote(remote_root)}/site.new")
    with subprocess.Popen(extract, stdin=subprocess.PIPE) as proc:
        write_tar(proc.stdin, files, upload, {"deploy.json": deploy_stamp("showcase")})
        proc.stdin.close()
        if proc.wait():
            raise SystemExit(f"upload failed ({proc.returncode})")
    if delete:
        ssh_input(target, f"cd {shlex.quote(remote_root)}/site.new && xargs -0 -r rm -f", "\0".join(delete).encode())
    ssh_input(target, f"cat > {shlex.quote(remote_root)}/manifest.json.new", json.dumps(local, indent=0).encode())
    ssh_script(target, SYNC_FINISH_SCRIPT, [remote_root])


# --------------------------------------------------------------------------
# Game target. Restored from the easy-deploy branch (commit d6456c6e): the
# box clones the public repository, builds strategic-web, the tactical server
# and dispatcher, publishes the SpacetimeDB module and runs three systemd
# units. Nothing is built or uploaded from the operator's machine.
# --------------------------------------------------------------------------

GAME_REPO_URL = os.environ.get("DEPLOY_REPO", "https://github.com/adventure-simulator-group/fabelgeist.git")


def spacetime_tarball() -> str:
    from just_tasks import SPACETIME_VERSION  # the one pinned version, shared with the dev recipes

    return (
        "https://github.com/clockworklabs/SpacetimeDB/releases/download/"
        f"v{SPACETIME_VERSION}/spacetime-x86_64-unknown-linux-gnu.tar.gz"
    )


GAME_SETUP_SCRIPT = r"""
set -euo pipefail
remote_root="$1"
service_user="$2"
spacetime_tarball="$3"
%(sudo)s
export DEBIAN_FRONTEND=noninteractive

# Bevy in release is the memory peak on this box, not the game. Swap covers it
# so a build cannot OOM-kill the running services next to it.
if [ ! -f /swapfile ]; then
    ${sudo_cmd} fallocate -l 8G /swapfile
    ${sudo_cmd} chmod 600 /swapfile
    ${sudo_cmd} mkswap /swapfile >/dev/null
    ${sudo_cmd} swapon /swapfile
    grep -q '^/swapfile' /etc/fstab || echo '/swapfile none swap sw 0 0' | ${sudo_cmd} tee -a /etc/fstab >/dev/null
fi

# clang and mold are not optional: .cargo/config.toml in the repo hardcodes
# them as the linker for x86_64-unknown-linux-gnu.
${sudo_cmd} apt-get install -y git build-essential pkg-config libssl-dev clang mold curl unzip

if [ ! -x "${HOME}/.cargo/bin/cargo" ]; then
    echo "Installing rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --no-modify-path --default-toolchain none
fi

if [ ! -x /usr/local/bin/spacetimedb-standalone ]; then
    echo "Installing SpacetimeDB..."
    tmp="$(mktemp -d)"
    curl -sSL -o "${tmp}/spacetime.tar.gz" "${spacetime_tarball}"
    tar xzf "${tmp}/spacetime.tar.gz" -C "${tmp}"
    ${sudo_cmd} install -m 0755 "${tmp}/spacetimedb-standalone" /usr/local/bin/spacetimedb-standalone
    ${sudo_cmd} install -m 0755 "${tmp}/spacetimedb-cli" /usr/local/bin/spacetimedb-cli
    rm -rf "${tmp}"
fi

id "${service_user}" >/dev/null 2>&1 \
    || ${sudo_cmd} useradd --system --home "${remote_root}" --shell /usr/sbin/nologin "${service_user}"

${sudo_cmd} mkdir -p "${remote_root}/bin" "${remote_root}/stdb" "${remote_root}/src"
${sudo_cmd} chown "$(id -u):$(id -g)" "${remote_root}/bin" "${remote_root}/src"
${sudo_cmd} chown "${service_user}" "${remote_root}/stdb"
""" % {"sudo": SUDO}

GAME_INSTALL_UNITS_SCRIPT = r"""
set -euo pipefail
%(sudo)s
for unit in fabelgeist-stdb fabelgeist-web fabelgeist-dispatcher; do
    ${sudo_cmd} install -m 0644 "/tmp/${unit}.service" "/etc/systemd/system/${unit}.service"
done
${sudo_cmd} systemctl daemon-reload
${sudo_cmd} systemctl enable fabelgeist-stdb fabelgeist-web fabelgeist-dispatcher
# `start`, not `restart`: a redeploy must not drop the database out from under
# the version that is still serving traffic. The swap window restarts it.
${sudo_cmd} systemctl start fabelgeist-stdb
for _ in $(seq 1 60); do
    curl -sf -o /dev/null "http://127.0.0.1:3000/v1/ping" && break
    sleep 1
done
""" % {"sudo": SUDO}

# The standalone server signs its own identities, so the whole deploy needs no
# spacetimedb.com account. Minted once and reused forever: the module's
# register_strategic_gateway claims its singleton for the first authenticated
# caller, and rotating this would lock us out of our own gateway.
GAME_IDENTITY_SCRIPT = r"""
set -euo pipefail
remote_root="$1"
service_user="$2"
%(sudo)s

if [ -s "${remote_root}/env" ] && grep -q '^SPACETIMEDB_TOKEN=' "${remote_root}/env"; then
    echo "Reusing the existing strategic gateway identity."
else
    echo "Minting the strategic gateway identity..."
    token="$(curl -sSf -X POST http://127.0.0.1:3000/v1/identity \
        | python3 -c 'import json,sys; print(json.load(sys.stdin)["token"])')"
    [ -n "${token}" ] || { echo "failed to mint a SpacetimeDB identity" >&2; exit 1; }
    printf 'SPACETIMEDB_TOKEN=%%s\n' "${token}" \
        | ${sudo_cmd} install -m 640 -o root -g "${service_user}" /dev/stdin "${remote_root}/env"
fi

token="$(sed -n 's/^SPACETIMEDB_TOKEN=//p' "${remote_root}/env" 2>/dev/null \
    || ${sudo_cmd} sed -n 's/^SPACETIMEDB_TOKEN=//p' "${remote_root}/env")"
/usr/local/bin/spacetimedb-cli login --token "${token}" >/dev/null
""" % {"sudo": SUDO}

# Build with the OLD version still serving. Nothing is stopped until the swap.
GAME_BUILD_SCRIPT = r"""
set -euo pipefail
remote_root="$1"
repo_url="$2"
ref="$3"
clean="$4"
src="${remote_root}/src"
export PATH="${HOME}/.cargo/bin:${PATH}"
export CARGO_TERM_COLOR=never

if [ ! -d "${src}/.git" ]; then
    echo "Cloning ${repo_url}..."
    rm -rf "${src}"
    git clone "${repo_url}" "${src}"
fi

cd "${src}"
git remote set-url origin "${repo_url}"
git fetch --prune origin
git checkout -q --detach "origin/${ref}" 2>/dev/null || git checkout -q --detach "${ref}"
# -fd, never -fdx: target/ and the generated wasm are gitignored, and keeping
# them is the difference between a 2-minute deploy and a 30-minute one.
git clean -fd
echo "Building $(git rev-parse --short HEAD) ($(git log -1 --format=%s))"

if [ "${clean}" = "1" ]; then
    echo "Clean build requested; discarding target/"
    git clean -fdx
fi

cargo build --release \
    -p strategic-web \
    -p adventuresim-tactical-server \
    -p adventuresim-tactical-server-dispatcher

required="$(awk '/name = "wasm-bindgen"/{p=1} p && /version = /{gsub(/"/,"",$3); print $3; exit}' Cargo.lock)"
if ! command -v wasm-bindgen >/dev/null 2>&1 \
    || [ "$(wasm-bindgen --version | awk '{print $2}')" != "${required}" ]; then
    echo "Installing wasm-bindgen-cli ${required}..."
    cargo install wasm-bindgen-cli --version "${required}" --locked --force
fi

python3 scripts/build_wasm.py
"""

# Stop, publish, swap, start. This is the only window with downtime.
GAME_SWAP_SCRIPT = r"""
set -euo pipefail
remote_root="$1"
service_user="$2"
database="$3"
load_world="$4"
src="${remote_root}/src"
export PATH="${HOME}/.cargo/bin:${PATH}"
%(sudo)s

# Stopping the dispatcher reaps every tactical server it spawned: they are
# children in its cgroup, and systemd's default KillMode=control-group takes
# the whole group down.
${sudo_cmd} systemctl stop fabelgeist-web fabelgeist-dispatcher

${sudo_cmd} systemctl restart fabelgeist-stdb
for _ in $(seq 1 60); do
    curl -sf -o /dev/null "http://127.0.0.1:3000/v1/ping" && break
    sleep 1
done

cd "${src}"
echo "Publishing the SpacetimeDB module..."
/usr/local/bin/spacetimedb-cli publish \
    --server http://127.0.0.1:3000 \
    --module-path crates/adventuresim-stdb-module \
    --yes=remote,migrate,skip-login%(break_clients)s \
    "${database}"

if [ "${load_world}" = "1" ]; then
    echo "Loading the compiled 1544 world..."
    python3 scripts/init_world_runtime.py --repository .
    cargo run --release --package adventuresim-world-import \
        --bin adventuresim-world-import -- \
        --input target/world-1544.json --load \
        --server http://127.0.0.1:3000 --database "${database}"
fi

install -m 0755 target/release/strategic-web "${remote_root}/bin/strategic-web"
install -m 0755 target/release/adventuresim-tactical-server "${remote_root}/bin/"
install -m 0755 target/release/adventuresim-tactical-server-dispatcher "${remote_root}/bin/"

# Copy rather than symlink into the source tree: the next build's `git clean`
# would otherwise pull the served files out from under Caddy.
swap_tree() {
    [ -d "$1" ] || { echo "missing build output: $1" >&2; return 1; }
    rm -rf "$2.new" && cp -a "$1" "$2.new" && rm -rf "$2" && mv "$2.new" "$2"
}
swap_tree crates/strategic-web/static "${remote_root}/public"
swap_tree crates/adventuresim-stdb-module/static "${remote_root}/tactical"
[ -d target/strategic-map ] && swap_tree target/strategic-map "${remote_root}/strategic-map" || true

chmod -R a+rX "${remote_root}/bin" "${remote_root}/public" "${remote_root}/tactical"
[ -d "${remote_root}/strategic-map" ] && chmod -R a+rX "${remote_root}/strategic-map" || true

${sudo_cmd} systemctl start fabelgeist-web fabelgeist-dispatcher
sleep 3
${sudo_cmd} systemctl is-active --quiet fabelgeist-web \
    || { ${sudo_cmd} journalctl -u fabelgeist-web -n 40 --no-pager; exit 1; }
${sudo_cmd} systemctl is-active --quiet fabelgeist-dispatcher \
    || { ${sudo_cmd} journalctl -u fabelgeist-dispatcher -n 40 --no-pager; exit 1; }
"""


def install_game_units(target: str, values: dict[str, str]) -> None:
    with tempfile.TemporaryDirectory() as work:
        for unit in GAME_UNITS:
            path = Path(work) / f"{unit}.service"
            path.write_text(render(DEPLOY_DIR / f"{unit}.service", values), encoding="utf-8", newline="\n")
            scp(path, target, f"/tmp/{unit}.service")
    ssh_script(target, GAME_INSTALL_UNITS_SCRIPT)


def deploy_game(target: str, remote_root: str, values: dict[str, str], args: argparse.Namespace) -> None:
    service_user = values["SERVICE_USER"]
    ssh_script(target, GAME_SETUP_SCRIPT, [remote_root, service_user, spacetime_tarball()])
    install_game_units(target, values)
    ssh_script(target, GAME_IDENTITY_SCRIPT, [remote_root, service_user])
    if args.setup_only:
        return
    log(f"Building {args.ref} on the VPS (the first build takes a while)...")
    ssh_script(target, GAME_BUILD_SCRIPT, [remote_root, GAME_REPO_URL, args.ref, "1" if args.clean else "0"])
    # Deliberately never --yes=delete-data: destroying the live world stays a
    # decision someone makes by hand.
    swap = GAME_SWAP_SCRIPT % {"sudo": SUDO, "break_clients": ",break-clients" if args.break_clients else ""}
    ssh_script(target, swap, [remote_root, service_user, args.database, "1" if args.load_world else "0"])


# --------------------------------------------------------------------------


def parse_args(argv: list[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("domain", help="public domain, e.g. example.com")
    parser.add_argument("--target", choices=TARGETS, default="showcase")
    parser.add_argument("--remote-root", default=os.environ.get("DEPLOY_ROOT", "/opt/fabelgeist"))
    parser.add_argument("--setup-only", action="store_true", help="provision and configure, then stop")
    parser.add_argument("--plan", action="store_true", help="showcase: build and hash locally, print what a first upload would send, no network")
    parser.add_argument("--skip-build", action="store_true", help="showcase: upload the existing target/showcase/site")
    parser.add_argument("--rollback", action="store_true", help="showcase: swap the live tree back to the previous upload")
    game = parser.add_argument_group("game target")
    game.add_argument("--ref", default="main", help="branch, tag or commit the box builds (default: main)")
    game.add_argument("--clean", action="store_true", help="discard the box's target/ and rebuild from scratch")
    game.add_argument("--load-world", action="store_true", help="also load the compiled 1544 world into the database")
    game.add_argument("--break-clients", action="store_true", help="allow a schema change that invalidates connected clients")
    game.add_argument("--database", default=os.environ.get("SPACETIMEDB_DATABASE", GAME_DATABASE))
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    load_env_file()
    args = parse_args(argv)

    if args.plan:
        if not args.skip_build:
            run([sys.executable, str(ROOT / "scripts" / "showcase.py"), "--build-only", "--no-open"])
        files = collect_showcase_files()
        local = {rel: digest(path) for rel, path in files.items()}
        upload, _ = plan_sync(local, {})
        size = sum(files[rel].stat().st_size for rel in upload)
        log(f"first upload to {args.domain}: {len(upload)} files, {size / 1e6:.1f} MB; later deploys send only changes")
        return 0

    deploy_user = os.environ.get("DEPLOY_USER", "root")
    host = os.environ.get("DEPLOY_HOST")
    if os.environ.get("HCLOUD_TOKEN"):
        host = provision(args.domain, os.environ.get("HCLOUD_SERVER_TYPE", SERVER_TYPES[args.target]))
    target = f"{deploy_user}@{host or args.domain}"
    wait_for_ssh(target)

    if args.rollback:
        ssh_script(target, ROLLBACK_SCRIPT, [args.remote_root])
        log(f"rolled back https://{args.domain} to the previous upload")
        return 0

    values = {
        "DOMAIN": args.domain,
        "DEPLOY_ROOT": args.remote_root,
        "SERVICE_USER": os.environ.get("SERVICE_USER", "fabelgeist"),
        "DATABASE": args.database,
    }
    log(f"Configuring {target} for the {args.target}...")
    ssh_script(target, BASE_SETUP_SCRIPT, [args.remote_root])
    caddyfile = DEPLOY_DIR / ("Caddyfile" if args.target == "showcase" else "Caddyfile.game")
    install_caddyfile(target, caddyfile, values)

    if args.target == "showcase":
        if not args.setup_only:
            deploy_showcase(target, args.remote_root, args.skip_build)
    else:
        deploy_game(target, args.remote_root, values, args)

    if args.setup_only:
        log(f"VPS configured for {args.domain}")
    else:
        log(f"deployed https://{args.domain}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
