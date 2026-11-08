"""Private build output and process serialization for focused card tools."""

from contextlib import contextmanager
import os
from pathlib import Path
import subprocess
import time


def workspace_root(script=None, cwd=None):
    """Prefer the invoking Git worktree, even when the tool lives in another checkout.

    Outside Git, fall back to the repository containing the named maintenance
    script. This permits unmerged lane tools to validate a pinned card worktree.
    """
    try:
        out = subprocess.run(["git", "rev-parse", "--show-toplevel"], cwd=cwd, check=True,
                             capture_output=True, text=True, encoding="utf-8", errors="replace").stdout.strip()
        if out:
            return str(Path(out).resolve())
    except (OSError, subprocess.CalledProcessError):
        pass
    return str(Path(script or __file__).resolve().parents[2])


def build_env(root):
    """Never borrow another worktree's target or an inherited partial registry."""
    env = dict(os.environ, CANTRIP_VERIFY="1", CARGO_TARGET_DIR=str(Path(root).resolve() / "target"),
               CARGO_PROFILE_MUTATE_DEBUG_ASSERTIONS="true")
    env.pop("CANTRIP_CARDS_ONLY", None)
    return env


@contextmanager
def build_lock(root, timeout=120):
    """OS-owned lock: a killed process releases it without a stale-lock recovery step.

    Coverage, mutations, preparation and focused checks share it because Cargo's
    own lock ends before corpus execution and does not protect mutated sources.
    """
    path = Path(root).resolve() / "target" / "card-tools.lock"
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a+b") as lock:
        lock.seek(0, os.SEEK_END)
        if lock.tell() == 0:
            lock.write(b"\0")
            lock.flush()
        deadline = time.monotonic() + timeout
        waiting = False
        while True:
            try:
                if os.name == "nt":
                    import msvcrt
                    lock.seek(0)
                    msvcrt.locking(lock.fileno(), msvcrt.LK_NBLCK, 1)
                else:
                    import fcntl
                    fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                break
            except (BlockingIOError, OSError):
                if time.monotonic() >= deadline:
                    raise TimeoutError(f"another card tool still owns {path}")
                if not waiting:
                    print(f"waiting for the other card tool in {Path(root).resolve()}", flush=True)
                    waiting = True
                time.sleep(0.2)
        try:
            yield
        finally:
            if os.name == "nt":
                lock.seek(0)
                msvcrt.locking(lock.fileno(), msvcrt.LK_UNLCK, 1)
            else:
                fcntl.flock(lock.fileno(), fcntl.LOCK_UN)
