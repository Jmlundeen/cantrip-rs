"""Behavioral regressions for canonical intake, actual input receipts and lost handoffs."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import unittest
from unittest import mock
from tools.maintenance import test_card_lane as fixtures


class RuntimeTest(unittest.TestCase):
    setUp = fixtures.LaneTest.setUp
    tearDown = fixtures.LaneTest.tearDown
    git = fixtures.LaneTest.git
    write = fixtures.LaneTest.write
    lane = fixtures.LaneTest.lane
    queue = fixtures.LaneTest.queue
    state = fixtures.LaneTest.state
    status = fixtures.LaneTest.status
    worktree = fixtures.LaneTest.worktree
    card_b_scenario = fixtures.LaneTest.card_b_scenario

    def test_upstream_inventory_ignores_stale_origin_and_dirty_checkout(self):
        old = self.git(self.main, "rev-parse", "HEAD")
        self.git(self.main, "remote", "add", "upstream", self.origin)
        self.write("content/mtg/cards/card-b.json", json.dumps({"name": "Card B", "faces": [{"name": "Face B"}]}))
        self.git(self.main, "add", ".")
        self.git(self.main, "commit", "--quiet", "-m", "upstream card")
        new = self.git(self.main, "rev-parse", "HEAD")
        self.git(self.main, "push", "--quiet", "origin", "main")
        self.git(self.main, "update-ref", "refs/remotes/origin/main", old)
        self.git(self.main, "switch", "--quiet", "-c", "stale", old)
        self.write("content/mtg/cards/card-c.json", json.dumps({"name": "Card C"}))
        made = self.queue("--card", "Card B", "--card", "Face B", "--card", "Card C")
        self.assertEqual(made.returncode, 0, made.stderr)
        self.assertEqual([c["name"] for c in self.state()["cards"]], ["Card C"])
        self.assertEqual(self.state()["run"]["base_sha"], new)
        self.assertEqual(self.git(self.main, "rev-parse", "origin/main"), old)

    def test_refresh_retires_pending_cards_and_preserves_working_claim(self):
        self.assertEqual(self.queue("--card", "Card B", "--card", "Card C").returncode, 0)
        worker = self.worktree("author")
        self.assertEqual(self.lane(worker, "next", "--queue", "q", "--id", "w1").returncode, 0)
        before = self.state()
        for name in ("B", "C"):
            self.write(f"content/mtg/cards/card-{name.lower()}.json", json.dumps({"name": f"Card {name}"}))
        self.git(self.main, "add", ".")
        self.git(self.main, "commit", "--quiet", "-m", "new upstream cards")
        self.git(self.main, "push", "--quiet")
        made = self.queue("--card", "Card B", "--card", "Card C")
        self.assertEqual(made.returncode, 0, made.stderr)
        after = self.state()
        self.assertEqual(after["cards"][0]["claim"], before["cards"][0]["claim"])
        self.assertEqual(self.status(after, "Card B"), "working")
        self.assertEqual(self.status(after, "Card C"), "done")
        self.assertEqual(after["run"]["base_sha"], before["run"]["base_sha"])

    def test_fingerprint_detects_hidden_worktree_and_ignored_evidence_edits(self):
        lane = fixtures.load_lane()
        self.queue("--card", "Card B")
        store = lane.Store(lane.common_dir(self.main), "q")
        card = {"name": "Card B"}
        first = lane.runtime.fingerprint(lane.api(), self.main, store, card)
        self.git(self.main, "update-index", "--assume-unchanged", "docs/readme.txt")
        self.write("docs/readme.txt", "hidden change\n")
        self.assertNotEqual(first, lane.runtime.fingerprint(lane.api(), self.main, store, card))
        second = lane.runtime.fingerprint(lane.api(), self.main, store, card)
        self.write("tools/maintenance/custom-helper.py", "# bundle helper changed\n")
        self.assertNotEqual(second, lane.runtime.fingerprint(lane.api(), self.main, store, card))
        second = lane.runtime.fingerprint(lane.api(), self.main, store, card)
        self.write(".card-lane/q/card-b/report.json", '{"issue":99}')
        self.assertNotEqual(second, lane.runtime.fingerprint(lane.api(), self.main, store, card))

    def test_external_lane_uses_its_companion_gate_tools_at_the_callers_root(self):
        lane = fixtures.load_lane()
        root = self.worktree("old-base")
        with mock.patch.dict(os.environ, {}, clear=False):
            os.environ.pop("CANTRIP_LANE_TOOLS_DIR", None)
            output = mock.Mock(returncode=0, stdout="1 scenarios: 1 pass, 0 fail, 0 known bugs, 0 need cards", stderr="")
            with mock.patch.object(lane, "run", return_value=output) as executed:
                lane.gate_corpus(root, {"name": "Card B"}, [])
            self.assertEqual(executed.call_args.args[0][1], os.path.join(lane.HERE, "card-mutate.py"))
            self.assertEqual(executed.call_args.args[1], root)
            output.stdout = "Oracle text matches Scryfall's current Oracle text"
            with mock.patch.object(lane, "run", return_value=output) as executed:
                lane.gate_oracle(root, {"name": "Card B"})
            self.assertEqual(executed.call_args.args[0][1], os.path.join(lane.HERE, "new-card.py"))
            self.assertIn("--oracle-only", executed.call_args.args[0])

    def test_full_receipt_reuse_requires_all_proof_and_unchanged_inputs(self):
        lane = fixtures.load_lane()
        self.queue("--card", "Card B")
        store = lane.Store(lane.common_dir(self.main), "q")
        card = {"name": "Card B"}
        gates = [{"name": n, "status": "pass"} for n in
                 ("scope", "oracle", "scenarios", "corpus", "lint", "citations", "report", "coverage", "mutation")]
        lane.save_check(self.main, store, card, gates, False)
        self.assertEqual(lane.runtime.reusable(lane.api(), self.main, store, card), gates)
        self.write("scenarios/new.cantrip", "new source")
        self.assertIsNone(lane.runtime.reusable(lane.api(), self.main, store, card))
        lane.save_check(self.main, store, card, gates, True)
        self.assertIsNone(lane.runtime.reusable(lane.api(), self.main, store, card))

    def test_finish_recovers_committed_author_without_chat_and_keeps_log_retry(self):
        lane = fixtures.load_lane()
        self.git(self.main, "switch", "--quiet", "-c", "card/finish")
        self.queue("--card", "Card B")
        self.assertEqual(self.lane(self.main, "next", "--queue", "q", "--id", "w1", "--issue", "9").returncode, 0)
        self.card_b_scenario()
        saved = os.getcwd()
        os.chdir(self.main)
        self.addCleanup(os.chdir, saved)
        gates = [{"name": "scope", "status": "pass", "summary": "checks out", "detail": "", "facts": {}}]
        args = argparse.Namespace(queue="q", id="w1", issue=None, note="")
        with mock.patch.object(lane, "check_card", return_value=gates):
            self.assertEqual(lane.cmd_commit(args), 0)
        self.assertEqual(self.state()["cards"][0]["review"]["state"], "awaiting-finish")
        self.assertEqual(lane.cmd_finish(args), 0)
        store = lane.Store(lane.common_dir(self.main), "q")
        ready = lane.runtime.ready(lane.api(), self.main, store)
        self.assertEqual(len(ready), 1)
        self.assertEqual(ready[0]["sha"], self.git(self.main, "rev-parse", "HEAD"))
        self.assertEqual(ready[0]["delivery"]["state"], "retry")
        self.assertTrue(Path(ready[0]["packet"], "check.json").is_file())
        self.assertEqual(lane.cmd_finish(args), 0)
        self.assertEqual(self.git(self.main, "rev-parse", "main"), self.git(self.main, "rev-parse", "origin/main"))

    def test_corpus_accounting_rejects_stale_incomplete_or_failed_run(self):
        spec = importlib.util.spec_from_file_location("corpus_report", Path(__file__).with_name("corpus-report.py"))
        report = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(report)
        data = {"schema": 1, "run_id": "fresh", "expected": 2, "completed": 2, "complete": True,
                "passed": 1, "failed": 0, "known_bugs": 0, "fixed": 0, "missing": 1, "missing_cards": {"X": 1}}
        self.assertEqual(report.validate(data, "fresh"), data)
        for changed in ({"run_id": "old"}, {"complete": False}, {"completed": 1}, {"fixed": 1, "passed": 0}):
            with self.assertRaises(ValueError):
                report.validate(dict(data, **changed), "fresh")


if __name__ == "__main__":
    unittest.main()
