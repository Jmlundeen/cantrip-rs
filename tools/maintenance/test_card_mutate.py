"""Tests of what card-mutate.py builds when it builds few cards:

    python -m unittest tools/maintenance/test_card_mutate.py
"""

import importlib.util
import contextlib
import io
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
spec = importlib.util.spec_from_file_location("card_mutate", os.path.join(HERE, "card-mutate.py"))
card_mutate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(card_mutate)


class DeckCards(unittest.TestCase):
    def test_every_way_of_saying_a_card(self):
        text = ("# Mox Ruby in a comment\n[Bot1 Deck]\nMountain\n2 Lightning Bolt\nIsland x20\nIsland X3\n[Bot1 Sideboard]\nFury\n"
                "[Bot2 Companion]\nLurrus\n[Life]\nBot1 7\n[Actions]\nBot1.Hand.Fury.Cast\n")
        self.assertEqual(card_mutate.deck_cards(text), ["Mountain", "Lightning Bolt", "Island", "Island", "Fury", "Lurrus"])


class Played(unittest.TestCase):
    def setUp(self):
        self.tree = tempfile.TemporaryDirectory()
        self.addCleanup(self.tree.cleanup)
        root = self.tree.name
        cards = os.path.join(root, "cards")
        scenarios = os.path.join(root, "scenarios")
        os.mkdir(cards)
        os.mkdir(scenarios)
        for stem, text in (("alpha", "prepare: Some(&crate::beta::CARD)"), ("beta", "use super::*;"), ("gamma", ""), ("filler", ""),
                           ("zeta", "pub static EXTRA: &[&CardDef] = &[&EMBLEM];\n"),
                           ("kappa", "pub static EXTRA: &[&CardDef] = &[\n    &tokens::MAP,\n    &LOCAL,\n];\n"),
                           ("sigma", "use cantrip_mechanics::tokens::SERVO;\npub static EXTRA: &[&CardDef] = &[&SERVO];\n"),
                           ("tau", "use cantrip_mechanics::tokens::{RAT_CANT_BLOCK as RAT, MONK};\npub static EXTRA: &[&CardDef] = &[&RAT];\n"),
                           ("omega", "pub static EXTRA: &[&CardDef] = &[&tokens::MAP];\n")):
            with open(os.path.join(cards, stem + ".rs"), "w", encoding="utf-8") as f:
                f.write(text)
        for name, text in (("a", "[Bot1 Deck]\nAlpha\nFiller x3\n[Bot2 Deck]\n2 Island\n[Actions]\nGamma\n"), ("b", "[Bot1 Deck]\nGamma\n[Actions]\n")):
            with open(os.path.join(scenarios, name + ".cantrip"), "w", encoding="utf-8") as f:
                f.write(text)
        self.saved = card_mutate.new_card.SCRIPTS, card_mutate.new_card.CARDS, card_mutate.new_card.ROOT
        self.addCleanup(self.restore)
        card_mutate.new_card.SCRIPTS, card_mutate.new_card.CARDS = scenarios, cards
        card_mutate.new_card.ROOT = root

    def restore(self):
        card_mutate.new_card.SCRIPTS, card_mutate.new_card.CARDS, card_mutate.new_card.ROOT = self.saved

    def test_the_cards_played_and_the_files_they_name(self):
        self.assertEqual(card_mutate.played("Alpha"), {"alpha", "beta", "filler", "kappa", "sigma", "tau"})

    def test_a_card_only_named_in_the_actions_is_not_played(self):
        self.assertNotIn("gamma", card_mutate.played("Alpha"))

    def test_a_card_in_no_deck_is_only_itself(self):
        self.assertEqual(card_mutate.played("Beta"), {"beta", "kappa", "sigma", "tau"})

    def test_one_file_brings_each_shared_definition(self):
        self.assertEqual(card_mutate.shared_definitions(), {"kappa", "sigma", "tau"})

    def test_imports_and_aliases(self):
        self.assertEqual(card_mutate.imports("use a::b::SERVO;\nuse a::{C as D, E};\n"), {"SERVO": "a::b::SERVO", "D": "a::C", "E": "a::E"})


PASS = "1 scenarios: 1 pass, 0 fail, 0 known bugs, 0 need cards the engine does not have.\n"
FAIL = "  FAIL case\ncase: a check did not hold: Bot1.Life expected 20, got 19\n\nTrace:\n  line 39 ? Bot1.Life = 20\n" \
       "1 scenarios: 0 pass, 1 fail, 0 known bugs, 0 need cards the engine does not have.\n"


class ProofFailures(unittest.TestCase):
    def test_missing_cards_and_non_corpus_errors_never_count_as_caught(self):
        for code, output, expected in (
            (1, "link.exe: access denied", "run error"),
            (1, FAIL.replace("0 need cards", "1 need cards"), "cards missing"),
            (0, PASS.replace("1 pass", "0 pass"), "no scenarios"),
            (1, FAIL.replace("a check did not hold", "does not parse:"), "invalid scenarios"),
        ):
            with self.subTest(expected=expected):
                self.assertEqual(card_mutate.classify_run(code, output)[0], expected)

    def test_any_cargo_failure_prevents_execution(self):
        error = mock.Mock(returncode=1, stdout="", stderr="failed to run custom build command")
        with mock.patch.object(card_mutate.subprocess, "run", return_value=error) as run:
            self.assertEqual(card_mutate.run_scenarios("Alpha", True)[0], "no build")
        self.assertEqual(run.call_count, 1)

    def test_hidden_definition_panic_is_replayed_whole(self):
        panic = "  PANIC case\n" + FAIL
        with mock.patch.object(card_mutate, "run_scenarios", side_effect=[("caught", panic), ("survived", PASS)]) as run:
            with contextlib.redirect_stdout(io.StringIO()):
                verdict, _, few = card_mutate.safe_scenarios("Alpha", True)
        self.assertEqual(verdict, "survived")
        self.assertFalse(few)
        self.assertEqual(run.call_args_list, [mock.call("Alpha", True), mock.call("Alpha", False)])


