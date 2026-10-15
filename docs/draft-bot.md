# The draft bot

`CubeDrafter` and `DeckBuilder` (`dotnet/src/Cantrip.Client/Drafting`) are what a seat nobody is sitting in drafts a booster
pod and builds its deck with. The bot it replaced took the best card by the cube's pick order whatever it already had,
and played the two colours it had most cards of with seventeen basic lands. This one drafts a deck: it takes the card
that most improves what its pool is becoming, reads what the packs passed to it say is open, and builds the colours,
splash and lands that cast the most of what it took.

The same two things were fixed first as for the seat that plays ([tactical-bot.md](tactical-bot.md)):

- **It runs entirely locally.** A pick is arithmetic over the pack and the pool: under a millisecond, no model
  to download, nothing to call.
- **It is a client.** It lives in `Cantrip.Client`, sees what a player's browser is sent (`DraftMessage`,
  `DeckBuildingMessage`) and nothing else, and so runs in a console process, in the Discord bot and in a player's tab
  alike. It cannot see another seat's pool or a pack it has not been passed.

`DraftPlay` is the seam: it hands a booster pack, a grid or a Winston pile to the drafter and the pool to the builder.

## It knows no card by name

Everything it knows about a card it reads off the card as shown (`DraftCard`): which colours each symbol of its cost
can be paid with (hybrid either, Phyrexian none, either half of a split card), what mana it makes or finds (a dual
land's types, an "Add", what a fetch land searches for), and what it is for - removal, a sweeper, a counterspell, card
advantage, evasion, a cheap creature that hits - from the common shapes of rules text.

How *good* a card is comes from two places (`CardWorth`), on one scale: about 3 for a cube's middling card, 5 for one
of its best twenty, 7 for the Power Nine.

- **The cube's pick order**, where the server sends one (`DraftMessage.PileRanks`). A rank is read onto the scale the
  order was fitted on - a Bradley-Terry strength - which across a cube follows the log-odds of the rank: flat in the
  middle, steep at both ends. The drafter remembers the rank of every card it has been shown, which is how it knows
  what its pool is worth when the deck is built.
- **The card itself**, for a card no order names - which the server marks by giving them all one rank past the end,
  and the drafter spots when two different cards share a rank. Its worth is then a weighted sum of what it costs, is
  and says. The weights are a ridge regression of the pick orders on those features, held towards a hand-written
  prior (`cantrip-botlab draft fit`). It is rough - held out, it explains about a sixth of the spread in worth, since
  a cube has no bad cards in it for a fit to learn from - but it tells cheap removal from an expensive vanilla
  creature, and it is what a draft with no pick order at all is drafted by (`--no-order` in the lab).

A pick order is what people *took*, which is not quite how good a card is in a deck: people take a colourless card
early because every deck plays it, and a gold card late because few can. The drafter works out for itself which decks
a card could go in, so it takes that back out (`CardWorth.InDeck`).

## Fifteen decks at once

The drafter keeps no colours and has no rule for when to commit. It carries fifteen hypotheses - the deck is each
colour alone, or each pair - and for each works out what that deck would be worth at the end of the draft
(`CubeDrafter.Project`):

- the best spells the pool has that those colours cast, each worth its power *as far as the lands can cast it*
  (below), with a third colour's best card or two splashed at what a splash is worth;
- for the picks still to come, the cards those colours can expect to be passed - a table of what a pack usually
  still holds with so many cards left, measured in the lab (`draft calibrate`), shifted by the signals;
- the best twenty-three of those together, since a deck holds twenty-three spells;
- what the pool's lands add (a land making both colours, a utility land), less what the deck's shape costs it:
  creatures it is short of, early plays it is short of, more five-drops than it can hold.

A pool is worth the best of its fifteen, softly - the log of the sum of their exponentials - and the card taken is
the one the pool is worth most with. How soft is one number (`DraftTuning.Sureness`), and it hardens by itself as the
draft runs out of picks to change course with.

That one sum gives what drafters describe as rules, without any of them being written down:

- **Early picks go by power.** Every hypothesis expects to fill its slots with good cards, so only a better card adds
  anything, and by how much better.
- **Flexibility is worth something while nothing is settled.** A colourless card raises all fifteen hypotheses, a
  mono-coloured one five, a gold one a single pair: of three cards as good, the colourless one is taken first and the
  gold one last.
- **Commitment is gradual.** Two strong cards of a colour are a lean; ten are a commitment. An off-colour bomb can
  still move the pool on pick five, and almost nothing can on pick thirty.
- **The twenty-third playable matters and the twenty-eighth does not.** Once a deck's slots are full of better cards a
  card adds nothing - and then the land that fixes its mana is worth more than one more spell.
- **Curve and creature count.** A fifth six-drop costs the deck its shape; a second two-drop repairs it.
- **A cheat needs something to cheat.** A card that puts a creature into play from a hand, a graveyard or the
  library, or an artifact from the library, for less than it costs (`DraftCard.Cheats`: Sneak Attack, Reanimate,
  Natural Order, Tinker) is worth its power only beside a creature or artifact costing six or more in the deck's
  colours (`DraftCard.Payload`). With none it keeps 40% (`DeckTuning.Unsupported`); one makes it whole
  (`Payloads`). Early in the draft it is counted on finding two (`DraftTuning.PayloadHope`, shrinking as the picks
  run out), so a first-pick Sneak Attack is still taken for its power; on pick thirty with nothing to cheat it is
  not. The same reading runs both ways: a pool with a Reanimate in it is worth more for the Griselbrand it is shown.

## Grid and Winston drafts

The same sum chooses a grid's row or column (`CubeDrafter.ChooseLine`): every line's cards added to the pool at once,
weighed against the grids still to come - three cards from the best line of nine when the seat picks first, two from
what is left when it picks second, each worth what a booster pack of nine, six or three usually still holds. A Winston
pile is taken (`CubeDrafter.TakePile`) when the pool is worth more with it than with one card from the stack - one
more card to come, as good as a pack of one - and where neither adds anything, as the plain bot did, when it has two
cards or more. The server shows where the cards of the pile looked at stand in the pick order, as it does a grid's;
the pool's cards are worth what the drafter learnt of them as they passed.

Against the plain way (the grid's line worth most by the pick order; any Winston pile of two or more), both built by
the new builder, 300 tables of two and 2,400 games a run, the referee pilot (`bot-draft.cmd --format grid --a new --b
new-deck --pods 300 --games 8`): grid 53.2%, 50.2%, 51.5% and 53.6% in four runs over both cubes, about 52% +/- 1.4%
together; Winston 51.8%, 51.5%, 53.6% and 52.8%, about 52% +/- 1.4% (each run +/- 2.8%). Weighing a pass as the next
pile rather than a card from the stack (`WinstonNext`) showed nothing either way (49.2% +/- 2.9%). Those Winston
numbers were taken before the server sent a Winston pile's ranks; with them, 52.4% +/- 2.7% (LSVCube) and 52.6% +/-
2.7% (PowerLSV). It does not hate-draft: costing a first pick from a
grid what it leaves the other player (`DraftTuning.Deny`, off) came out 50.6% +/- 2.9% at 1 in both cubes and 49.0%
+/- 3.0% at 0.3, head to head with the play bot's pilot of 9029b602.

## What it remembers, and what is open

`DraftMemory` keeps every pack the seat was shown, what it took from each and so what it passed, and - when a pack
comes round the table again - which cards did not come back (`TakenByOthers`). From those it estimates, colour by
colour, how much better or worse than usual the cards reaching this seat are going to be.

A pack is read as how much each colour has in it worth playing - every card's worth above a middling one's, a gold
card's split between its colours - set against the average of the five colours (`DraftMemory.Reading`). A pack with
two strong blue cards and nothing red reads blue up and red down, whatever pick it is. Three things are kept:

- **What is passed to it** (`Flow`): the readings of the packs it is shown after the first pick, a round at a time,
  held towards nothing while few have been seen.
- **What it passes** (`Passed`): the same readings of what it sends on. The players it passes to are the ones passing
  to it when the direction turns.
- **What does not wheel** (`Contested`): between two sightings of a pack every other player took one card from it.
  Had they gone by worth alone the best of it would be missing; a lesser card missing instead was taken for its
  colour.

What each of those is worth as a forecast is measured, not assumed. `bot-draft.cmd calibrate` drafts hundreds of pods
and fits what a colour went on to deliver - the best card of it in the packs that came, above the usual - to what had
been read of it, and the slopes are the drafter's weights (`DraftTuning`):

| reading | forecasts | weight | |
|---|---|---|---|
| a pack round's flow | the round after next, when packs come the same way | `Carry` 0.55 | the one that matters: a whole pack of looks at what the same players upstream did not take |
| this round's flow so far | the rest of this round | `Signals` 0.12 | 0.06 in pack one, 0.18 by pack three; little, because by the time a pack has said anything what is left of it is its last picks |
| cards taken over better ones | that colour, everywhere | `Crowd` 0.2 | small |
| what was passed downstream | the next round, when the direction turns | `Cut` 0 | nothing, among these drafters: what a seat passes does not show in what comes back to it |

The first reading tried - the single best card of each colour against what a pack usually holds - forecast a third as
well as this one, which is why the lab measures a reading before the drafter is given it. The estimate is added to
what each hypothesis expects of its coming picks, so an open colour makes the decks that play it worth more, and the
cards of that colour with them.

## The deck

`DeckBuilder` does not choose colours and then fill them in. It builds every deck the pool could be - each colour
alone, each pair, and each of those with each third colour splashed - and keeps the best, where best is the spells'
power as far as the lands cast them. One sum settles colours, splash and lands together, so a splash has to pay for
the sources it takes from the main colours.

- **Size.** Always the minimum (`DeckBuildingMessage.Minimum`), never a card more: a pool too thin to fill it is made
  up with basic lands.
- **Lands.** Frank Karsten's count for the curve - in forty cards, two thirds of 19.59 + 1.9 x the average mana
  value - less a little for each cheap card that makes mana or draws, and one for each Mox or other free mana;
  between fifteen and eighteen (`ManaBase.LandsFor`).
- **Which lands.** The pool's lands that make two of the deck's colours; a utility land or two where what it does is
  worth more than the spells lose by the basic it replaces; basic lands split every way there is, keeping the split
  that casts the most.
- **Casting.** How much of a card is left on so many sources is counted, not looked up (`ManaBase.Castable`): the
  chance of having drawn as many sources as the cost has symbols of the colour by the turn it could first be cast,
  and three turns later - the sum Karsten's tables of sources are thresholds of. A mana creature or rock counts half
  a source of what it makes, a fetch land a whole one of each type it finds.
- **Splash.** Cards with a single symbol of the third colour, the best three at most.
- **Shape.** Spells are traded one for another while a trade improves the deck: thirteen creatures, six plays at two
  mana or less and no more than five at five or more, shifted for a deck that leans early or late.

Between the games of a match the deck already built is played again as it stands.

## What it was thinking

A bot seat can say why it picked what it picked and built what it built (`DraftPlay.Narrator`, a `TextWriter`; off by
default). It is written after each choice is sent, from the numbers the choice was made by, and is never a part of
one: a narrated seat drafts and builds exactly as a silent one (`DraftPlayTests`, `CubeDrafterTests` compare every
score, line, pile and deck; 480 lab decks and 400 grid and Winston decks came out byte for byte the same as before
it was written). A pick told costs about a millisecond more than one not told. A writer that fails is let go and the
seat drafts on.

**Switching it on.** It goes wherever the seat's games are narrated (`TacticalPlay.Narrator`): an `AutoSeat` hands its
tactical player's narrator to its drafter, so one file per seat holds the draft, then the deck, then its games.

- `cantrip-bot --tactical` writes to a file in the temp folder and says where (`--narrate PATH`, `--narrate none`).
- The Discord bot's seats: `--bot-narration DIR` (or `CANTRIP_BOT_NARRATION`), one file a seat in DIR.
- The web page's own bot seats (`?bots=`, `?bot-tickets=`): the browser's console under `?narrate`, each line
  prefixed with the seat's name.
- The lab: `bot-draft.cmd watch --seat 3 [--seed N] [--cube PowerLSV]` drafts a pod of bots and tells one seat;
  `--format grid` or `--format winston` a table of two. `bot-draft.cmd watch --event ID --seat 3 [--data DIR]`
  replays a booster draft kept in a data directory (`CANTRIP_DATA`, else `.cantrip-data`) from its log: each pack the
  seat was shown, put to the drafter as it is built now, and the deck it would build. For a bot seat drafted by the
  same build that is what it thought at the time; where the seat took something else (a person's seat, the clock's
  pick, or a drafter changed since) a `!!` line says what it took. A grid or Winston event keeps no record of what
  each seat was shown, only what was taken, so it cannot be replayed this way: it would have to be dealt again from
  the event's seed with the server's own shuffle.

**Reading it.** A booster pick, PowerLSV:

```
Pack 3 pick 2: 14 cards, pool 31. Took Memory Lapse.
  deck so far: UB 100%
  open, worth a pick: B +0.12 U +0.05 R +0.02 W -0.09 G -0.11 <- passed me this pack U +0.24 R +0.17 W -0.06 B -0.10 G -0.26 (x0.12); pack 1 B +0.24 W -0.14 G -0.14 (x0.55); taken over better W +0.05 (x-0.2)
  -> Memory Lapse            +1.11  worth 4.0 #64       UB +0.99 (100%), UG +1.80 (0%)
                            in UB: plays as 3.9; in place of a pick to come 2.6; not a creature -0.25 = +0.99
     Strip Mine              +1.08  worth 4.6 #30       8 decks (UB/UG...) +0.94 (100%)
                            in UB: a utility land +0.94 = +0.94
     Gut, True Soul Zealot   +0.14  worth 4.7 #24       BR +2.65 (0%), UR +3.44 (0%)
                            in BR: plays as 4.5; in place of a pick to come 2.1; a creature it wants +0.20 = +2.65
     also: Fractured Identity +0.27, Brazen Borrower +0.23, Windswept Heath +0.13, Burst Lightning +0.09
```

- A card's **score** is what the decks the pool could become gain with it, each weighed by how likely it is, and a
  little of its worth to break ties. **Worth** is the cube's scale (3 a middling card, 5 one of its best twenty, 7 the
  Power Nine), from its place (`#`) in the pick order the server sent.
- `UB +0.99 (100%)`: a UB deck gains 0.99 with the card, and the pool is that likely to be UB. Decks it adds the same
  to are named together (`W pairs`, `any pair`). The best of them is broken down on the next line: what the card
  **plays as** there (its worth as far as the lands cast it, less for a cheat with nothing to cheat, a Mox of the
  wrong colour, a splash), what it takes the place of in the deck's twenty-three (one of the pool's cards, or a pick
  still to come, which is what the drafter expects the rest of the draft to bring), what it does to the lands and the
  shape (creatures, early plays, five-drops), and anything else it changes in the deck - the cheats a fatty feeds, the
  splash a land lets in.
