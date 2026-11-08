"""Reference indexes and the direct Oracle gate, without network or Rust builds."""

import contextlib
import importlib.util
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
spec = importlib.util.spec_from_file_location("new_card_under_test", HERE / "new-card.py")
new_card = importlib.util.module_from_spec(spec)
spec.loader.exec_module(new_card)


class ReferenceIndexes(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.content = self.root / "content" / "mtg" / "cards"
        self.scenarios = self.root / "scenarios"
        self.content.mkdir(parents=True)
        self.scenarios.mkdir()
        self.stack = contextlib.ExitStack()
        self.addCleanup(self.stack.close)
        for key, value in (("ROOT", self.root), ("CONTENT", self.root / "content"), ("SCRIPTS", self.scenarios)):
            self.stack.enter_context(mock.patch.object(new_card, key, str(value)))

    def test_content_index_follows_edits_additions_deletions_and_reads_selected_source(self):
        path = self.content / "alpha.json"
        path.write_text(json.dumps({"name": "Alpha", "oracleText": "one"}), encoding="utf-8")
        self.assertEqual(new_card.find_card("Alpha")[3]["oracleText"], "one")
        self.assertEqual(new_card.find_card("Alpha")[3]["oracleText"], "one")
        path.write_text(json.dumps({"name": "Bravo", "oracleText": "two"}), encoding="utf-8")
        self.assertIsNone(new_card.find_card("Alpha"))
        self.assertEqual(new_card.find_card("Bravo")[3]["oracleText"], "two")
        second = self.content / "second.json"
        second.write_text(json.dumps({"name": "Alpha"}), encoding="utf-8")
        self.assertEqual(new_card.find_card("Alpha")[1], str(second))
        second.unlink()
        self.assertIsNone(new_card.find_card("Alpha"))

    def test_scenario_index_matches_old_selection_and_invalidates_changed_files(self):
        cases = {"a": "[Bot1 Deck]\n2 Alpha\nIsland x20\n[Actions]\n", "b": "# Alpha\n[Bot1 Deck]\nBravo\n",
                 "c": "[Bot2 Deck]\nALPHA X3\n", "d": "[Bot1 Deck]\nAſh\n", "e": "[Bot1 Deck]\n1996 World Champion\n"}
        for name, text in cases.items():
            (self.scenarios / (name + ".cantrip")).write_text(text, encoding="utf-8")
        for wanted in ("Alpha", "Ash", "1996 World Champion"):
            pattern = new_card.re.compile(r"^\s*(?:\d+\s+)?" + new_card.re.escape(wanted) + r"(?:\s+x\d+)?\s*$",
                                          new_card.re.M | new_card.re.I)
            expected = ["scenarios/" + name + ".cantrip" for name, text in sorted(cases.items()) if pattern.search(text)]
            self.assertEqual(new_card.scenario_files(wanted), expected)
            self.assertEqual(new_card.scenario_files(wanted), expected)
        (self.scenarios / "b.cantrip").write_text("[Bot1 Deck]\nAlpha\n", encoding="utf-8")
        (self.scenarios / "a.cantrip").unlink()
        (self.scenarios / "new.cantrip").write_text("[Bot1 Deck]\nAlpha x4\n", encoding="utf-8")
        self.assertEqual(new_card.scenario_files("Alpha"), ["scenarios/b.cantrip", "scenarios/c.cantrip", "scenarios/new.cantrip"])

    def test_repeated_prior_art_avoids_index_build_and_source_edits_invalidate_it(self):
        path = self.root / "crates" / "example" / "src" / "lib.rs"
        path.parent.mkdir(parents=True)
        path.write_text("one", encoding="utf-8")
        index = mock.Mock()
        index.report.return_value = "prior art"
        card = {"name": "Alpha"}
        self.assertEqual(new_card.prior_art_report(card, index, []), "prior art")
        self.assertEqual(new_card.prior_art_report(card, index, []), "prior art")
        self.assertEqual(index.Index.call_count, 1)
        path.write_text("different", encoding="utf-8")
        new_card.prior_art_report(card, index, [])
        self.assertEqual(index.Index.call_count, 2)


def face(name="Alpha", **extra):
    return {"name": name, "mana_cost": "{1}{U}", "type_line": "Sorcery", "oracle_text": "Draw a card."} | extra


class DirectOracle(unittest.TestCase):
    def test_exact_characteristics_and_text_without_building_prior_art(self):
        card = {"name": "Alpha", "manaCost": "{1}{U}", "type": "Sorcery", "oracleText": "Draw a card."}
        found = {"name": "Alpha", "faces": [face()], "rulings": []}
        index = new_card._ability_index()
        with mock.patch.object(index, "Index", side_effect=AssertionError("prior art must not be built")):
            self.assertTrue(new_card.oracle_comparison(card, found, index)["matches"])
            for key, value in (("manaCost", "{U}"), ("type", "Instant"), ("oracleText", "Draw two cards.")):
                result = new_card.oracle_comparison(card | {key: value}, found, index)
                self.assertFalse(result["matches"])
                self.assertEqual(result["differences"][0]["field"], key)

    def test_all_dfc_and_split_faces_must_match(self):
        front = {"name": "Alpha", "manaCost": "{1}{U}", "type": "Sorcery", "oracleText": "Draw a card."}
        back = {"name": "Bravo", "manaCost": "{1}{U}", "type": "Sorcery", "oracleText": "Draw a card."}
        found = {"name": "Alpha // Bravo", "faces": [face(), face("Bravo")], "rulings": []}
        self.assertTrue(new_card.oracle_comparison(front | {"backFace": back}, found)["matches"])
        self.assertFalse(new_card.oracle_comparison(front, found)["matches"])
        split = {"name": "Alpha // Bravo", "oracleText": "combined text", "halves": [front, back]}
        self.assertTrue(new_card.oracle_comparison(split, found)["matches"])
        split["halves"][1] = back | {"manaCost": "{U}"}
        self.assertFalse(new_card.oracle_comparison(split, found)["matches"])

    def test_unavailable_evidence_fails_and_cli_never_runs_full_report(self):
        card = {"name": "Alpha", "manaCost": "{1}{U}", "type": "Sorcery", "oracleText": "Draw a card."}
        index = new_card._ability_index()
        with mock.patch.object(new_card.scryfall, "card", return_value=None):
            self.assertFalse(new_card.oracle_comparison(card)["matches"])
        with mock.patch.object(sys, "argv", ["new-card.py", "Alpha", "--oracle-only"]), \
             mock.patch.object(new_card, "find_card", return_value=("mtg", "alpha.json", "", card)), \
             mock.patch.object(new_card.scryfall, "card", return_value={"name": "Alpha", "faces": [face()]}), \
             mock.patch.object(new_card, "_ability_index", return_value=index), \
             mock.patch.object(new_card, "scenario_files", side_effect=AssertionError("corpus scanned")), \
             mock.patch.object(new_card, "prior_art_report", side_effect=AssertionError("prior art built")), \
             contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(new_card.main(), 0)
        self.assertIn("Oracle text matches Scryfall's", output.getvalue())


if __name__ == "__main__":
    unittest.main()
