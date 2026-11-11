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
# The messages a shared-code commit owes: what changed in the rules' words, and why. The lane refuses the
# function-name template it can generate, so a test that makes an Engine: or Mechanics: commit writes one.
ENGINE_MESSAGE = ("Engine: an entry is ready once its parts are\n\n"
                  "A part that is not ready kept the whole entry from being ready, and no card could ask otherwise.")
MECHANICS_MESSAGE = ("Mechanics: a shared recipe two cards print exactly\n\n"
                     "Both cards had their own copy of it, so every change to the rule was made twice.")
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


def headless():
    """A stdin with no terminal, as a worker agent's shell has: `commit` refuses rather than prompts."""
    fake = mock.Mock()
    fake.isatty.return_value = False
    return fake


def commit_args(no_issue=False, fix=False, no_review=False, message=None):
    """What the `commit` command line hands its handler, when the test calls it rather than the subprocess."""
    return argparse.Namespace(queue="q", id="w1", issue=None, note="", message=message or [],
                              no_review=no_review, review_note="", no_issue=no_issue, review_lane="review",
                              fix=fix)


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

    def write_queue(self, state):
        path = os.path.join(self.main, ".git", "card-lane", "q", "queue.json")
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            json.dump(state, f)
        return 0

    def commit_with_gates(self, lane, fix=False):
        """`commit` with every gate passing, so the review gate is what is under test, not a card's code."""
        gates = [{"name": "scope", "status": "pass", "summary": "checks out", "detail": "", "facts": {}}]
        saved = os.getcwd()
        os.chdir(self.main)
        try:
            with mock.patch.object(lane, "check_card", mock.Mock(return_value=gates)):
                return lane.cmd_commit(commit_args(fix=fix))
        finally:
            os.chdir(saved)

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
        made = lane.plan_commits(self.main, store, card, claim, reference, 7, review=False,
                                 messages=lane.parsed_messages([
                                     "engine=" + ENGINE_MESSAGE, "mechanics=" + MECHANICS_MESSAGE]))
        self.assertEqual([entry["kind"] for entry in made], ["engine", "mechanics", "client", "card"])
        self.assertEqual(self.git(self.main, "log", "-4", "--format=%s").splitlines(),
                         ["Card: Card B", "Client: Thing::ThingView",
                          "Mechanics: a shared recipe two cards print exactly",
                          "Engine: an entry is ready once its parts are"])
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
        with mock.patch.object(lane.sys, "stdin", headless()):
            made = lane.plan_commits(self.main, store, card, claim, reference, 7, review=False,
                                     messages=lane.parsed_messages(["mechanics=" + MECHANICS_MESSAGE]))
        self.assertEqual([entry["kind"] for entry in made], ["mechanics", "card"])
        self.assertEqual(sorted(made[0]["paths"]),
                         ["crates/cantrip-cards/src/cards/card_c.rs", "crates/cantrip-mechanics/src/shared_thing.rs"])
        self.assertTrue(made[0]["message"].startswith("Mechanics: a shared recipe"))
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
        with mock.patch.object(lane.sys, "stdin", headless()):
            made = lane.plan_commits(self.main, store, card, card["claim"], reference, 7, review=False,
                                     messages=lane.parsed_messages(["mechanics=" + MECHANICS_MESSAGE]))
        self.assertTrue(made[0]["message"].startswith("Mechanics: a shared recipe"))
        self.assertIn("every change to the rule was made twice", made[0]["body"])
        self.assertNotIn("card_c", made[0]["message"])
        self.assertNotIn("Card C", made[0]["message"])
        self.assertEqual(self.git(self.main, "status", "--porcelain"), "")

    def test_a_note_names_a_mechanics_commit_the_way_a_person_would(self):
        lane = load_lane()
        note = "Mechanics: depletion lands\n\nLand Cap, Lava Tubes and Veldt share it."
        self.assertEqual(
            lane.surface_message("Mechanics", self.main, ["crates/cantrip-mechanics/src/shared_thing.rs"], note),
            note)

    def test_a_shared_code_message_is_the_template_until_it_is_written_by_hand(self):
        lane = load_lane()
        # what commit generates from the changed functions is a template, not a message
        self.assertIn("names a function", lane.message_problem("mechanics", "Mechanics: shared_thing::shared_recipe"))
        self.assertIn("names a function", lane.message_problem("engine", "Engine: state::entry_ready"))
        self.assertIn("names a function", lane.message_problem("engine", "Engine: entry_ready"))
        self.assertIn("shared mechanics", lane.message_problem("engine", "Engine: shared mechanics"))
        self.assertTrue(lane.message_problem("engine", "Engine: "))
        self.assertIn("must start", lane.message_problem("engine", "Handler: an entry is ready") or "")
        # a message that says what changed passes, and a Client: message is never rewritten
        self.assertIsNone(lane.message_problem("engine", ENGINE_MESSAGE))
        self.assertIsNone(lane.message_problem("mechanics", MECHANICS_MESSAGE))
        self.assertIsNone(lane.message_problem("client", "Client: Thing::ThingView"))

    def test_a_message_takes_a_subject_and_a_body_from_a_file(self):
        lane = load_lane()
        body = os.path.join(self.tmp, "body.txt")
        with open(body, "w", encoding="utf-8", newline="\n") as f:
            f.write("It was needed before the card could ask the question.\n")
        given = lane.parsed_messages(["mechanics=Mechanics: the shared recipe", "mechanics2=@" + body])
        self.assertEqual(given["mechanics"],
                         "Mechanics: the shared recipe\n\nIt was needed before the card could ask the question.")
        self.assertIn("names a function", lane.message_problem("mechanics", "Mechanics: shared_thing::go"))
        with self.assertRaises(lane.Refused):
            lane.parsed_messages(["mechanics2=orphan body"])
        with self.assertRaises(lane.Refused):
            lane.parsed_messages(["cards=Card: nope"])

    def test_commit_refuses_a_shared_code_message_nobody_rewrote(self):
        lane = load_lane()
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        self.assertEqual(self.lane(self.main, "next", "--id", "w1", "--queue", "q",
                                   "--no-issue").returncode, 0)
        self.card_b_scenario()
        self.write("crates/cantrip-mechanics/src/shared_thing.rs", "pub fn shared_recipe() {}\n")
        saved = os.getcwd()
        os.chdir(self.main)
        try:
            gates = [{"name": "scope", "status": "pass", "summary": "ok", "detail": "", "facts": {}}]
            headless = mock.Mock()
            headless.isatty.return_value = False
            with mock.patch.object(lane, "check_card", mock.Mock(return_value=gates)), \
                    mock.patch.object(lane.sys, "stdin", headless):
                with self.assertRaises(lane.Refused) as refused:
                    lane.cmd_commit(commit_args())
        finally:
            os.chdir(saved)
        self.assertIn("rewritten", str(refused.exception))
        self.assertIn("--message mechanics=", str(refused.exception))
        self.assertEqual(self.status(self.state(), "Card B"), "working")

    def test_a_written_message_survives_into_the_commit(self):
        lane = load_lane()
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        self.assertEqual(self.lane(self.main, "next", "--id", "w1", "--queue", "q",
                                   "--no-issue").returncode, 0)
        self.card_b_scenario()
        self.write("crates/cantrip-mechanics/src/shared_thing.rs", "pub fn shared_recipe() {}\n")
        body = os.path.join(self.tmp, "mechanics.txt")
        with open(body, "w", encoding="utf-8", newline="\n") as f:
            f.write(MECHANICS_MESSAGE + "\n")
        saved = os.getcwd()
        os.chdir(self.main)
        try:
            gates = [{"name": "scope", "status": "pass", "summary": "ok", "detail": "", "facts": {}}]
            with mock.patch.object(lane, "check_card", mock.Mock(return_value=gates)):
                code = lane.cmd_commit(commit_args(message=[f"mechanics=@{body}"]))
        finally:
            os.chdir(saved)
        self.assertEqual(code, 0)
        text = self.git(self.main, "log", "-1", "--skip=1", "--format=%B")
        self.assertTrue(text.startswith(MECHANICS_MESSAGE.splitlines()[0]))
        self.assertIn("every change to the rule was made twice", text)

    def test_no_review_lands_the_card_at_once(self):
        lane = load_lane()
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        self.assertEqual(self.lane(self.main, "next", "--id", "w1", "--queue", "q",
                                   "--issue", "9").returncode, 0)
        self.card_b_scenario()
        saved = os.getcwd()
        os.chdir(self.main)
        try:
            gates = [{"name": "scope", "status": "pass", "summary": "ok", "detail": "", "facts": {}}]
            with mock.patch.object(lane, "check_card", mock.Mock(return_value=gates)):
                code = lane.cmd_commit(commit_args(no_review=True))
        finally:
            os.chdir(saved)
        self.assertEqual(code, 0)
        self.assertEqual(self.status(self.state(), "Card B"), "done")
        self.assertEqual(self.git(self.main, "rev-parse", "card-lane/q"), self.git(self.main, "rev-parse", "HEAD"))
        self.assertIn("Closes #9", self.git(self.main, "log", "-1", "--format=%B"))

    def test_a_review_note_is_kept_and_a_block_needs_one(self):
        lane = load_lane()
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        self.assertEqual(self.lane(self.main, "next", "--id", "w1", "--queue", "q",
                                   "--no-issue").returncode, 0)
        self.card_b_scenario()
        self.commit_with_gates(lane)
        refused = self.lane(self.main, "review", "--queue", "q", "--block", "Card B", "--findings", "1")
        self.assertEqual(refused.returncode, 2)
        self.assertIn("--note", refused.stderr)
        blocked = self.lane(self.main, "review", "--queue", "q", "--block", "Card B", "--findings", "1",
                            "--note", "the trigger must fire for the opponent too, so make it 'each upkeep'")
        self.assertEqual(blocked.returncode, 0, blocked.stderr)
        listing = self.lane(self.main, "review", "--queue", "q", "--list")
        self.assertEqual(listing.returncode, 0, listing.stderr)
        self.assertIn("BLOCKED", listing.stdout)
        self.assertIn("each upkeep", listing.stdout)

    def test_review_list_shows_the_waiting_cards_and_the_cap(self):
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        listing = self.lane(self.main, "review", "--queue", "q", "--list")
        self.assertEqual(listing.returncode, 0, listing.stderr)
        self.assertIn("0/20", listing.stdout)
        self.assertIn("nothing is waiting", listing.stdout)

    def test_a_card_name_with_a_comma_survives_the_pending_list_and_next(self):
        lane = load_lane()
        name = "Cmdr. Troi, Outgoing Empath"
        self.assertEqual(self.queue("--card", name).returncode, 0)
        status = self.lane(self.main, "status", "--queue", "q")
        self.assertEqual(status.returncode, 0, status.stderr)
        self.assertIn("  " + name, status.stdout.replace(chr(13) + chr(10), chr(10)))
        taken = self.lane(self.main, "next", "--id", "w1", "--queue", "q", "--card", name, "--issue", "7")
        self.assertEqual(taken.returncode, 0, taken.stderr)
        self.assertIn("# Card: " + name, taken.stdout)

    def test_next_names_the_whole_card_when_given_a_cut_down_name(self):
        lane = load_lane()
        name = "Cmdr. Troi, Outgoing Empath"
        self.assertEqual(self.queue("--card", name).returncode, 0)
        taken = self.lane(self.main, "next", "--id", "w1", "--queue", "q", "--card", "Cmdr. Troi")
        self.assertNotEqual(taken.returncode, 0)
        self.assertIn(name, taken.stdout + taken.stderr)

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

    def test_commit_plan_takes_core_without_engine_scope(self):
        """The core is always in scope: a card opened without --engine still plans its core change as an Engine: commit."""
        lane = load_lane()
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        self.assertEqual(self.lane(self.main, "next", "--id", "w1", "--queue", "q", "--issue", "8").returncode, 0)
        self.card_b_scenario()
        self.write("crates/cantrip-state/src/lib.rs", "// the core change" + chr(10))
        state = self.state()
        card = next(entry for entry in state["cards"] if entry["name"] == "Card B")
        with open(os.path.join(self.main, ".card-lane", "q", "card-b", "reference.md"), encoding="utf-8") as f:
            reference = f.read()
        store = lane.Store(lane.common_dir(self.main), "q")
        with mock.patch.object(lane.sys, "stdin", headless()):
            made = lane.plan_commits(self.main, store, card, card["claim"], reference, 8, review=False,
                                     messages=lane.parsed_messages(["engine=" + ENGINE_MESSAGE]))
        kinds = [entry["kind"] for entry in made]
        self.assertIn("engine", kinds)
        self.assertLess(kinds.index("engine"), kinds.index("card"))

    def test_a_claim_records_the_core_as_open(self):
        lane = load_lane()
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        self.assertEqual(self.lane(self.main, "next", "--id", "w1", "--queue", "q").returncode, 0)
        card = next(entry for entry in self.state()["cards"] if entry["name"] == "Card B")
        self.assertTrue(card["claim"]["engine"])

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

    def test_commit_sends_the_card_to_the_review_branch_and_not_the_queue(self):
        lane = load_lane()
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        taken = self.lane(self.main, "next", "--id", "w1", "--queue", "q", "--issue", "9")
        self.assertEqual(taken.returncode, 0, taken.stderr)
        self.card_b_scenario()
        self.commit_with_gates(lane)
        self.assertEqual(self.status(self.state(), "Card B"), "review")
        head = self.git(self.main, "rev-parse", "HEAD")
        self.assertEqual(self.git(self.main, "rev-parse", "card-lane/review"), head)
        self.assertNotEqual(self.git(self.main, "rev-parse", "card-lane/q"), head)
        body = self.git(self.main, "log", "-1", "--format=%B")
        self.assertTrue(body.startswith("Card: Card B"))
        self.assertNotIn("Closes #", body)  # only the review's pass closes the issue
        card = next(entry for entry in self.state()["cards"] if entry["name"] == "Card B")
        self.assertEqual(card["review"]["author"], "w1")
        self.assertEqual(card["review"]["commits"], [head])

    def test_review_pass_moves_the_card_onto_the_queue_and_writes_its_closes(self):
        lane = load_lane()
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        self.assertEqual(self.lane(self.main, "next", "--id", "w1", "--queue", "q",
                                   "--issue", "9").returncode, 0)
        self.card_b_scenario()
        self.commit_with_gates(lane)
        passed = self.lane(self.main, "review", "--queue", "q", "--pass", "Card B")
        self.assertEqual(passed.returncode, 0, passed.stderr)
        self.assertEqual(self.status(self.state(), "Card B"), "done")
        self.assertEqual(self.git(self.main, "rev-parse", "card-lane/q"),
                         self.git(self.main, "rev-parse", "HEAD"))
        body = self.git(self.main, "log", "-1", "--format=%B")
        self.assertTrue(body.startswith("Card: Card B"))
        self.assertTrue(body.rstrip().endswith("Closes #9"), body)
        self.assertEqual(self.git(self.main, "status", "--porcelain"), "")

    def test_a_card_cleared_from_the_gate_still_holds_its_place_until_it_passes(self):
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        state = self.state()
        state["cards"][0].update(status="review", review={"author": "w9", "commits": [], "state": "blocked"})
        self.write_queue(state)
        status = self.lane(self.main, "status", "--queue", "q")
        self.assertEqual(status.returncode, 0, status.stderr)
        self.assertIn("1/20", status.stdout)
        self.assertIn("blocked, back for its fixes", status.stdout)

    def test_a_blocked_card_goes_back_to_its_author_and_its_fix_returns_to_the_gate(self):
        lane = load_lane()
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        self.assertEqual(self.lane(self.main, "next", "--id", "w1", "--queue", "q",
                                   "--issue", "9").returncode, 0)
        self.card_b_scenario()
        self.commit_with_gates(lane)
        blocked = self.lane(self.main, "review", "--queue", "q", "--block", "Card B", "--findings", "1",
                            "--note", "the trigger fires on the wrong player's upkeep; it must read 'each upkeep', "
                                      "so make it fire for the opponent as well")
        self.assertEqual(blocked.returncode, 0, blocked.stderr)
        card = next(entry for entry in self.state()["cards"] if entry["name"] == "Card B")
        self.assertEqual(card["status"], "review")
        self.assertEqual(card["review"]["state"], "blocked")
        self.assertEqual(card["review"]["findings"], 1)
        self.assertTrue(card["review"]["next_message"].startswith("the trigger fires"))
        # the author takes it back as a claim, fixes it, and sends it back to the gate
        self.assertEqual(self.lane(self.main, "next", "--id", "w1", "--queue", "q").returncode, 0)
        self.assertEqual(self.status(self.state(), "Card B"), "working")
        self.write("crates/cantrip-cards/src/cards/card_b.rs", "// the fix\n")
        self.commit_with_gates(lane, fix=True)
        card = next(entry for entry in self.state()["cards"] if entry["name"] == "Card B")
        self.assertEqual(card["status"], "review")
        self.assertEqual(card["review"]["state"], "waiting")
        self.assertEqual(card["review"]["bumps"], 1)
        # and a fix that passed again does not close the issue a second time
        self.lane(self.main, "review", "--queue", "q", "--pass", "Card B")
        self.assertEqual(self.status(self.state(), "Card B"), "done")
        self.assertEqual(self.git(self.main, "log", "-1", "--format=%B").count("Closes #9"), 1)

    def test_the_review_gate_caps_the_queue_at_twenty_cards(self):
        names = [f"Card {n}" for n in range(20)]
        self.assertEqual(self.queue(*names, "--card", "Card Z").returncode, 0)
        state = self.state()
        for index, name in enumerate(names):
            card = next(entry for entry in state["cards"] if entry["name"] == name)
            card.update(status="review", review={"author": f"w{index}", "commits": [], "state": "waiting"})
        self.write_queue(state)
        refused = self.lane(self.main, "next", "--id", "w1", "--queue", "q")
        self.assertEqual(refused.returncode, 2)
        self.assertIn("20", refused.stderr)
        self.assertIn("review gate", refused.stderr)
        self.assertEqual(self.status(self.state(), "Card Z"), "pending")
        # a card cleared from the gate makes room for the next one
        state = self.state()
        done = next(entry for entry in state["cards"] if entry["name"] == names[0])
        done["status"] = "done"
        self.write_queue(state)
        taken = self.lane(self.main, "next", "--id", "w1", "--queue", "q")
        self.assertEqual(taken.returncode, 0, taken.stderr)
        self.assertEqual(self.status(self.state(), "Card Z"), "working")


    def test_a_scratch_directory_does_not_block_a_claim(self):
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        self.write(".w1-scratch/notes.txt", "thinking about the card\n")
        taken = self.lane(self.main, "next", "--id", "w1", "--queue", "q", "--no-issue")
        self.assertEqual(taken.returncode, 0, taken.stderr)
        self.assertEqual(self.status(self.state(), "Card B"), "working")

    def test_a_no_issue_commit_closes_the_card_without_a_closes_line(self):
        lane = load_lane()
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        taken = self.lane(self.main, "next", "--id", "w1", "--queue", "q", "--no-issue")
        self.assertEqual(taken.returncode, 0, taken.stderr)
        self.card_b_scenario()
        gates = [{"name": "scope", "status": "pass", "summary": "checks out", "detail": "", "facts": {}}]
        saved = os.getcwd()
        os.chdir(self.main)
        try:
            with mock.patch.object(lane, "check_card", mock.Mock(return_value=gates)):
                code = lane.cmd_commit(commit_args(no_issue=True))
        finally:
            os.chdir(saved)
        self.assertEqual(code, 0)
        self.assertEqual(self.status(self.state(), "Card B"), "review")
        body = self.git(self.main, "log", "-1", "--format=%B")
        self.assertTrue(body.startswith("Card: Card B"))
        self.assertNotIn("Closes #", body)
        passed = self.lane(self.main, "review", "--queue", "q", "--pass", "Card B")
        self.assertEqual(passed.returncode, 0, passed.stderr)
        self.assertEqual(self.status(self.state(), "Card B"), "done")
        self.assertNotIn("Closes #", self.git(self.main, "log", "-1", "--format=%B"))

    def test_commit_refuses_without_an_issue_unless_no_issue_is_passed(self):
        lane = load_lane()
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        self.assertEqual(self.lane(self.main, "next", "--id", "w1", "--queue", "q").returncode, 0)
        saved = os.getcwd()
        os.chdir(self.main)
        try:
            with self.assertRaises(lane.Refused):
                lane.cmd_commit(argparse.Namespace(queue="q", id="w1", issue=None, note="", no_issue=False))
        finally:
            os.chdir(saved)
        self.assertEqual(self.status(self.state(), "Card B"), "working")

    def test_done_gated_keeps_the_claim_open_when_a_gate_fails(self):
        lane = load_lane()
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        self.assertEqual(self.lane(self.main, "next", "--id", "w1", "--queue", "q", "--no-issue").returncode, 0)
        gates = [{"name": "scenarios", "status": "fail", "summary": "no scenario holds it", "detail": "", "facts": {}}]
        saved = os.getcwd()
        os.chdir(self.main)
        try:
            with mock.patch.object(lane, "check_card", mock.Mock(return_value=gates)):
                with self.assertRaises(lane.Refused):
                    lane.cmd_done(argparse.Namespace(queue="q", id="w1", commit=None, no_commit=False,
                                                     note="", gated=True))
        finally:
            os.chdir(saved)
        self.assertEqual(self.status(self.state(), "Card B"), "working")

    def test_a_second_check_reuses_the_corpus_run_until_a_file_changes(self):
        lane = load_lane()
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        taken = self.lane(self.main, "next", "--id", "w1", "--queue", "q", "--no-issue")
        self.assertEqual(taken.returncode, 0, taken.stderr)
        store = lane.Store(lane.common_dir(self.main), "q")
        card = next(entry for entry in self.state()["cards"] if entry["name"] == "Card B")
        reference = "=== Test points\n"
        with mock.patch.object(lane, "gate_corpus", mock.Mock(return_value=("1 pass, 0 fail", ""))) as corpus:
            first = lane.check_card(self.main, store, card, card["claim"], reference, fast=True)
            lane.save_check(self.main, store, card, first, True)
            second = lane.check_card(self.main, store, card, card["claim"], reference, fast=True)
        self.assertEqual(corpus.call_count, 1)
        summary = next(gate["summary"] for gate in second if gate["name"] == "corpus")
        self.assertIn("reused", summary)
        self.write("scenarios/card-b-more.cantrip", "# a new scenario\n\n[Bot1 Deck]\nCard B x20\n")
        with mock.patch.object(lane, "gate_corpus", mock.Mock(return_value=("1 pass, 0 fail", ""))) as corpus:
            lane.check_card(self.main, store, card, card["claim"], reference, fast=True)
        self.assertEqual(corpus.call_count, 1)


    def test_next_claims_the_cards_issue_and_the_brief_closes_it(self):
        tool = self.write("fake-card-issues.py", '''print("#42 is yours (Closes #42)")
''', root=self.tmp)
        os.environ["CANTRIP_ISSUES_TOOL"] = tool
        self.addCleanup(os.environ.pop, "CANTRIP_ISSUES_TOOL", None)
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        taken = self.lane(self.main, "next", "--id", "w1", "--queue", "q")
        self.assertEqual(taken.returncode, 0, taken.stderr)
        claim = next(entry for entry in self.state()["cards"] if entry["name"] == "Card B")["claim"]
        self.assertEqual(claim["issue"], 42)
        self.assertIn("Closes #42", taken.stdout)

    def test_next_survives_an_issue_it_cannot_claim(self):
        tool = self.write("fake-card-issues.py", '''import sys
print("gh: not signed in", file=sys.stderr)
raise SystemExit(1)
''', root=self.tmp)
        os.environ["CANTRIP_ISSUES_TOOL"] = tool
        self.addCleanup(os.environ.pop, "CANTRIP_ISSUES_TOOL", None)
        self.assertEqual(self.queue("--card", "Card B").returncode, 0)
        taken = self.lane(self.main, "next", "--id", "w1", "--queue", "q")
        self.assertEqual(taken.returncode, 0, taken.stderr)
        claim = next(entry for entry in self.state()["cards"] if entry["name"] == "Card B")["claim"]
        self.assertIsNone(claim.get("issue"))
        self.assertIn("has no issue to close", taken.stderr)


if __name__ == "__main__":
    unittest.main()
