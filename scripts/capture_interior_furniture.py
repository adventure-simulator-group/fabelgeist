"""Capture the interior furniture catalog through production presentation."""
from capture_building_review import capture


if __name__ == "__main__":
    capture("interior-furniture-catalog", "Interior furniture: compact and broad variants",
            evidence_name="furniture-presentation.json", review_input="interior-catalog.json")
