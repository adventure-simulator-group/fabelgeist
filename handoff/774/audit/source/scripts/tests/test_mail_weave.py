"""Physical ring spacing must produce four linked neighbors without fused wires."""
import sys
from pathlib import Path
import unittest

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from bake_mail_weave import RING_RADIUS_M, WIRE_RADIUS_M, PITCH_X_M, PITCH_Y_M, INCLINATION_RADIANS


def ring(row, column):
    angle = INCLINATION_RADIANS * (-1 if row % 2 else 1)
    t = np.linspace(0, 2*np.pi, 512, endpoint=False)
    centre = np.array([(column + (row % 2)*.5)*PITCH_X_M, row*PITCH_Y_M, 0])
    points = np.column_stack((RING_RADIUS_M*np.cos(t),
        RING_RADIUS_M*np.sin(t)*np.cos(angle), RING_RADIUS_M*np.sin(t)*np.sin(angle)))
    return centre + points, centre, np.array([0, -np.sin(angle), np.cos(angle)])


class MailWeaveTests(unittest.TestCase):
    def test_each_ring_links_four_neighbors_without_wire_intersections(self):
        points, _, _ = ring(0, 0)
        linked = []
        for row in range(-2, 3):
            for column in range(-2, 3):
                if (row, column) == (0, 0):
                    continue
                other, centre, normal = ring(row, column)
                closest = np.linalg.norm(points[:,None,:]-other[None,:,:], axis=2).min()
                self.assertGreater(closest, 2*WIRE_RADIUS_M)
                relative = points - centre
                heights = relative @ normal
                crossings = 0
                for i in range(len(points)):
                    j = (i+1) % len(points)
                    if heights[i]*heights[j] < 0:
                        fraction = -heights[i]/(heights[j]-heights[i])
                        hit = relative[i] + fraction*(relative[j]-relative[i])
                        if np.linalg.norm(hit) < RING_RADIUS_M:
                            crossings += 1 if heights[j] > heights[i] else -1
                if crossings:
                    self.assertEqual(abs(crossings), 1)
                    linked.append((row,column))
        self.assertEqual(set(linked), {(-1,-1), (-1,0), (1,-1), (1,0)})


if __name__ == "__main__":
    unittest.main()