class SemanticProbes(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.path = self.root / "crates" / "cantrip-cards" / "src" / "cards" / "alpha.rs"
        self.path.parent.mkdir(parents=True)
        self.original = b'\xef\xbb\xbf// caf\xc3\xa9\r\nfn alpha() { effect(1); }\r\n'
        self.path.write_bytes(self.original)
        self.stack = contextlib.ExitStack()
        self.addCleanup(self.stack.close)
        self.stack.enter_context(mock.patch.object(card_mutate, "ROOT", str(self.root)))
        self.stack.enter_context(mock.patch.object(card_mutate, "name_few_cards"))
        self.output = io.StringIO()
        self.stack.enter_context(contextlib.redirect_stdout(self.output))

    def probes(self, **overrides):
        probe = {"name": "wrong amount", "file": self.path.relative_to(self.root).as_posix(), "before": "effect(1)",
                 "after": "effect(2)", "expected_failing_assertion": "Bot1.Life expected 20"}
        probe.update(overrides)
        path = self.root / "probes.json"
        path.write_text(json.dumps([probe]), encoding="utf-8")
        return card_mutate.load_probes(path)

    def test_named_proof_runs_fresh_and_restored_baselines_preserving_exact_bytes(self):
        changes, originals = self.probes()
        observed = []

        def run(card, few):
            data = self.path.read_bytes()
            observed.append(data)
            return ("survived", PASS) if data == self.original else ("caught", FAIL)

        with mock.patch.object(card_mutate, "run_scenarios", side_effect=run):
            self.assertEqual(card_mutate.run_changes("Alpha", changes, originals), 0)
        self.assertEqual(observed, [self.original, changes[0]["changed"], self.original])
        self.assertEqual(self.path.read_bytes(), self.original)

    def test_interruption_restores_source_and_executes_restored_baseline(self):
        changes, originals = self.probes()
        observed = []

        def run(card, few):
            data = self.path.read_bytes()
            observed.append(data)
            if data != self.original:
                raise KeyboardInterrupt()
            return "survived", PASS

        with mock.patch.object(card_mutate, "run_scenarios", side_effect=run):
            with self.assertRaises(KeyboardInterrupt):
                card_mutate.run_changes("Alpha", changes, originals)
        self.assertEqual(self.path.read_bytes(), self.original)
        self.assertEqual(observed[-1], self.original)
        self.assertEqual(len(observed), 3)

    def test_mutated_missing_dependency_is_survivor_when_whole_registry_passes(self):
        changes, originals = self.probes()

        def run(card, few):
            if self.path.read_bytes() == self.original:
                return "survived", PASS
            return ("cards missing", FAIL.replace("0 need cards", "1 need cards")) if few else ("survived", PASS)

        with mock.patch.object(card_mutate, "run_scenarios", side_effect=run):
            self.assertEqual(card_mutate.run_changes("Alpha", changes, originals), 1)
        self.assertIn("0 caught, 1 survived", self.output.getvalue())
        self.assertEqual(self.path.read_bytes(), self.original)

    def test_expected_assertion_printed_only_in_trace_is_not_proof(self):
        changes, originals = self.probes(expected_failing_assertion="line 39 ? Bot1.Life = 20")
        with mock.patch.object(card_mutate, "run_scenarios", side_effect=[("survived", PASS), ("caught", FAIL), ("survived", PASS)]):
            self.assertEqual(card_mutate.run_changes("Alpha", changes, originals), 1)
        self.assertIn("wrong assertion", self.output.getvalue())
        self.assertIn("0 caught", self.output.getvalue())

    def test_build_error_and_restored_failure_are_not_successful_semantic_proof(self):
        changes, originals = self.probes()
        with mock.patch.object(card_mutate, "run_scenarios", side_effect=[("survived", PASS), ("no build", "error"),
                                                                        ("no build", "error"), ("survived", PASS)]):
            self.assertEqual(card_mutate.run_changes("Alpha", changes, originals), 1)
        self.assertIn("0 caught", self.output.getvalue())
        with mock.patch.object(card_mutate, "run_scenarios", side_effect=[("survived", PASS), ("caught", FAIL),
                                                                        ("caught", FAIL)]):
            self.assertEqual(card_mutate.run_changes("Alpha", changes, originals, False), 1)
        self.assertIn("restored baseline failed", self.output.getvalue())

    def test_rejects_escaping_paths_and_ambiguous_or_noop_replacements(self):
        for changes in ({"file": "../outside.rs"}, {"file": str(self.path)}, {"before": "absent"},
                        {"before": ""}, {"after": "effect(1)"}, {"expected_failing_assertion": ""}):
            with self.subTest(changes=changes), self.assertRaises(ValueError):
                self.probes(**changes)
        self.path.write_bytes(self.original + b'\n// effect(1)')
        with self.assertRaises(ValueError):
            self.probes()


if __name__ == "__main__":
    unittest.main()
