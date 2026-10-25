import json, pathlib, datetime, zipfile
R = pathlib.Path(__file__).resolve().parents[2]
O = pathlib.Path(__file__).resolve().parent

def save(number, indices, facts, narratives=(), ranges=None, disposition="accounted exact current07 source authority roles and explicit limits"):
    cat=json.load(open(R / "recovered-sources/catalog.json"))["rows"]
    at=datetime.datetime.now(datetime.timezone.utc).isoformat()
    src=[dict(scope_index=i,source_id=cat[i]["source_id"],sha256=cat[i]["expected_sha256"],origin=cat[i]["origin"],actual_EOF=cat[i]["line_count"],ranges=(ranges or {}).get(i,[[1,cat[i]["line_count"]]]),read_kind="fresh exact scoped source in bounded untruncated reads") for i in indices]
    for x in src:
        c = cat[x["scope_index"]]
        if c.get("source_state") == "deleted":
            x.update(read_kind="deleted role: exact recovered base personally compared to retained relocation; no current body", current_body_coverage=False, source_state="deleted", recovered_base_sha256=c["verified_object_sha256"])
    a=dict(id=f"CONT07-CONTRACT-{number}",at=at,lens="audit-redundant-authority",sources=src,supplier_narratives_personally_consumed=list(narratives),facts=list(facts),new_local_findings=[],repair_status="frozen",validation_status="not_run",no_other_lens_credit=True)
    with (O / "coverage.jsonl").open("a") as f:
        for x in src:
            f.write(json.dumps(dict(x,contract=a["id"],disposition=disposition,discovery_status=("deleted role personally inventoried from exact recovered base and absence evidence; no current-body credit" if x.get("source_state")=="deleted" else "exact current07 scoped source personally inventoried"),repair_status="frozen",validation_status="not_run",other_lens_credit=False))+"\n")
    z=zipfile.ZipFile("/home/brunosegovia/Downloads/774-audit-for-segovb-20261023.zip")
    bn=next(n for n in z.namelist() if n.endswith("camera/initial07-redundant-authority/coverage.jsonl"))
    base={json.loads(l)["source_id"] for l in z.read(bn).splitlines()}
    rows=[json.loads(l) for l in (O / "coverage.jsonl").read_text().splitlines()]
    seen={x["source_id"] for x in rows}
    n=next((c for c in cat if c["source_id"] not in base|seen),None)
    a["next_cell"]={"scope_index":n["scope_index"],"start_line":1} if n else None
    (O / f"contract-{number}.json").write_text(json.dumps(a,indent=2)+"\n")
    p=json.load(open(O / "progress.json"))
    p.update(at=at,latest_contract=a["id"],new_unique_sources=len(seen-base),current_unique_sources=len(base|seen),remaining_sources=2853-len(base|seen),fresh_read_rows=len(rows),first_unfinished_scope072_index=n["scope_index"] if n else None,first_unfinished_source=n["source_id"] if n else None,baseline_source_rereads=[{"scope_index":c["scope_index"],"source_id":c["source_id"]} for c in cat if c["source_id"] in seen&base])
    p.pop("first_unfinished_line",None)
    (O / "progress.json").write_text(json.dumps(p,indent=2)+"\n")
    print({k:p[k] for k in ["current_unique_sources","remaining_sources","first_unfinished_scope072_index"]})