- The closest three cards are told, and the best card by the pick order whatever it scored: why it was passed is the
  first question.
- **open**: how much better than usual each colour's coming picks are expected to be, in worth a pick, and after `<-`
  what that was read from (each colour's share of the playables in the packs against the average colour) with the
  weight it is given (`DraftTuning`). **wheel**: what did not come back of a pack seen before, and which of it was taken
  over better cards - somebody's colour.

A deck:

```
Deck: UB+g, 24 spells and 16 lands, worth 84.0 (pool 45).
  next best: UB 83.3, UB+w 83.3, UG+b 72.2, BG+u 70.9
  splash g for Oko, Thief of Crowns: +0.68 over none, 3.0 sources
  lands: mana value 2.96 asks 16.8, 2 cheap mana or draw -0.6 = 16.2: 16
  mana: Starting Town, Polluted Delta, Sunken Ruins, Indatha Triome, Zagoth Triome, Darkslick Shores, Creeping Tar Pit, 5 Island, 4 Swamp; sources U 11.0 B 11.0 G 3.0
  shape: 8 creatures, 11 at 2 or less, 3 at 5 or more; leans late (-0.31)
   1: Tamiyo, Inquisitive Student, Sensei's Divining Top, Mind Twist, Fatal Push, Grindstone, Demonic Consultation
   ...
  last in: Ertai Resurrected 2.2, Collective Brutality 2.5, Sheoldred's Edict 2.7; first out: Lim-Dûl's Vault 2.2, Generous Ent 1.6
  best of the rest: Abrade (R) 3.2, No More Lies (WU) 2.8
```

A grid pick lists every line with what the pool gains by it and each card's worth (and, with `Deny` on, what it leaves
the other player); a Winston pile says what the pile adds against what passing it comes to - the next pile's unseen
cards, or a card from the stack.

