"""Tests of the card lane (card-lane.py), each against a throwaway origin and clone:

    python -m unittest tools/maintenance/test_card_lane.py

The real tool runs against a small fake Cantrip repo whose `new-card.py` writes a content file and a skeleton. No
network, no cargo, no actual card: what is under test is the shared queue, the claim ids, the packet, the pool state
and the shared branch.
"""

import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True

HERE = os.path.dirname(os.path.abspath(__file__))
SCRIPT = os.path.join(HERE, "card-lane.py")
GIT_ENV = {"GIT_AUTHOR_NAME": "Test", "GIT_AUTHOR_EMAIL": "test@example.com",
           "GIT_COMMITTER_NAME": "Test", "GIT_COMMITTER_EMAIL": "test@example.com"}

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


if __name__ == "__main__":
    unittest.main()
