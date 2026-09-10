"""Capture furnished settlement rooms and their authoritative access proofs."""
from capture_building_review import capture


if __name__ == "__main__":
    capture("furnished-room-review", "Furnished settlement interiors",
            evidence_name="furniture-presentation.json", review_input="interior-access-proofs.json",
            additional_evidence_names=("building-presentation.json",))