## What was taken from elsewhere

- **CubeCobra's draft bots** score a card over every colour combination by the chance of casting it from a
  seventeen-land allocation, and weigh "openness" - the worth of cards seen - most around the first wheel. The
  hypotheses-with-castability shape here is theirs; projecting each to the end of the draft, rather than five
  weighted "oracles", is not.
- **Draftsim's** bots add a bonus to cards of colours the pool has committed to, colourless cards getting the best
  of it, with a hard switch by the fourth pick of pack two. Here that falls out of the sum, with no switch.
- **17lands** found the best card of each colour left in a pack a usable signal, and pack one's signals still
  telling in pack three: the flow and its carry. Nobody published a weighting for what is *passed* or for what does
  not wheel; those are this bot's own.
- **Frank Karsten's** land count and coloured-source arithmetic, and Reid Duke's cube conventions (a Mox for a land,
  fifteen to eighteen lands, three or four sources for a splash).
- **Forge's** limited AI's curve caps were the starting point for the deck's shape.

## How good it is

`tools\bot-draft.cmd` (`cantrip-botlab draft`) drafts pods with nobody at the table: the server's own `BoosterDraft`
with each seat shown what a client is shown, two ways of drafting in alternate seats and the other way round in every
second pod. A pod drafts and builds in a few milliseconds. With `--games N` every deck one way made plays every deck
the other made in its pod, the tactical bot playing both sides.

