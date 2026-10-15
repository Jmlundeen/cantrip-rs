# An AI player: design notes

A design sketch for a competitive AI seat, much stronger than the tactical bot, and the open questions it raises.
Nothing here is built. It records the approach the user proposed on 2026-10-14, what is known to work for games like
this, and what has to be measured before anything is committed to.

Magic is harder than any game where a strong AI player exists. It has hidden information (both hands and libraries),
chance (every draw), a state of unbounded size (objects, the stack, layered effects, pending triggers), turns of many
decisions with priority passing between them, and thousands of distinct cards. The goal is a seat that plays well
in one format first, measured against the [tactical bot](tactical-bot.md), and grows from there.

## The shape

Four parts, each usable on its own:

1. **Search** over the game, pruned hard by context, with chance and hidden information handled by expectation and
   sampling rather than plain minimax.
2. **An evaluator**, a neural network that scores a position. It replaces a hand-written heuristic both as the leaf
   value and as the guide to what the search expands first.
3. **A belief model** of what the opponent holds, built in layers and summarised as classes of threat.
4. **A training loop**, in which search and network improve each other, with a language model as an optional critic.

### 1. Search

The search is expectimax or Monte Carlo tree search, with a learned value at the leaves (AlphaZero's pattern) and
sampled opponent hands. Plain minimax has nothing for draws or unseen cards.

**Search over plans, not priority passes.** With every priority pass as a node, depth explodes long before anything
interesting happens. The moves the search considers are units a player thinks in:

- this combat (attackers, then blocks, then damage assignment);
- this main-phase line (what to cast, in what order, holding what back);
- respond or pass, at a point where something is on the stack.

Each unit is searched inside with its own narrow sub-search or a policy. That is where the per-context pruning and
heuristics live: combat has its own move generator and cut-offs, a stack fight its own.

**The search's cost is paid at play time.** A decision budget of one to five seconds buys thousands of node
expansions on a well-optimised engine. Training cost is separate (part 4).

### 2. The evaluator

A network that takes a position and returns a value (who is winning, by how much) and, later, a policy (which moves
look worth expanding).

- **Encoding: a set, not a grid.** The state is a variable number of objects, so a set encoder over permanents, hand,
  graveyard and stack is the natural fit; a transformer over objects is the obvious first choice.
- **Cards by their text, not by an id.** Each card is represented by an embedding of its Oracle text and its numbers.
  A learned per-card id would make every new card unknown; a text embedding lets the 1,000+ scripted cards and every
  future one generalise from what they say.
- **Belief features go in as inputs** (part 3), so the value it learns is already risk-weighted.

### 3. Hidden information

Sampling concrete opponent hands and searching each as if it were known makes a player act as if it could see the
hand ("strategy fusion"). The defence is to reason about **classes**, the way poker players
reason about ranges:

- class-based: "the opponent can cast a two-mana counterspell, probability 0.4";
- specific: "the second half of a known two-card combo is live".

The model is built in layers, each narrowing the last:

1. **Format and metagame priors**: which decks are played, from published decklists and 17lands data, and what is in
   the card pool at all.
2. **Revealed information**: cards seen in hand, in the graveyard, on the battlefield; cards known to be in the
   library.
3. **Inference from play**: what the opponent did and did not do. Holding up two Islands and passing without playing
   on curve is evidence of a counterspell. This layer is the most powerful and the hardest; it comes last.

The output is a small set of probabilities over threat classes, fed to the evaluator and used to weight sampled
hands in the search.

### 4. Training

**Expert iteration** is the self-play loop: search with the current network, train the network on what the search
concluded at each position, repeat. Each search result is a better label than the network's own guess, so it needs
far fewer games than training a policy from scratch by reinforcement.

The data does not all have to come from self-play:

- **Human play.** 17lands publishes Arena limited games (picks, decks, turn summaries). Imitation learning on them
  gives a competent starting policy and value before any game is simulated. Self-play then improves a player that
  already knows Magic.
- **Forced positions.** A `.cantrip` scenario puts the game in an exact position, so training can run on short
  problems: combat from this board, win this turn, this stack fight, this mulligan. Each sample is one or two orders
  of magnitude cheaper than a whole game, and they make a curriculum.
- **A language model as critic.** It can label positions, flag blunders in replays, or propose candidate lines for
  the network to imitate. It is not the authority on rules: the engine is. Language models misread Magic's rules
  often enough that their claims about legality are not used.

## What it needs from the engine

These are requirements, not yet checked against the engine.

- **Branching from a mid-game state.** Search has to try a move and come back. That includes prompts that arise
  partway through a resolution, such as the choice in Grist's −2. The engine raises such a prompt from inside an
  `await`, and a suspended async state machine cannot be cloned. Two designs fit:
  - **Replay branching.** Make the engine fully deterministic and branch by replaying from the last checkpoint with a
    different answer. The scenario harness already works this way: answers go in, in order. The cost grows with the
    distance to the checkpoint.
  - **Branch only at priority.** The search branches only where nothing is suspended; choices inside a resolution are
    left to the policy.

  Which of these the engine can support, and at what cost, is the first thing to find out.
- **A view the AI may see.** The [tactical bot](tactical-bot.md) is a client: it sees only what a player's browser
  is sent, so it cannot cheat. A searching player needs the engine, and must still not read hidden state. It searches
  over a redacted state with the hidden parts filled in from the belief model, never from the real hands. That
  boundary has to be as hard as the client's is.
- **Throughput.** Games per second, steps per second, the cost of a snapshot, and the cost of turning a state into
  the network's input.

## Throughput

The worry is that every version of this needs more simulated games than the machine can run. Whether it does depends
on which parts are used:

| Part | Simulation needed |
| --- | --- |
| Imitation from 17lands data | none |
| Forced-position training | small: short problems, many per second |
| Search at play time | a per-decision budget, not a training cost |
| Expert iteration | moderate: games played with search, far fewer than plain self-play |
| Self-play reinforcement from scratch | the largest; whether it is affordable is a question for the measured budget, not a given |

Games are independent, so simulation spreads across all cores (32 threads on the current machine), and the network
is evaluated in batches on the GPU. That split is the standard one: CPU actors, GPU inference and learning. Running
the rules themselves on a GPU does not fit Magic. Its code paths diverge on every card, and its state has no fixed
size; projects that do simulate on the GPU (Pgx for board games, Brax for physics) have fixed-size state and uniform
rules.

The language is a small factor. Modern .NET is usually within 1.2-2x of C++ for branchy, allocation-heavy code like a
rules engine. The order-of-magnitude costs are design choices: how state is snapshotted, allocations per action,
whether characteristics are recomputed on every query. Without runtime compilation of cards, C# can also ship with
NativeAOT.

## First steps

1. **Measure the engine.** Bot-against-bot games on one core, timed: games and steps per second, bytes allocated per
   step, the cost of a snapshot, and a profile of one long game. Multiplied by the core count, that is the daily
   budget, and it says how far down the throughput table the plan can go.
2. **Find out how the engine can branch.** Whether replay from a checkpoint is deterministic and fast enough, or
   whether branching should be limited to priority.
3. **A first slice.** One format with a fixed card pool (a small cube, or a handful of fixed decks); combat and
   main-phase lines only; a value network and a shallow search; measured against the tactical bot with
   `tools\bot-duel.cmd`. That proves the whole pipeline: encoding, evaluator, search, belief features. Everything
   after it adds to the same pipeline.

## Open questions

- Which format first, and where its data comes from: 17lands covers Arena limited formats; constructed needs
  decklists and a metagame source.
- How to split the network: one model with a value and a policy head, or separate specialists for mulligans,
  combat, targeting and sequencing, tied together later (the draft and tactical bots are already separate).
- How much of the hidden-information layer can be learned rather than written: classes by hand at first, learned
  later?
- How deep the plan-level abstraction goes before it hides good lines: what a "move" is in each context.
- Where the language model earns its cost: labelling, critique, candidate lines, or none of them.
- What strength the first slice must reach to justify the next: a margin over the tactical bot, measured the way
  [the tactical bot's own changes are](tactical-bot.md).
