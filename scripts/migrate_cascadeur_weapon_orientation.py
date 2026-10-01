"""Batch-migrate legacy Cascadeur scenes to the corrected left weapon bind.

Run through Cascadeur's CLI. Each original is backed up, edited through the
Cascadeur API in a temporary sibling file, validated in memory, and atomically
replaced only after save succeeds.
"""

from pathlib import Path
import json
import os
import shutil
import traceback

import csc
import pycsc
ROOT = Path(os.environ.get("FABELGEIST_ROOT", r"C:\Users\adler\projects\fabelgeist"))
SOURCE_ROOT = ROOT / "assets_src" / "biped"
WORK_ROOT = ROOT / "target" / "cascadeur_weapon_orientation_migration"
BACKUP_ROOT = WORK_ROOT / "backups"
REPORT = WORK_ROOT / "migration.txt"
MANIFEST = WORK_ROOT / "expected.json"
EXPECTED_ROOT = WORK_ROOT / "expected"
STATUS_ROOT = WORK_ROOT / "migration_status"
TOLERANCE = 1e-5
POSE_TOLERANCE = 5e-3


def _write_report(lines):
    REPORT.parent.mkdir(parents=True, exist_ok=True)
    REPORT.write_text("\n".join(lines) + "\n", encoding="utf-8")


def _objects(scene, name):
    return [
        object_id
        for object_id in scene.model_viewer().get_objects(name=name)
        if scene.model_viewer().get_object_name(object_id) == name
    ]


def _one_object(scene, name):
    matches = _objects(scene, name)
    if len(matches) != 1:
        raise RuntimeError(f"Expected one {name!r}, found {len(matches)}")
    return matches[0]


def _quaternion_tuple(rotation):
    quaternion = rotation.to_quaternion()
    return tuple(float(getattr(quaternion, name)()) for name in ("w", "x", "y", "z"))


def _rotation_error(a, b):
    qa = _quaternion_tuple(a)
    qb = _quaternion_tuple(b)
    dot = abs(sum(x * y for x, y in zip(qa, qb)))
    return 1.0 - min(1.0, dot)


def _quaternion_error(a, b):
    qa = tuple(float(getattr(a, name)()) for name in ("w", "x", "y", "z"))
    qb = tuple(float(getattr(b, name)()) for name in ("w", "x", "y", "z"))
    dot = abs(sum(x * y for x, y in zip(qa, qb)))
    return 1.0 - min(1.0, dot)


def _triangle_rotation(main, direction, additional):
    return csc.math.basic_transform_from_triangle(
        csc.math.Triangle(main, direction, additional)
    ).rotation


def _rotate_triangle(main, direction, additional, correction, target_rotation):
    """Rotate a point triangle using Cascadeur's actual vector convention."""
    candidates = []
    for quaternion in (correction.to_quaternion(), correction.inverse().to_quaternion()):
        new_direction = main + quaternion * (direction - main)
        new_additional = main + quaternion * (additional - main)
        result = _triangle_rotation(main, new_direction, new_additional)
        candidates.append(
            (_quaternion_error(result, target_rotation), new_direction, new_additional)
        )
    error, new_direction, new_additional = min(candidates, key=lambda item: item[0])
    if error > TOLERANCE:
        raise RuntimeError(f"Could not realize target point-triangle rotation: error={error}")
    return new_direction, new_additional


def _load(app, path):
    scene_manager = app.get_scene_manager()
    application_scene = scene_manager.create_application_scene()
    scene_manager.set_current_scene(application_scene)
    if not app.get_data_source_manager().load_scene(str(path)):
        raise RuntimeError(f"Failed to load {path}")
    return application_scene