As shipped (2026-10-03), 60 pods of LSVCube, 1,900 games a line, the same pilot in both seats. The margin is taken
over pods, not games: the same eight decks play all thirty-two of a pod's games, so a pod is one sample.

| | won | |
|---|---|---|
| the cube drafter and its deck, against the pick-order bot and its deck | 66.8% +/- 3.2% | 1281 to 637 |
| its picks alone: both built by the new builder | 62.0% +/- 3.2% | 1190 to 728 |
| its builder alone: both drafted by the pick-order bot | 55.9% +/- 3.9% | 1074 to 846 |

What the decks are says why. The pick-order bot's pool is the best forty-five cards it was passed in five colours, and
its deck is twenty spells and twenty lands, seven creatures, with five good cards it could not cast left in the
sideboard. The cube drafter's is twenty-five spells and fifteen lands, thirteen creatures, two colours and usually a
splash, with next to nothing good left out; by the cube's pick order its spells are worth a third more, and its lands
cast all but a twentieth of that.

A pick takes under a millisecond and a deck about twenty, on one core. Drafting PowerLSV gives much the same decks.
With no pick order sent at all (`--no-order`), so that every card is worth only what it reads as, it still builds
two-colour forty-card decks of twelve creatures, worth about a tenth less by the order it was not shown. Every deck
the lab builds is put to the server's own check of a submitted deck (`PoolDecks.Built`), and a pod with a Cogwork
Librarian in it spends it as a seat would.

