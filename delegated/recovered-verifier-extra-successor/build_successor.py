#!/usr/bin/env python3
"""Derive the two-record successor deterministically; no overwrite or product execution."""
import argparse,hashlib,pathlib
HERE=pathlib.Path(__file__).resolve().parent
SOURCE_SHA='2b0be3743a7703f9ce11dfb905a079b89074a3ef3ce8031467951d110f1d5172'
HELPER='''\n# BEGIN ADDITIONAL QUALIFIED PUBLIC REDACTIONS (two historical records only)
# The writer output is a reconstructed historical transcript; the portability
# input was already a redacted record, not the unredacted candidate it cites.
# Public hashes identify reconstructed bytes, not the withheld original bytes.
_extra_crosswalk_path = "recovered/privacy-redaction-evidence/ADDITIONAL-CROSSWALK.json"
_extra_crosswalk_sha = "617070c5ebd6d57ea79cdf4a69f1adc9cbece96feb144dccba99f40e234e2dc1"
_extra_objects = {
    "recovered/final-writer-static-trace/OUTPUT-419499.txt":
        ("57acc4af7fb1ed29ab96640a6f584028be9898dc9ff790add5ae5f2cca6af0ad", 590, "8564fa7a5b482ca1b774590be271608f572e8020abc7fa9d3d609821b690e192", 458, 3),
    "recovered/transcript-other-candidates/portability-review.redacted.json":
        ("21c676a13fb8e24af1bbbcdf4f19ffd911fc33e19d65b4f939b6c5da6bcf3e84", 8746, "37cd6bdad0a3c4f5779f5161a7292bc88f7cc701678ee279f98112c11766acf5", 8723, 2),
}
_extra_manifests = {
    "recovered/final-writer-static-trace/MANIFEST.json": "7d3b0330a9bede96b03a3f237148ff72b078724b51bb3347b54e368345ffda2a",
    "recovered/late-current-cell-count-evidence/MANIFEST.json": "727392524ed24cfb9a1f43b4851b7fdf7ed7f9c91952bcb1c8c2791c63659786",
    "recovered/transcript-other-candidates/portability-review.manifest.json": "5350e430c3926481aa099e9a17d8e977006e990aa355627462fb6ba60882c559",
}
_extra_crosswalk_bytes = (root / _extra_crosswalk_path).read_bytes()
if hashlib.sha256(_extra_crosswalk_bytes).hexdigest() != _extra_crosswalk_sha:
    raise ValueError("Additional qualified public redaction crosswalk hash mismatch")
_extra_crosswalk = json.loads(_extra_crosswalk_bytes)
if _extra_crosswalk.get("schema") != "726-extra-qualified-redaction-crosswalk-v1" or len(_extra_crosswalk.get("records", [])) != 2:
    raise ValueError("Additional qualified redaction scope mismatch")
_extra_rows = {}
for _extra_row in _extra_crosswalk["records"]:
    _extra_relative = _extra_row["publication_path"]
    if _extra_relative in _extra_rows or _extra_relative not in _extra_objects:
        raise ValueError("Unexpected additional redaction identity")
    _orig_sha, _orig_bytes, _public_sha, _public_bytes, _spans = _extra_objects[_extra_relative]
    if ((_extra_row["original_sha256"], _extra_row["original_bytes"], _extra_row["public_sha256"], _extra_row["public_bytes"], len(_extra_row["redactions"]))
            != (_orig_sha, _orig_bytes, _public_sha, _public_bytes, _spans)):
        raise ValueError("Additional qualified public redaction identity mismatch")
    _extra_rows[_extra_relative] = _extra_row
if set(_extra_rows) != set(_extra_objects):
    raise ValueError("Incomplete additional qualified redaction crosswalk")
_extra_seen = set()

def _extra_historical_manifest(relative):
    if relative not in _extra_manifests:
        raise ValueError("Unexpected additional historical manifest")
    data = (root / relative).read_bytes()
    if hashlib.sha256(data).hexdigest() != _extra_manifests[relative]:
        raise ValueError("Additional historical manifest hash mismatch")
    return data

def _extra_public_object(relative, historical_sha, data):
    if relative not in _extra_objects:
        raise ValueError("Unexpected additional qualified public object path")
    original_sha, _, public_sha, public_bytes, _ = _extra_objects[relative]
    if historical_sha != original_sha:
        raise ValueError("Additional historical manifest identity mismatch")
    if len(data) != public_bytes or hashlib.sha256(data).hexdigest() != public_sha:
        raise ValueError("Additional reconstructed public bytes mismatch")
    row = _extra_rows[relative]
    for span in row["redactions"]:
        start, size, marker = span["public_byte_offset"], span["public_token_bytes"], span["replacement"]
        if marker != "<redacted-local-path>" or data[start:start + size] != marker.encode("utf-8"):
            raise ValueError("Additional reconstructed public redaction span mismatch")
    if row["kind"] == "historical-temp-locators":
        doc = json.loads(data)
        locator = doc["external_locator_constraints"][4]["locator"]
        if locator.count("<redacted-local-path>") != 2:
            raise ValueError("Reconstructed portability locator mismatch")
    _extra_seen.add(relative)
    return data
# END ADDITIONAL QUALIFIED PUBLIC REDACTIONS
'''
REPLACEMENTS=[
('# END QUALIFIED PUBLIC REDACTIONS\n','# END QUALIFIED PUBLIC REDACTIONS\n'+HELPER),
('item = json.loads((directory / "portability-review.manifest.json").read_text(encoding="utf-8"))','item = json.loads(_extra_historical_manifest("recovered/transcript-other-candidates/portability-review.manifest.json"))'),
('if hashlib.sha256((directory / item["file"]).read_bytes()).hexdigest() != item["redacted_sha256"]:\n    raise ValueError("Redacted portability review hash mismatch")','if item["file"] != "portability-review.redacted.json":\n    raise ValueError("Historical portability review identity mismatch")\n_extra_public_object("recovered/transcript-other-candidates/" + item["file"], item["redacted_sha256"], (directory / item["file"]).read_bytes())'),
('writer_trace = json.loads((writer_dir / "MANIFEST.json").read_bytes())','writer_trace = json.loads(_extra_historical_manifest("recovered/final-writer-static-trace/MANIFEST.json"))'),
('for file, key in (("OUTPUT-419499.txt", "apply_output_sha256"), ("FINAL-BUNDLE-LINE.txt", "bundle_line_sha256")):\n    if hashlib.sha256((writer_dir / file).read_bytes()).hexdigest() != writer_trace[key]:\n        raise ValueError("Historical writer evidence hash mismatch")','for file, key in (("OUTPUT-419499.txt", "apply_output_sha256"), ("FINAL-BUNDLE-LINE.txt", "bundle_line_sha256")):\n    if file == "OUTPUT-419499.txt":\n        _extra_public_object("recovered/final-writer-static-trace/" + file, writer_trace[key], (writer_dir / file).read_bytes())\n    elif hashlib.sha256((writer_dir / file).read_bytes()).hexdigest() != writer_trace[key]:\n        raise ValueError("Historical writer evidence hash mismatch")'),
('late_cell = json.loads((root / "recovered/late-current-cell-count-evidence/MANIFEST.json").read_text(encoding="utf-8"))','late_cell = json.loads(_extra_historical_manifest("recovered/late-current-cell-count-evidence/MANIFEST.json"))'),
('if hashlib.sha256(writer_bytes).hexdigest() != late_cell["writer_sha256"] or hashlib.sha256(output_bytes).hexdigest() != late_cell["output_sha256"]:\n    raise ValueError("Late cell count evidence hash mismatch")','if hashlib.sha256(writer_bytes).hexdigest() != late_cell["writer_sha256"]:\n    raise ValueError("Late cell count evidence hash mismatch")\n_extra_public_object(late_cell["output"], late_cell["output_sha256"], output_bytes)'),
('print("Verified", count, "evidence files,", dependency_count,','if _extra_seen != set(_extra_objects):\n    raise ValueError("Not all additional qualified redactions were checked")\nprint("Qualified redaction: reconstructed historical writer output and previously redacted portability record checked; their earlier bytes were not verified.")\n\nprint("Verified", count, "evidence files,", dependency_count,'),
]
def transform(raw):
 if hashlib.sha256(raw).hexdigest()!=SOURCE_SHA:raise ValueError('unexpected previous successor identity')
 s=raw.decode('utf-8')
 for old,new in REPLACEMENTS:
  if s.count(old)!=1:raise ValueError('transformation anchor missing or non-unique')
  s=s.replace(old,new,1)
 return s.encode('utf-8')
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--source',type=pathlib.Path,default=HERE/'ORIGINAL-verify_recovered_public_redacted.py');g=ap.add_mutually_exclusive_group();g.add_argument('--write',type=pathlib.Path);g.add_argument('--check',type=pathlib.Path);a=ap.parse_args();out=transform(a.source.read_bytes())
 if a.write:
  with a.write.open('xb') as f:f.write(out)
 if a.check and a.check.read_bytes()!=out:raise ValueError('successor differs from deterministic transformation')
 print(hashlib.sha256(out).hexdigest())
if __name__=='__main__':main()
