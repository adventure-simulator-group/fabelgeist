"""Fresh-process verification for the Cascadeur weapon migration."""

from pathlib import Path
import json
import os
import traceback

import csc
import pycsc


ROOT = Path(os.environ.get("FABELGEIST_ROOT", r"C:\Users\adler\projects\fabelgeist"))
SOURCE_ROOT = ROOT / "assets_src" / "biped"
WORK_ROOT = ROOT / "target" / "cascadeur_weapon_orientation_migration"
MANIFEST = WORK_ROOT / "expected.json"
REPORT = WORK_ROOT / "verification.txt"
EXPECTED_ROOT = WORK_ROOT / "expected"
STATUS_ROOT = WORK_ROOT / "verification_status"
TOLERANCE = 1e-5


def _write_report(lines):
    REPORT.write_text("\n".join(lines) + "\n", encoding="utf-8")


def _one_object(scene, name):
    matches = [
        object_id
        for object_id in scene.model_viewer().get_objects(name=name)
        if scene.model_viewer().get_object_name(object_id) == name
    ]
    if len(matches) != 1:
        raise RuntimeError(f"Expected one {name!r}, found {len(matches)}")
    return matches[0]


def _error(rotation, expected):
    quaternion = rotation.to_quaternion()
    actual = tuple(float(getattr(quaternion, name)()) for name in ("w", "x", "y", "z"))
    dot = abs(sum(x * y for x, y in zip(actual, expected)))
    return 1.0 - min(1.0, dot)


def _load(app, path):
    manager = app.get_scene_manager()
    application_scene = manager.create_application_scene()
    manager.set_current_scene(application_scene)
    if not app.get_data_source_manager().load_scene(str(path)):
        raise RuntimeError(f"Failed to load {path}")
    return application_scene


def run(_scene):
    lines = [f"manifest={MANIFEST}"]
    app = csc.app.get_application()
    data_sources = app.get_data_source_manager()
    opened = []
    try:
        requested = os.environ.get("CASCADEUR_MIGRATION_FILE")
        if requested:
            relative_path = Path(requested)
            expected_path = (EXPECTED_ROOT / relative_path).with_suffix(".json")
            status_path = (STATUS_ROOT / relative_path).with_suffix(".txt")
            status_path.parent.mkdir(parents=True, exist_ok=True)
            if not expected_path.exists():
                status_path.write_text("skipped-no-weapon-pair\n", encoding="utf-8")
                lines.append("status=success")
                _write_report(lines)
                return
            expected = {
                relative_path.as_posix(): json.loads(expected_path.read_text(encoding="utf-8"))
            }
        else:
            status_path = None
            expected = json.loads(MANIFEST.read_text(encoding="utf-8"))["files"]
        lines.append(f"files={len(expected)}")
        _write_report(lines)
        for index, (relative, record) in enumerate(sorted(expected.items()), 1):
            path = SOURCE_ROOT / Path(relative)
            application_scene = _load(app, path)
            opened.append(application_scene)
            scene = application_scene.domain_scene()
            wrapped = pycsc.wrap(scene)
            left = pycsc.wrap(_one_object(scene, "l_weapon"), wrapped)
            bind_error = _error(left.Node3d.model_local_rotation.get(), record["bind"])
            frame_count = scene.layers_viewer().frames_count()
            if frame_count != len(record["frames"]):
                raise RuntimeError(
                    f"{relative}: frame count changed from {len(record['frames'])} to {frame_count}"
                )
            frame_errors = [
                _error(left.Transform.local_rotation.get(frame), expected_rotation)
                for frame, expected_rotation in enumerate(record["frames"])
            ]
            max_frame_error = max(frame_errors, default=0.0)
            if bind_error > TOLERANCE or max_frame_error > TOLERANCE:
                raise RuntimeError(
                    f"{relative}: bind_error={bind_error}, max_frame_error={max_frame_error}"
                )
            data_sources.close_scene(application_scene)
            opened.remove(application_scene)
            lines.append(
                f"[{index}/{len(expected)}] verified {relative} "
                f"bind_error={bind_error} max_frame_error={max_frame_error}"
            )
            _write_report(lines)
        lines.append("status=success")
        _write_report(lines)
        if status_path is not None:
            status_path.write_text("verified\n", encoding="utf-8")
    except Exception:
        lines.extend(("status=failed", traceback.format_exc()))
        _write_report(lines)
        requested = os.environ.get("CASCADEUR_MIGRATION_FILE")
        if requested:
            failed_status = (STATUS_ROOT / Path(requested)).with_suffix(".txt")
            failed_status.parent.mkdir(parents=True, exist_ok=True)
            failed_status.write_text("failed\n", encoding="utf-8")
        raise
    finally:
        for application_scene in opened:
            data_sources.close_scene(application_scene)
