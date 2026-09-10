"""Capture deterministic outdoor furniture and ground through the production renderer."""
from capture_building_review import capture


if __name__ == "__main__":
    capture("furniture-review", "City furniture and street surfaces",
            evidence_name="furniture-presentation.json", review_input="furniture-layout.json")