def _migrate_scene(scene):
    outer = pycsc.wrap(scene)
    left = pycsc.wrap(_one_object(scene, "l_weapon"), outer)
    right = pycsc.wrap(_one_object(scene, "r_weapon"), outer)
    point_ids = [
        _one_object(scene, "weapon_MainPoint_l"),
        _one_object(scene, "weapon_DirectionPoint_l"),
        _one_object(scene, "weapon_AdditionalPoint_l"),
    ]
    frame_count = scene.layers_viewer().frames_count()
    old_bind = left.Node3d.model_local_rotation.get()
    new_bind = right.Node3d.model_local_rotation.get()
    if _rotation_error(old_bind, new_bind) <= TOLERANCE:
        return "already-current", [_quaternion_tuple(left.Transform.local_rotation.get(frame)) for frame in range(frame_count)]

    old_frames = [left.Transform.local_rotation.get(frame) for frame in range(frame_count)]
    # Change the joint's local basis while retaining each animation pose.
    local_basis_correction = old_bind.inverse() * new_bind
    desired_frames = [value * local_basis_correction for value in old_frames]
    callback_errors = []

    def apply_animation(model, update, scene_updater):
        try:
            data_editor = model.data_editor()
            position_nodes = [
                update.get_object_by_id(object_id).root_group().node_deep("Position")
                for object_id in point_ids
            ]
            position_ids = [node.data_id() for node in position_nodes]
            actuals = set(position_ids)
            for frame in range(frame_count):
                desired_local = desired_frames[frame]
                actual_local = left.Transform.local_rotation.get(frame)
                old_global = left.Transform.global_rotation.get(frame)
                effective_parent = old_global * actual_local.inverse()
                desired_global = effective_parent * desired_local
                correction = desired_global * old_global.inverse()
                positions = [node.value(frame) for node in position_nodes]
                old_triangle_rotation = _triangle_rotation(*positions)
                target_triangle_rotation = correction.to_quaternion() * old_triangle_rotation
                direction, additional = _rotate_triangle(
                    positions[0],
                    positions[1],
                    positions[2],
                    correction,
                    target_triangle_rotation,
                )
                data_editor.set_data_value(position_ids[0], frame, positions[0])
                data_editor.set_data_value(position_ids[1], frame, direction)
                data_editor.set_data_value(position_ids[2], frame, additional)
                scene_updater.run_update(actuals, frame)
        except Exception:
            callback_errors.append(traceback.format_exc())
            raise

    if not scene.modify_update("Correct left weapon animation orientation", apply_animation):
        detail = callback_errors[0] if callback_errors else "no Python exception was exposed"
        raise RuntimeError("Cascadeur rejected left weapon animation migration:\n" + detail)

    def apply_bind(py_scene):
        py_left = pycsc.wrap(_one_object(scene, "l_weapon"), py_scene)
        py_left.Node3d.model_local_rotation.set(new_bind)

    outer.edit("Correct left weapon bind orientation", apply_bind)

    actual_bind = left.Node3d.model_local_rotation.get()
    bind_error = _rotation_error(actual_bind, new_bind)
    frame_errors = [
        _rotation_error(left.Transform.local_rotation.get(frame), desired)
        for frame, desired in enumerate(desired_frames)
    ]
    if bind_error > TOLERANCE or any(error > POSE_TOLERANCE for error in frame_errors):
        worst_frame = max(range(len(frame_errors)), key=frame_errors.__getitem__, default=None)
        actual_worst = None if worst_frame is None else _quaternion_tuple(left.Transform.local_rotation.get(worst_frame))
        desired_worst = None if worst_frame is None else _quaternion_tuple(desired_frames[worst_frame])
        raise RuntimeError(
            f"Live validation failed: bind_error={bind_error}, "
            f"actual_bind={_quaternion_tuple(actual_bind)}, "
            f"desired_bind={_quaternion_tuple(new_bind)}, "
            f"max_frame_error={max(frame_errors, default=0.0)}, "
            f"worst_frame={worst_frame}, actual={actual_worst}, desired={desired_worst}"
        )
    actual_frames = [
        _quaternion_tuple(left.Transform.local_rotation.get(frame))
        for frame in range(frame_count)
    ]
    return "migrated", actual_frames


def run(_scene):
    WORK_ROOT.mkdir(parents=True, exist_ok=True)
    BACKUP_ROOT.mkdir(parents=True, exist_ok=True)
    lines = [f"source_root={SOURCE_ROOT}", f"backup_root={BACKUP_ROOT}"]
    expected = {"files": {}}
    app = csc.app.get_application()
    data_sources = app.get_data_source_manager()
    opened = []
    try:
        requested = os.environ.get("CASCADEUR_MIGRATION_FILE")
        paths = [SOURCE_ROOT / Path(requested)] if requested else sorted(SOURCE_ROOT.rglob("*.casc"))
        lines.append(f"discovered={len(paths)}")
        _write_report(lines)
        for index, source in enumerate(paths, 1):
            relative = source.relative_to(SOURCE_ROOT)
            status_path = (STATUS_ROOT / relative).with_suffix(".txt")
            status_path.parent.mkdir(parents=True, exist_ok=True)
            temporary = source.with_name(f".{source.name}.orientation-migration.tmp.casc")
            shutil.copy2(source, temporary)
            application_scene = _load(app, temporary)
            opened.append(application_scene)
            scene = application_scene.domain_scene()
            if len(_objects(scene, "l_weapon")) != 1 or len(_objects(scene, "r_weapon")) != 1:
                status = "skipped-no-weapon-pair"
                data_sources.close_scene(application_scene)
                opened.remove(application_scene)
                temporary.unlink()
                lines.append(f"[{index}/{len(paths)}] {status} {relative}")
                _write_report(lines)
                status_path.write_text(f"{status}\n", encoding="utf-8")
                continue

            status, actual_frames = _migrate_scene(scene)
            left = pycsc.wrap(_one_object(scene, "l_weapon"), pycsc.wrap(scene))
            right = pycsc.wrap(_one_object(scene, "r_weapon"), pycsc.wrap(scene))
            expected["files"][relative.as_posix()] = {
                "bind": _quaternion_tuple(right.Node3d.model_local_rotation.get()),
                "frames": actual_frames,
                "status": status,
            }
            application_scene.save(str(temporary))
            data_sources.close_scene(application_scene)
            opened.remove(application_scene)

            backup = BACKUP_ROOT / relative
            backup.parent.mkdir(parents=True, exist_ok=True)
            if not backup.exists():
                shutil.copy2(source, backup)
            os.replace(temporary, source)
            expected_path = (EXPECTED_ROOT / relative).with_suffix(".json")
            expected_path.parent.mkdir(parents=True, exist_ok=True)
            expected_path.write_text(
                json.dumps(expected["files"][relative.as_posix()], indent=2) + "\n",
                encoding="utf-8",
            )
            lines.append(f"[{index}/{len(paths)}] {status} {relative}")
            _write_report(lines)
            status_path.write_text(f"{status}\n", encoding="utf-8")

        MANIFEST.write_text(json.dumps(expected, indent=2) + "\n", encoding="utf-8")
        lines.extend((f"matched={len(expected['files'])}", "status=success"))
        _write_report(lines)
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