Most of the drafter's numbers were set against each other in play, forty pods and 1,300 games a pair, which tells
apart about four points either way and no less. None of these showed: how sure the pool is of its colours
(`Sureness` 0.5 against 2), twenty-three deck slots against twenty-five, what a dual land is worth (`Fixing` 0, 0.5,
1), three splashed cards against five, a land more or a land fewer than the curve asks, and reading signals at all
(every signal weight at nothing lost 942 to 961). Never splashing came out behind twice, at 46% and 47%, which is why
a splash costs nothing beyond its sources (`DeckTuning.SplashCost`). That signals do not show is the lab's answer -
among drafters like itself, in a cube where every colour is deep, the forecasts above account for a twentieth of
what a colour goes on to deliver at best - and not necessarily a table of people's.

**What does show, and what it is.** `--out` writes `decks.csv`, every deck as columns (lands, creatures, removal,
counterspells, cards of each kind, what the pilot cannot read, which of five archetypes it is), and with games a
`games.csv`; `draft shape --from DIR` fits who beat whom to the difference between the two decks' columns, with
margins from drawing the pods again, and `--cards N` does the same card by card. Among the cube drafter's own decks,
with the pilot of 848e2373 (`results/draft-1-*` in the lab exchange has the tables): a deck's worth by the pick order,
as far as its lands cast it, predicts nothing (0.0 +/- 0.3 points of win rate a point of worth); a creature is worth
3.5 +/- 0.5 points over a non-creature permanent, a removal spell 3.2 +/- 0.6, a land 2.5 +/- 1.1, a planeswalker 1.5
+/- 0.6, a card-draw spell 1.2 +/- 0.6, and a counterspell, a mana rock or an instant the pilot cannot read nothing;
the curve terms (`Cheap`, `Expensive`) nothing. The builder's own sum calls games no better than a coin. Every Mox,
Mana Vault, Mana Crypt and Black Lotus come out *below* an ordinary card, which the play bot's own mirror tables
confirm is the pilot (it left them in hand). So the creature finding is the pilot's: asked for seventeen creatures at
three times the price (`new:Creatures=17,CreatureCost=1.5`) the drafter beats itself 69.0% +/- 2.8% in alternate
seats, but builder-only (`Build.Creatures=17`, the same picks) gains 0.1% +/- 1.7%, and a whole table of such
drafters beats a whole table of the shipped one 53.1% +/- 1.7% (`--apart`): the rest is taken from the neighbours,
who are left eight creatures. The middle course, fifteen at 0.8, is 57.5% +/- 3.2% in alternate seats and 50.6% +/-
1.5% table against table. With the play bot's pilot of 9029b602, which casts its Moxen and Mana Vault, crews and
attacks planeswalkers, the same packs say the same: seventeen creatures beat thirteen 53.5% +/- 1.7% table against
table (52.3% with the referee, +1.1 +/- 1.4 paired) and 66.5% in alternate seats (70.7%, -4.2 +/- 2.0 paired: less
is taken from the neighbours). The shipped target stays at thirteen: a drafter tuned to its pilot's blind spots passes the
cube's best cards to whoever sits beside it, and every bot deck a creature deck is no variety at all.

