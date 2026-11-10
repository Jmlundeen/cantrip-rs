"""`done_anywhere` counts only cards git knows about: the lane scaffolds a card's content file before anyone works it."""

import importlib.util
import os
import subprocess
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))


def load_issues():
    spec = importlib.util.spec_from_file_location("card_issues", os.path.join(HERE, "card-issues.py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class TrackedCardsTest(unittest.TestCase):
    def setUp(self):
        self.root = tempfile.mkdtemp()
        cards = os.path.join(self.root, "content", "mtg", "cards")
        os.makedirs(cards)
        for name in ("a-real-card", "b-scaffold"):
            with open(os.path.join(cards, name + ".json"), "w", encoding="utf-8") as f:
                f.write('{"name": "A Card"}' + chr(10))
        subprocess.run(["git", "-C", self.root, "init", "-q"], check=True)
        subprocess.run(["git", "-C", self.root, "add", "content/mtg/cards/a-real-card.json"], check=True)
        subprocess.run(["git", "-C", self.root, "commit", "-qm", "one card"], check=True)

    def test_a_scaffold_the_lane_just_wrote_is_not_a_card_done(self):
        issues = load_issues()
        cards = issues.tracked_cards(self.root)
        self.assertIn("a-real-card", cards)
        self.assertNotIn("b-scaffold", cards)


if __name__ == "__main__":
    unittest.main()
