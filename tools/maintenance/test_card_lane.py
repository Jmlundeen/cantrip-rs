"""Tests of the card lane (card-lane.py), each against a throwaway origin and clone:

    python -m unittest tools/maintenance/test_card_lane.py

The real tool runs against a small fake Cantrip repo whose `new-card.py` writes a content file and a skeleton. No
network, no cargo, no actual card: what is under test is the shared queue, the claim ids, the packet, the pool state
and the shared branch.
"""

import argparse
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

sys.dont_write_bytecode = True

HERE = os.path.dirname(os.path.abspath(__file__))
SCRIPT = os.path.join(HERE, "card-lane.py")
GIT_ENV = {"GIT_AUTHOR_NAME": "Test", "GIT_AUTHOR_EMAIL": "test@example.com",
           "GIT_COMMITTER_NAME": "Test", "GIT_COMMITTER_EMAIL": "test@example.com"}


def load_lane():
    spec = importlib.util.spec_from_file_location("card_lane_under_test", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

NEW_CARD = r'''import json, os, re, sys

root = os.getcwd()
name = sys.argv[1]
slug = re.sub(r"[^a-z0-9]+", "-", name.lower()).strip("-")
cards = os.path.join(root, "content", "mtg", "cards")
os.makedirs(cards, exist_ok=True)
path = os.path.join(cards, slug + ".json")
if not os.path.exists(path):
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        json.dump({"name": name, "oracleText": ""}, f)
code = os.path.join(root, "crates", "cantrip-cards", "src", "cards", slug.replace("-", "_") + ".rs")
os.makedirs(os.path.dirname(code), exist_ok=True)
written = not os.path.exists(code)
if written:
    with open(code, "w", encoding="utf-8", newline="\n") as f:
        f.write("use cantrip_api::*;\n\npub static CARD: CardDef = CardDef { ..oracle::CARD };\n")
print(f"=== {name}  (content/mtg/cards/{slug}.json)  oracle::CARD")
print("Oracle text for " + name)
print()
print("=== Scenarios naming it")
print("  scenarios/some-card-does-things.cantrip")
print()
print("=== Scryfall: current Oracle text and official rulings (tools/maintenance/scryfall.py)")
print("  none")
print()
print("=== Prior art: the implemented cards that already do each ability (tools/maintenance/ability-index.py)")
print("  0.90  Some Card  crates/cantrip-cards/src/cards/some_card.rs")
print()
print("=== Test points: each needs an expectation that fails if the card gets it wrong")
print("  - it does the thing")
relative = os.path.relpath(code, root).replace(os.sep, "/")
print(("\n=== wrote " if written else "\n=== ") + relative + (" (characteristics only; add the behavior)" if written else " exists:"))
'''


class LaneTest(unittest.TestCase):
    def setUp(self):
        self.saved_env = {key: os.environ.get(key) for key in GIT_ENV}
        os.environ.update(GIT_ENV)
        self.tmp = tempfile.mkdtemp(prefix="card-lane-test-")
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        self.origin = os.path.join(self.tmp, "origin.git")
        self.git(self.tmp, "init", "--quiet", "--bare", "-b", "main", self.origin)
        self.main = os.path.join(self.tmp, "main")
        self.git(self.tmp, "clone", "--quiet", self.origin, self.main)
        self.git(self.main, "checkout", "--quiet", "-b", "main")
        self.write(".gitignore", ".card-lane/\n.claude/worktrees/\ntarget/\n")
        self.write("docs/readme.txt", "tracked text\n")
        self.write("docs/formats/test.json", json.dumps(
            {"format": "test", "cards": [{"name": "Card B"}, {"name": "Card C"}, {"name": "Card D"}]}))
        self.write("content/mtg/cards/card-a.json", json.dumps({"name": "Card A"}))
        self.write("crates/cantrip-cards/src/cards/some_card.rs", "use cantrip_api::*;\n\n// prior art\n")
        self.write("scenarios/some-card-does-things.cantrip", "# a prior scenario\n\n[Bot1 Deck]\nIsland x20\n")
        self.write("tools/maintenance/new-card.py", NEW_CARD)
        shutil.copyfile(os.path.join(HERE, "missing-cards.py"),
                        os.path.join(self.main, "tools", "maintenance", "missing-cards.py"))
        self.write("docs/cube-lists/TestCube.txt", "Card A\nCard B\nCard C\nCard E\n")
        self.write("docs/cube-lists/AllImplemented.txt", "Card A\n")
        self.git(self.main, "add", ".")
        self.git(self.main, "commit", "--quiet", "-m", "initial")
        self.git(self.main, "push", "--quiet", "-u", "origin", "main")

    def tearDown(self):
        for key, value in self.saved_env.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value

    # -- helpers -----------------------------------------------------------------------------------------------------

    def git(self, cwd, *args, check=True):
        result = subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True, encoding="utf-8")
        if check and result.returncode != 0:
            raise AssertionError(f"git {' '.join(args)} failed:\n{result.stdout}\n{result.stderr}")
        return result.stdout.strip()

    def write(self, relative, text, root=None):
        path = os.path.join(root or self.main, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(text)
        return path

    def lane(self, cwd, *args):
        env = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
        return subprocess.run([sys.executable, SCRIPT, *args], cwd=cwd, capture_output=True, text=True,
                              encoding="utf-8", errors="replace", env=env)

    def queue(self, *args, cwd=None, name="q"):
        return self.lane(cwd or self.main, "queue", "--queue", name, *args)

    def state(self, name="q"):
        with open(os.path.join(self.main, ".git", "card-lane", name, "queue.json"), encoding="utf-8") as f:
            return json.load(f)

    def status(self, state, card):
        return next(entry["status"] for entry in state["cards"] if entry["name"] == card)

    def worktree(self, name):
        path = os.path.join(self.tmp, name)
        self.git(self.main, "worktree", "add", "--quiet", "-B", name, path, "origin/main")
        return path

    def pool_slot(self, slot="pool-1", member=True, leased=True, read_only=False):
        path = os.path.join(self.main, ".claude", "worktrees", slot)
        if not os.path.isdir(path):
            self.git(self.main, "worktree", "add", "--quiet", "-B", slot, path, "origin/main")
        state = {"slots": {slot: {"state": "leased" if leased else "free", "note": ""}}, "leases": {},
                 "members": {}, "tokens": {}, "log": []}
        if leased:
            state["leases"]["L-1"] = {"slot": slot, "purpose": "card: Card B", "name": "", "acquired": "2026-01-01T00:00:00+00:00"}
            if member:
                state["members"]["M-1"] = {"lease": "L-1", "agent": "impl-a", "name": "impl", "read_only": read_only,
                                           "joined": "2026-01-01T00:00:00+00:00"}
        with open(os.path.join(self.main, ".git", "cantrip-worktrees.json"), "w", encoding="utf-8") as f:
            json.dump(state, f)
        return path

    # -- the queue ---------------------------------------------------------------------------------------------------

    def test_queue_is_shared_and_skips_implemented_cards(self):
        made = self.queue("--format", "test")
        self.assertEqual(made.returncode, 0, made.stderr)
        queue_path = os.path.join(self.main, ".git", "card-lane", "q", "queue.json")
        self.assertTrue(os.path.isfile(queue_path))
        self.assertEqual([card["name"] for card in self.state()["cards"]], ["Card B", "Card C", "Card D"])

        other = self.worktree("worker-two")
        shown = self.lane(other, "status", "--queue", "q")
        self.assertEqual(shown.returncode, 0, shown.stderr)
        self.assertIn("queue q", shown.stdout)
        self.assertIn("3 card(s)", shown.stdout)

        skipped = self.queue("--card", "Card A", "--card", "Card E")
        self.assertEqual(skipped.returncode, 0, skipped.stderr)
        names = [card["name"] for card in self.state()["cards"]]
        self.assertEqual(names, ["Card B", "Card C", "Card D", "Card E"])
        self.assertIn("implemented, skipped: Card A", skipped.stdout)

    def test_queue_keeps_earlier_cards(self):
        self.assertEqual(self.queue("--card", "Card C").returncode, 0)
        self.assertEqual(self.queue("--card", "Card D").returncode, 0)
        self.assertEqual([card["name"] for card in self.state()["cards"]], ["Card C", "Card D"])

    def test_queue_branch_is_cut_from_remote_main_not_the_lane_checkout(self):
        lane = self.worktree("lane")
        self.write("tools/maintenance/card-lane.py", "# the lane's own tool\n", root=lane)
        self.git(lane, "add", "tools/maintenance/card-lane.py")
        self.git(lane, "commit", "--quiet", "-m", "lane: the card lane")
        lane_head = self.git(lane, "rev-parse", "HEAD")
        self.assertNotEqual(lane_head, self.git(self.main, "rev-parse", "origin/main"))

        made = self.queue("--card", "Card B", cwd=lane)
        self.assertEqual(made.returncode, 0, made.stderr)
        self.assertEqual(self.git(self.main, "rev-parse", "card-lane/q"),
                         self.git(self.main, "rev-parse", "origin/main"))

        refused = self.lane(lane, "next", "--id", "w1", "--queue", "q")
        self.assertEqual(refused.returncode, 2)
        self.assertIn("not a queue worktree", refused.stderr)
        self.assertIn("lane's own files", refused.stderr)
        self.assertEqual(self.status(self.state(), "Card B"), "pending")

    def test_a_queue_branch_with_the_lanes_own_work_is_refused(self):
        lane = self.worktree("lane")
        self.write("tools/maintenance/card-lane.py", "# the lane's own tool\n", root=lane)
        self.git(lane, "add", "tools/maintenance/card-lane.py")
        self.git(lane, "commit", "--quiet", "-m", "lane: the card lane")
        lane_head = self.git(lane, "rev-parse", "HEAD")
        self.git(self.main, "branch", "card-lane/q", lane_head)

        refused = self.queue("--card", "Card B")
        self.assertEqual(refused.returncode, 2)
        self.assertIn("lane's own work", refused.stderr)
        self.assertEqual(self.git(self.main, "rev-parse", "card-lane/q"), lane_head)

    def test_a_claim_opened_from_lane_work_cannot_be_shared(self):
        lane = self.worktree("lane")
        self.write("tools/maintenance/card-lane.py", "# the lane's own tool\n", root=lane)
        self.git(lane, "add", "tools/maintenance/card-lane.py")
        self.git(lane, "commit", "--quiet", "-m", "lane: the card lane")
        lane_head = self.git(lane, "rev-parse", "HEAD")
        self.assertEqual(self.queue("--card", "Card B", cwd=lane).returncode, 0)
        provided = load_lane()
        store = provided.Store(os.path.join(self.main, ".git"), "q")
        with self.assertRaises(provided.Refused) as caught:
            provided.require_claim_base(lane, store, {"head": lane_head})
        self.assertIn("lane's own work", str(caught.exception))

    def test_queue_backlog_uses_missing_cards_in_its_order(self):
        made = self.queue("--backlog")
        self.assertEqual(made.returncode, 0, made.stderr)
        self.assertIn("backlog: 3 missing card(s)", made.stdout)
        self.assertIn("added 3", made.stdout)
        self.assertEqual([card["name"] for card in self.state()["cards"]], ["Card B", "Card C", "Card E"])

        again = self.queue("--backlog")
        self.assertEqual(again.returncode, 0, again.stderr)
        self.assertIn("already queued 3", again.stdout)

        one = self.queue("--backlog", "--list", "TestCube", name="one")
        self.assertEqual(one.returncode, 0, one.stderr)
        self.assertIn("cube list TestCube", one.stdout)
        self.assertEqual([card["name"] for card in self.state("one")["cards"]], ["Card B", "Card C", "Card E"])

        refused = self.queue("--backlog", "--card", "Card E")
        self.assertEqual(refused.returncode, 2)
        self.assertIn("own source", refused.stderr)
        without = self.queue("--list", "TestCube")
        self.assertEqual(without.returncode, 2)
        self.assertIn("--backlog", without.stderr)

    def test_queue_backlog_when_nothing_is_missing_is_a_no_op(self):
        made = self.queue("--backlog", "--list", "AllImplemented", name="empty")
        self.assertEqual(made.returncode, 0, made.stderr)
        self.assertIn("nothing to queue", made.stdout)
        self.assertFalse(os.path.exists(os.path.join(self.main, ".git", "card-lane", "empty", "queue.json")))

    # -- claims and packets ------------------------------------------------------------------------------------------

    def test_next_claims_one_card_and_the_same_id_gets_it_back(self):
        self.queue("--format", "test")
        first = self.lane(self.main, "next", "--id", "w1", "--queue", "q")
        self.assertEqual(first.returncode, 0, first.stderr)
        self.assertIn("# Card: Card B", first.stdout)
        packet = os.path.join(self.main, ".card-lane", "q", "card-b")
        for entry in ("brief.md", "reference.md", "prior-art.md", "content.json", "state.json"):
            self.assertTrue(os.path.isfile(os.path.join(packet, entry)), entry)
        with open(os.path.join(packet, "reference.md"), encoding="utf-8") as f:
            reference = f.read()
        self.assertIn("Oracle text for Card B", reference)
        with open(os.path.join(packet, "prior-art.md"), encoding="utf-8") as f:
            prior = f.read()
        self.assertIn("some_card.rs", prior)
        self.assertIn("some-card-does-things.cantrip", prior)

        again = self.lane(self.main, "next", "--id", "w1", "--queue", "q")
        self.assertEqual(again.returncode, 0, again.stderr)
        self.assertIn("# Card: Card B", again.stdout)
        state = self.state()
        working = [card for card in state["cards"] if card["status"] == "working"]
        self.assertEqual([card["name"] for card in working], ["Card B"])
        self.assertEqual(state["cards"][0]["attempts"], 1)

        same_tree = self.lane(self.main, "next", "--id", "w2", "--queue", "q")
        self.assertEqual(same_tree.returncode, 2)
        self.assertIn("one open card per worktree", same_tree.stderr.lower())

        other = self.worktree("worker-two")
        second = self.lane(other, "next", "--id", "w2", "--queue", "q")
        self.assertEqual(second.returncode, 0, second.stderr)
        self.assertIn("# Card: Card C", second.stdout)
        self.assertTrue(os.path.isdir(os.path.join(other, ".card-lane", "q", "card-c")))

    def test_next_skips_a_blocked_card_and_a_released_card_comes_back(self):
        self.queue("--format", "test")
        self.assertEqual(self.lane(self.main, "next", "--id", "w1", "--queue", "q").returncode, 0)
        self.write("docs/readme.txt", "tracked text\nchanged while working\n")
        blocked = self.lane(self.main, "block", "--id", "w1", "--queue", "q", "--rule", "601.2c", "--note", "needs a rule")
        self.assertEqual(blocked.returncode, 0, blocked.stderr)
        self.assertEqual(self.status(self.state(), "Card B"), "blocked")
        with open(os.path.join(self.main, ".card-lane", "q", "card-b", "work.patch"), encoding="utf-8") as f:
            self.assertIn("changed while working", f.read())

        self.git(self.main, "checkout", "--", "docs/readme.txt")
        self.git(self.main, "clean", "-fd")
        nxt = self.lane(self.main, "next", "--id", "w1", "--queue", "q")
        self.assertEqual(nxt.returncode, 0, nxt.stderr)
        self.assertIn("# Card: Card C", nxt.stdout)

        self.assertEqual(self.lane(self.main, "release", "--id", "w1", "--queue", "q", "--reason", "stopping").returncode, 0)
        self.assertEqual(self.status(self.state(), "Card C"), "pending")
        other = self.worktree("worker-two")
        again = self.lane(other, "next", "--id", "w2", "--queue", "q")
        self.assertEqual(again.returncode, 0, again.stderr)
        self.assertIn("# Card: Card C", again.stdout)

    def test_done_shares_the_branch(self):
        self.queue("--format", "test")
        self.assertEqual(self.lane(self.main, "next", "--id", "w1", "--queue", "q").returncode, 0)
        self.write("card-b-work.txt", "the card's work\n")
        self.git(self.main, "add", "content", "crates", "card-b-work.txt")
        self.git(self.main, "commit", "--quiet", "-m", "Card: Card B")
        head = self.git(self.main, "rev-parse", "HEAD")
        done = self.lane(self.main, "done", "--id", "w1", "--queue", "q")
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(self.status(self.state(), "Card B"), "done")
        self.assertEqual(self.git(self.main, "rev-parse", "card-lane/q"), head)

        other = self.worktree("worker-two")
        nxt = self.lane(other, "next", "--id", "w2", "--queue", "q")
        self.assertEqual(nxt.returncode, 0, nxt.stderr)
        self.assertIn("# Card: Card C", nxt.stdout)
        self.assertTrue(os.path.isfile(os.path.join(other, "card-b-work.txt")), "the shared branch was not merged in")

        again = self.lane(self.main, "next", "--id", "w1", "--queue", "q")
        self.assertEqual(again.returncode, 0, again.stderr)
        self.assertIn("# Card: Card D", again.stdout)

    def test_share_publishes_a_mechanics_commit_for_the_next_worker(self):
        self.queue("--format", "test")
        self.assertEqual(self.lane(self.main, "next", "--id", "w1", "--queue", "q").returncode, 0)
        self.write("crates/cantrip-mechanics/src/shared_thing.rs", "// one shared recipe\n")
        self.git(self.main, "add", "crates/cantrip-mechanics/src/shared_thing.rs")
        self.git(self.main, "commit", "--quiet", "-m", "Mechanics: one shared recipe")
        head = self.git(self.main, "rev-parse", "HEAD")
        shared = self.lane(self.main, "share", "--id", "w1", "--queue", "q")
        self.assertEqual(shared.returncode, 0, shared.stderr)
        self.assertEqual(self.git(self.main, "rev-parse", "card-lane/q"), head)
        self.assertIn("Mechanics: one shared recipe", shared.stdout)

        other = self.worktree("worker-two")
        nxt = self.lane(other, "next", "--id", "w2", "--queue", "q")
        self.assertEqual(nxt.returncode, 0, nxt.stderr)
        self.assertTrue(os.path.isfile(os.path.join(other, "crates", "cantrip-mechanics", "src", "shared_thing.rs")))

    def test_done_refuses_an_unchanged_tree_and_no_commit_accepts_it(self):
        self.queue("--card", "Card Z")
        self.write("content/mtg/cards/card-z.json", json.dumps({"name": "Card Z"}))
        self.write("crates/cantrip-cards/src/cards/card_z.rs", "use cantrip_api::*;\n")
        self.git(self.main, "add", "content", "crates")
        self.git(self.main, "commit", "--quiet", "-m", "Card: Card Z")
        self.assertEqual(self.lane(self.main, "next", "--id", "w1", "--queue", "q", "--card", "Card Z").returncode, 0)
        refused = self.lane(self.main, "done", "--id", "w1", "--queue", "q")
        self.assertEqual(refused.returncode, 2)
        self.assertIn("no commit", refused.stderr)
        done = self.lane(self.main, "done", "--id", "w1", "--queue", "q", "--no-commit", "--note", "already implemented")
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(self.status(self.state(), "Card Z"), "done")

    def test_named_card_and_refresh(self):
        self.queue("--card", "Card C")
        named = self.lane(self.main, "next", "--id", "w1", "--queue", "q", "--card", "Card C")
        self.assertEqual(named.returncode, 0, named.stderr)
        self.assertIn("# Card: Card C", named.stdout)
        os.remove(os.path.join(self.main, ".card-lane", "q", "card-c", "brief.md"))
        again = self.lane(self.main, "next", "--id", "w1", "--queue", "q")
        self.assertEqual(again.returncode, 0, again.stderr)
        self.assertTrue(os.path.isfile(os.path.join(self.main, ".card-lane", "q", "card-c", "brief.md")))

    # -- the pool ----------------------------------------------------------------------------------------------------

    def test_pool_slot_needs_a_joined_member_and_records_the_lease(self):
        self.queue("--format", "test")
        unjoined = self.pool_slot(member=False)
        refused = self.lane(unjoined, "next", "--id", "w1", "--queue", "q")
        self.assertEqual(refused.returncode, 2)
        self.assertIn("no member", refused.stderr)

        joined = self.pool_slot()
        taken = self.lane(joined, "next", "--id", "w1", "--queue", "q")
        self.assertEqual(taken.returncode, 0, taken.stderr)
        claim = self.state()["cards"][0]["claim"]
        self.assertEqual(claim["slot"], "pool-1")
        self.assertEqual(claim["lease"], "L-1")
        self.assertEqual(claim["member"], "M-1")
        self.assertEqual(claim["agent"], "impl-a")
        self.assertIn("pool slot pool-1", taken.stdout)

    def test_a_read_only_pool_member_cannot_claim(self):
        self.queue("--format", "test")
        reviewing = self.pool_slot(read_only=True)
        refused = self.lane(reviewing, "next", "--id", "rev-1", "--queue", "q")
        self.assertEqual(refused.returncode, 2)
        self.assertIn("read-only", refused.stderr)

    def test_a_solo_worktree_is_allowed(self):
        self.queue("--format", "test")
        other = self.worktree("worker-two")
        taken = self.lane(other, "next", "--id", "w1", "--queue", "q")
        self.assertEqual(taken.returncode, 0, taken.stderr)
        self.assertIn("not a pool slot", taken.stderr)
        self.assertIsNone(self.state()["cards"][0]["claim"]["lease"])

    # -- the gates ---------------------------------------------------------------------------------------------------

    def card_b_scenario(self):
        return self.write("scenarios/card-b-shows-itself.cantrip",
                          "# Card B: it is cast.\n# CR 601.2.\n\n[Bot1 Deck]\nCard B\n\n[Bot2 Deck]\nIsland x20\n\n"
                          "[Actions]\nBot1.Hand.Card B.Cast\n? Bot1.Mana =\n\n[Expect]\nBot1.Battlefield.Card B = 1\n")

    def report_path(self):
        return os.path.join(self.main, ".card-lane", "q", "card-b", "report.json")

    def check_path(self):
        return os.path.join(self.main, ".card-lane", "q", "card-b", "check.json")

    def test_check_fast_skips_coverage_and_mutation_and_enforces_the_report(self):
        lane = load_lane()
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        taken = self.lane(self.main, "next", "--id", "w1", "--queue", "q", "--issue", "5")
        self.assertEqual(taken.returncode, 0, taken.stderr)
        self.card_b_scenario()
        with open(self.report_path(), encoding="utf-8") as f:
            report = json.load(f)
        report["decision"] = "done"
        self.write(os.path.relpath(self.report_path(), self.main), json.dumps(report))
        passing = mock.Mock(return_value=("checks out", ""))

        def check():
            saved = os.getcwd()
            os.chdir(self.main)
            try:
                return lane.cmd_check(argparse.Namespace(queue="q", id="w1", card=None, fast=True))
            finally:
                os.chdir(saved)

        with mock.patch.object(lane, "gate_scope", passing), \
                mock.patch.object(lane, "gate_oracle", passing), \
                mock.patch.object(lane, "gate_scenarios", passing), \
                mock.patch.object(lane, "gate_corpus", passing), \
                mock.patch.object(lane, "gate_lint", mock.Mock(return_value=("clean", {"looks": []}))), \
                mock.patch.object(lane, "gate_citations", passing), \
                mock.patch.object(lane, "gate_coverage", side_effect=AssertionError("coverage ran under --fast")), \
                mock.patch.object(lane, "gate_mutation", side_effect=AssertionError("mutation ran under --fast")):
            self.assertEqual(check(), 0)
            with open(self.check_path(), encoding="utf-8") as f:
                checked = json.load(f)
            self.assertTrue(checked["passed"])
            statuses = {gate["name"]: gate["status"] for gate in checked["gates"]}
            self.assertEqual(statuses["coverage"], "skip")
            self.assertEqual(statuses["mutation"], "skip")

            self.write("content/mtg/cards/card-b.json", json.dumps({"name": "Card B", "oracleText": "Flying"}))
            self.assertEqual(check(), 1)
            with open(self.check_path(), encoding="utf-8") as f:
                failed = json.load(f)
            self.assertFalse(failed["passed"])
            report_gate = next(gate for gate in failed["gates"] if gate["name"] == "report")
            self.assertIn("no validation entry for the printed line: Flying", report_gate["detail"])

    def test_a_test_point_needs_an_expectation_line_in_a_scenario_holding_the_card(self):
        lane = load_lane()
        self.card_b_scenario()
        scenario = "scenarios/card-b-shows-itself.cantrip"
        self.assertIsNone(lane.scenario_line_problem(self.main, scenario, 12, "Card B"))
        self.assertIn("not a ? check", lane.scenario_line_problem(self.main, scenario, 7, "Card B"))
        self.assertIn("does not hold", lane.scenario_line_problem(self.main, scenario, 12, "Card C"))

    def test_commit_plan_writes_engine_mechanics_client_then_one_card_commit(self):
        lane = load_lane()
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        taken = self.lane(self.main, "next", "--id", "w1", "--queue", "q", "--engine", "--issue", "7")
        self.assertEqual(taken.returncode, 0, taken.stderr)
        self.card_b_scenario()
        self.write("crates/cantrip-state/src/lib.rs", "pub fn entry_ready() {}\n")
        self.write("crates/cantrip-mechanics/src/shared_thing.rs", "pub fn shared_recipe() {}\n")
        self.write("dotnet/src/Thing.cs", "public sealed class ThingView {}\n")
        state = self.state()
        card = next(entry for entry in state["cards"] if entry["name"] == "Card B")
        claim = card["claim"]
        with open(os.path.join(self.main, ".card-lane", "q", "card-b", "reference.md"), encoding="utf-8") as f:
            reference = f.read()
        store = lane.Store(lane.common_dir(self.main), "q")
        made = lane.plan_commits(self.main, store, card, claim, reference, 7)
        self.assertEqual([entry["kind"] for entry in made], ["engine", "mechanics", "client", "card"])
        self.assertEqual(self.git(self.main, "log", "-4", "--format=%s").splitlines(),
                         ["Card: Card B", "Client: Thing::ThingView",
                          "Mechanics: shared_thing::shared_recipe",
                          "Engine: state::entry_ready"])
        body = self.git(self.main, "log", "-1", "--format=%B")
        self.assertTrue(body.startswith("Card: Card B"))
        self.assertIn("Closes #7", body)
        self.assertNotIn("Co-Authored-By", body)
        self.assertEqual(self.git(self.main, "status", "--porcelain"), "")

    def test_a_mechanics_commit_carries_the_existing_cards_that_adopt_the_function(self):
        lane = load_lane()
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        taken = self.lane(self.main, "next", "--id", "w1", "--queue", "q", "--issue", "7")
        self.assertEqual(taken.returncode, 0, taken.stderr)
        self.card_b_scenario()
        self.write("crates/cantrip-mechanics/src/shared_thing.rs", "pub fn shared_recipe() {}" + '\\n')
        self.write("crates/cantrip-cards/src/cards/card_c.rs", "pub static CARD: usize = 3;" + '\\n')
        state = self.state()
        card = next(entry for entry in state["cards"] if entry["name"] == "Card B")
        claim = card["claim"]
        with open(os.path.join(self.main, ".card-lane", "q", "card-b", "reference.md"), encoding="utf-8") as f:
            reference = f.read()
        store = lane.Store(lane.common_dir(self.main), "q")
        made = lane.plan_commits(self.main, store, card, claim, reference, 7)
        self.assertEqual([entry["kind"] for entry in made], ["mechanics", "card"])
        self.assertEqual(sorted(made[0]["paths"]),
                         ["crates/cantrip-cards/src/cards/card_c.rs", "crates/cantrip-mechanics/src/shared_thing.rs"])
        self.assertTrue(made[0]["message"].startswith("Mechanics: "))
        self.assertEqual(made[1]["message"], "Card: Card B")
        self.assertEqual(self.git(self.main, "status", "--porcelain"), "")

    def test_a_mechanics_commit_names_no_card_in_its_subject(self):
        lane = load_lane()
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        taken = self.lane(self.main, "next", "--id", "w1", "--queue", "q", "--issue", "7")
        self.assertEqual(taken.returncode, 0, taken.stderr)
        self.card_b_scenario()
        self.write("crates/cantrip-mechanics/src/shared_thing.rs", "pub fn shared_recipe() {}" + '\\n')
        self.write("crates/cantrip-cards/src/cards/card_c.rs", "pub static CARD: usize = 3;" + '\\n')
        state = self.state()
        card = next(entry for entry in state["cards"] if entry["name"] == "Card B")
        with open(os.path.join(self.main, ".card-lane", "q", "card-b", "reference.md"), encoding="utf-8") as f:
            reference = f.read()
        store = lane.Store(lane.common_dir(self.main), "q")
        made = lane.plan_commits(self.main, store, card, card["claim"], reference, 7)
        self.assertEqual(made[0]["message"], "Mechanics: shared_thing::shared_recipe")
        body = self.git(self.main, "log", "-1", "--skip=1", "--format=%B")
        self.assertIn("Card C now calls it instead of keeping its own copy.", body)
        self.assertNotIn("card_c", made[0]["message"])
        self.assertEqual(self.git(self.main, "status", "--porcelain"), "")

    def test_a_note_names_a_mechanics_commit_the_way_a_person_would(self):
        lane = load_lane()
        note = "Mechanics: depletion lands\n\nLand Cap, Lava Tubes and Veldt share it."
        self.assertEqual(
            lane.surface_message("Mechanics", self.main, ["crates/cantrip-mechanics/src/shared_thing.rs"], note),
            note)

    def test_a_rewritten_scenario_of_the_card_is_in_scope_and_lands_again(self):
        lane = load_lane()
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        taken = self.lane(self.main, "next", "--id", "w1", "--queue", "q", "--issue", "9")
        self.assertEqual(taken.returncode, 0, taken.stderr)
        path = self.card_b_scenario()
        state = self.state()
        card = next(entry for entry in state["cards"] if entry["name"] == "Card B")
        with open(os.path.join(self.main, ".card-lane", "q", "card-b", "reference.md"), encoding="utf-8") as f:
            reference = f.read()
        store = lane.Store(lane.common_dir(self.main), "q")
        self.assertEqual([entry["kind"] for entry in lane.plan_commits(self.main, store, card, card["claim"], reference, 9)],
                         ["card"])
        with open(path, encoding="utf-8") as f:
            text = f.read()
        self.write("scenarios/card-b-shows-itself.cantrip", text + "? Bot1.Graveyard.Card B = 0\n")
        summary, facts = lane.gate_scope(self.main, card, card["claim"], reference)
        self.assertIn("scenarios/card-b-shows-itself.cantrip", facts["scenarios"])
        made = lane.plan_commits(self.main, store, card, card["claim"], reference, 9)
        self.assertEqual([entry["kind"] for entry in made], ["card"])
        body = self.git(self.main, "log", "-1", "--format=%B")
        self.assertTrue(body.startswith("Card: Card B"))
        self.assertNotIn("Closes", body)
        self.assertEqual(self.git(self.main, "status", "--porcelain"), "")

    def test_a_scenario_that_does_not_name_the_card_is_not_its_own(self):
        lane = load_lane()
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        self.assertEqual(self.lane(self.main, "next", "--id", "w1", "--queue", "q").returncode, 0)
        self.card_b_scenario()
        self.write("scenarios/card-c-shows-itself.cantrip",
                   "# Card C: it is cast.\n\n[Bot1 Deck]\nCard C\n\n[Expect]\nBot1.Battlefield.Card C = 1\n")
        state = self.state()
        card = next(entry for entry in state["cards"] if entry["name"] == "Card B")
        with open(os.path.join(self.main, ".card-lane", "q", "card-b", "reference.md"), encoding="utf-8") as f:
            reference = f.read()
        with self.assertRaises(lane.GateFailed) as caught:
            lane.gate_scope(self.main, card, card["claim"], reference)
        self.assertIn("card-c-shows-itself.cantrip", str(caught.exception))

    def test_commit_plan_refuses_core_without_engine_scope(self):
        lane = load_lane()
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        self.assertEqual(self.lane(self.main, "next", "--id", "w1", "--queue", "q", "--issue", "8").returncode, 0)
        self.card_b_scenario()
        self.write("crates/cantrip-state/src/lib.rs", "// the core change\n")
        state = self.state()
        card = next(entry for entry in state["cards"] if entry["name"] == "Card B")
        with open(os.path.join(self.main, ".card-lane", "q", "card-b", "reference.md"), encoding="utf-8") as f:
            reference = f.read()
        store = lane.Store(lane.common_dir(self.main), "q")
        with self.assertRaises(lane.Refused) as caught:
            lane.plan_commits(self.main, store, card, card["claim"], reference, 8)
        self.assertIn("core", str(caught.exception))

    def test_a_card_design_note_must_be_deleted(self):
        lane = load_lane()
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        self.assertEqual(self.lane(self.main, "next", "--id", "w1", "--queue", "q").returncode, 0)
        self.card_b_scenario()
        note = self.write("docs/card-notes/card_b.md", "# the owner's design\n")
        state = self.state()
        card = next(entry for entry in state["cards"] if entry["name"] == "Card B")
        with open(os.path.join(self.main, ".card-lane", "q", "card-b", "reference.md"), encoding="utf-8") as f:
            reference = f.read()
        with self.assertRaises(lane.GateFailed) as caught:
            lane.gate_scope(self.main, card, card["claim"], reference)
        self.assertIn("card_b.md", str(caught.exception))
        os.remove(note)
        summary, _ = lane.gate_scope(self.main, card, card["claim"], reference)
        self.assertIn("card", summary)

    def test_deleting_a_card_file_is_still_the_card_path(self):
        lane = load_lane()
        self.assertEqual(self.queue("--card", "Some Card").returncode, 0)
        taken = self.lane(self.main, "next", "--id", "w1", "--queue", "q", "--card", "Some Card")
        self.assertEqual(taken.returncode, 0, taken.stderr)
        self.write("scenarios/some-card-shows-itself.cantrip",
                   "# Some Card: it is shown.\n\n[Bot1 Deck]\nSome Card\n\n[Expect]\nBot1.Battlefield.Some Card = 1\n")
        os.remove(os.path.join(self.main, "crates", "cantrip-cards", "src", "cards", "some_card.rs"))
        state = self.state()
        card = next(entry for entry in state["cards"] if entry["name"] == "Some Card")
        with open(os.path.join(self.main, ".card-lane", "q", "some-card", "reference.md"), encoding="utf-8") as f:
            reference = f.read()
        summary, facts = lane.gate_scope(self.main, card, card["claim"], reference)
        self.assertIn("card", summary)
        self.assertIn("some_card.rs", facts["card"])

    def test_a_mutation_survivor_passes_only_with_a_declared_equivalent(self):
        lane = load_lane()
        self.write("tools/maintenance/card-mutate.py",
                   "print('1 caught, 1 survived, 0 did not build')\n"
                   "print('SURVIVED crates/cantrip-cards/src/cards/card_b.rs:4: 4 to 5')\n"
                   "raise SystemExit(1)\n")
        survivor = {"path": "crates/cantrip-cards/src/cards/card_b.rs", "line": "4", "what": "4 to 5"}
        with self.assertRaises(lane.GateFailed):
            lane.gate_mutation(self.main, {"name": "Card B"}, {})
        summary, facts = lane.gate_mutation(
            self.main, {"name": "Card B"},
            {"equivalent_mutants": [{"mutant": "crates/cantrip-cards/src/cards/card_b.rs:4",
                                     "why": "the amount is four on both faces"}]})
        self.assertIn("declared equivalent", summary)
        self.assertEqual(facts["survivors"], [survivor])

    def test_commit_closes_the_claim_and_shares_the_branch(self):
        lane = load_lane()
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        taken = self.lane(self.main, "next", "--id", "w1", "--queue", "q", "--issue", "9")
        self.assertEqual(taken.returncode, 0, taken.stderr)
        self.card_b_scenario()
        gates = [{"name": "scope", "status": "pass", "summary": "checks out", "detail": "", "facts": {}}]
        saved = os.getcwd()
        os.chdir(self.main)
        try:
            with mock.patch.object(lane, "check_card", mock.Mock(return_value=gates)):
                code = lane.cmd_commit(argparse.Namespace(queue="q", id="w1", issue=None, note=""))
        finally:
            os.chdir(saved)
        self.assertEqual(code, 0)
        self.assertEqual(self.status(self.state(), "Card B"), "done")
        self.assertEqual(self.git(self.main, "rev-parse", "card-lane/q"), self.git(self.main, "rev-parse", "HEAD"))
        body = self.git(self.main, "log", "-1", "--format=%B")
        self.assertTrue(body.startswith("Card: Card B"))
        self.assertIn("Closes #9", body)


if __name__ == "__main__":
    unittest.main()