Each of the drafter's numbers (`DraftTuning`, `DeckTuning`) can be set against the shipped one the same way:
`bot-draft.cmd --a new:Signals=0 --b new --games 2`; `Build.Creatures=15` sets a deck number for the builder alone
(the same picks), `Draft.Creatures=15` for the picks alone, `Varied=1` draws every seat's deck numbers afresh (for
`draft shape` to read), and `--apart` gives each way of drafting a whole table of the same packs, so that what one
takes from the seats beside it is told from what it gains. `draft paired --base DIR --variant DIR` sets two runs of the same `--pods` and `--seed` side by side pod by pod: the
tight test of a change to one side (+/-1.3% at 60 pods against +/-2.7%). `--format grid` or `--format winston` drafts
tables of two instead, one seat of each way. `--keep control:1..` (or any column, `name:least..most`) with `--out`
writes only such decks: a pool of one kind. `bot-draft.cmd watch --seat 3` tells one draft pick by pick as a
narrated bot seat does ("What it was thinking", above), and
`--cards N` lists the cards whose decks did worst and best, which is how a card the playing bot cannot use is found.

What it does not do, and would be the next things to teach it:

- It drafts colours, not archetypes. The synergies it knows are which way a deck leans, early or late, and that a
  cheat needs something big to cheat; not storm and its rituals, not a graveyard deck's discard outlets, not
  Equipment for a Stoneforge, not what a tutor should find.
- It plays at most two colours and a splash, however good the pool's fixing.
- It does not hate-draft, and does not count what the seat it passes to is likely to be.
- Its opponents in the lab are itself and the pick-order bot. What it does beside people is not measured.
- Two numbers are there to be measured and are off as shipped: `DeckTuning.Ramp`, more five-drops held for each card
  that makes mana beyond two, and `DeckTuning.Outlets`, a reanimation spell needing a way to put its target into the
  graveyard (`DraftCard.Outlet`: looting, discarding as a cost, Entomb). `Ramp` at 1 changed hardly a deck and
  nothing in games (+0.1% +/- 1.0% paired, LSVCube; +0.4% +/- 1.8% PowerLSV; pilot of 9029b602). `DeckTuning.LateFewer`
  (a control deck wants four creatures fewer) at 2 did nothing either: builder-only the same decks, table against table
  50.5% +/- 1.4%. A control deck's pool has no more creatures to give.
- The deck is judged by a pilot (`TacticalPlay`) that plays creatures and removal well and counterspells, tutors and
  rituals badly, so the lab flatters aggressive decks: by archetype, with that pilot, its aggro decks win 69.6% +/-
  6.8% against the pick-order bot's decks, midrange 63.2% +/- 3.6%, combo 58.5% +/- 8.8%, ramp 56.9% +/- 9.6% and
  control 45.0% +/- 8.4% (LSVCube, 60 pods). The drafter is deliberately not tuned to that.
