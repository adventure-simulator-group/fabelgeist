#!/usr/bin/env python3
"""Exercise #719 against a fresh, owned local SpacetimeDB; never reset data."""
from __future__ import annotations

import argparse
import http.client
import json
import os
from pathlib import Path
import secrets
import socket
import subprocess
import tempfile
import time

from dev_stack import dev_bootstrap_token, spacetime_auth_token

ROOT = Path(__file__).resolve().parents[1]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--catalog", type=Path, required=True)
    parser.add_argument("--world", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    catalog = json.loads(args.catalog.read_text())
    world = json.loads(args.world.read_text())
    settlement = next(row for row in world["settlements"] if row["id"] == catalog["settlement_id"])
    args.output.mkdir(parents=True, exist_ok=True)
    run = Path(tempfile.mkdtemp(prefix="property-acceptance-", dir=args.output.resolve()))
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1", 0))
        port = reservation.getsockname()[1]
    server = f"http://127.0.0.1:{port}"
    database = f"property-acceptance-{secrets.token_hex(8)}"
    capability = dev_bootstrap_token(ROOT, args.output.resolve() / "capabilities")
    environment = dict(os.environ, ADVENTURESIM_DEV_BOOTSTRAP_TOKEN=capability, CARGO_NET_OFFLINE="true")
    log = (run / "build.log").open("w")
    subprocess.run(["cargo", "build", "--release", "--target", "wasm32-unknown-unknown",
                    "-p", "adventuresim-stdb-module", "--features", "authority-tests"],
                   cwd=ROOT, env=environment, stdout=log, stderr=subprocess.STDOUT, check=True)
    log.close()
    process_log = (run / "server.log").open("w")
    process = subprocess.Popen(["spacetime", "start", "--non-interactive", "--listen-addr",
                                f"127.0.0.1:{port}", "--data-dir", str(run / "data")],
                               cwd=ROOT, stdout=process_log, stderr=subprocess.STDOUT)
    timings: dict[str, float] = {}
    try:
        for _ in range(200):
            if process.poll() is not None:
                raise RuntimeError(f"Owned server exited; see {run / 'server.log'}")
            with socket.socket() as probe:
                if probe.connect_ex(("127.0.0.1", port)) == 0:
                    break
            time.sleep(0.1)
        else:
            raise RuntimeError("Owned server did not become ready")
        wasm = ROOT / "target/wasm32-unknown-unknown/release/adventuresim_stdb_module.wasm"
        with (run / "publish.log").open("w") as publish_log:
            subprocess.run(["spacetime", "publish", "--no-config", "--delete-data=never",
                            "--yes=remote,skip-login", "--server", server,
                            "--bin-path", str(wasm), database], cwd=ROOT,
                           stdout=publish_log, stderr=subprocess.STDOUT, check=True)
        token = spacetime_auth_token()
        headers = {"Authorization": f"Bearer {token}"}

        def request(path: str, body: str, *, authenticated: bool = True,
                    content_type: str = "application/json") -> tuple[int, str]:
            connection = http.client.HTTPConnection("127.0.0.1", port, timeout=180)
            try:
                connection.request("POST", f"/v1/database/{database}/{path}", body,
                                   {"Content-Type": content_type, **(headers if authenticated else {})})
                response = connection.getresponse()
                return response.status, response.read().decode()
            finally:
                connection.close()

        def call(name: str, arguments: list[object], *, succeeds: bool = True,
                 authenticated: bool = True) -> None:
            start = time.perf_counter()
            status, body = request(f"call/{name}", json.dumps(arguments), authenticated=authenticated)
            timings[name] = time.perf_counter() - start
            assert (status // 100 == 2) == succeeds, (name, status, body)

        def query(sql: str, *, authenticated: bool = True) -> list[list[object]]:
            status, body = request("sql", sql, authenticated=authenticated, content_type="text/plain")
            assert status // 100 == 2, (sql, status, body)
            return json.loads(body)[0]["rows"]

        call("register_strategic_gateway", [{"none": []}, 0])
        call("dev_bootstrap_base", [capability])
        call("authority_test_property_setup", [capability, json.dumps(catalog), json.dumps(settlement["economy"])])
        call("authority_test_materialize_property_household", [capability, catalog["settlement_id"]])
        count = query(f"SELECT COUNT(*) AS count FROM settlement_property WHERE settlement_id = '{catalog['settlement_id']}'")[0][0]
        assert count == len(catalog["homes"]), (count, len(catalog["homes"]))
        occupied = query("SELECT * FROM backend_household_property_occupancies")
        assert occupied and query("SELECT * FROM backend_household_property_occupancies", authenticated=False) == []
        assert sum(row[2] + row[3] for row in occupied) == catalog["population"]
        materialized = sum(row[2] for row in occupied)
        assert materialized > 0
        market = next(home for home in catalog["homes"] if home["market_reserve"] and home["tier"] == "Cheap")
        property_id = market["id"]
        call("register_settlement_properties", [json.dumps(catalog)])
        changed = json.loads(json.dumps(catalog))
        changed["homes"][0]["east_metres"] += 1
        call("register_settlement_properties", [json.dumps(changed)], succeeds=False)
        call("register_settlement_properties", [json.dumps(catalog)], succeeds=False, authenticated=False)
        call("rent_residence", [719001, property_id], authenticated=False, succeeds=False)
        call("rent_residence", [719001, property_id])
        before = query("SELECT * FROM inventory_item WHERE character_id = 719002")
        call("buy_residence", [719002, property_id], succeeds=False)
        assert query("SELECT * FROM inventory_item WHERE character_id = 719002") == before
        call("authority_test_property_contract", [capability])
        call("authority_test_property_capacity", [capability])
        # Registration remains a read-only idempotent operation after holdings exist.
        start = time.perf_counter()
        for _ in range(5):
            call("register_settlement_properties", [json.dumps(catalog)])
        timings["warm_registration_mean"] = (time.perf_counter() - start) / 5
        report = {"settlement": catalog["settlement_id"], "population": catalog["population"],
                  "generated_homes": count, "household_home_rows": len(occupied),
                  "materialized_residents": materialized,
                  "checks": ["immutable scoped identity", "bounded census", "exact rental and ownership",
                             "guest occupancy independent of title and membership", "atomic double-acquisition rejection",
                             "temporal capacity", "gateway privacy", "idempotence"],
                  "seconds": timings}
        (args.output / "acceptance.json").write_text(json.dumps(report, indent=2) + "\n")
        print(json.dumps(report, indent=2))
    finally:
        process.terminate()
        try:
            process.wait(timeout=15)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=15)
        process_log.close()


if __name__ == "__main__":
    main()
