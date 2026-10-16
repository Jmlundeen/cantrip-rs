//! Decisions are data (docs/engine-design.md, "Decisions are data"): what the engine is in the middle of is a
//! stack of frames in the state, and a question waiting for an answer is a frame holding the typed request.
//!
//! The requests mirror the C# engine's prompts kind for kind and option for option, in the same order, because the
//! protocol the clients speak and the scenarios' "the first one offered" both depend on it.

use crate::object::*;
use crate::vocab::*;

/// One unit of work in progress. The engine runs the top frame; a frame that asks a question stops there, holding the
/// request, and runs again from its start when the answer arrives, with its answers so far replayed (or, for a frame
/// written as explicit stages, from the stage it stopped at).
#[derive(Clone, Debug)]
pub struct Frame {
    pub kind: FrameKind,
    /// The answers this run of the frame has been given, in the order it asked.
    pub answers: Vec<Answer>,
    /// The question it is waiting on.
    pub pending: Option<Request>,
    /// Where the journal stood when this run of the frame began: what an answer rolls back to before the re-run.
    /// Bookkeeping of the run, not of the game: written without the journal, and no part of the position.
    pub mark: u32,
    /// Where the game log stood then, for the same reason.
    pub log_mark: u32,
    /// For a frame written as explicit stages: the stage to continue from, and values kept between stages. A staged
    /// frame is not rolled back when answered.
    pub stage: u16,
    /// The stage a staged frame's pending question continues at.
    pub next_stage: u16,
    pub staged: bool,
    pub scratch: [u32; 4],
}

impl Frame {
    pub fn new(kind: FrameKind) -> Frame {
        Frame {
            kind,
            answers: Vec::new(),
            pending: None,
            mark: 0,
            log_mark: 0,
            stage: 0,
            next_stage: 0,
            staged: false,
            scratch: [0; 4],
        }
    }

    fn position(&self) -> (&FrameKind, &Vec<Answer>, Option<(&PlayerId, &RequestKind)>, u16, u16, bool, &[u32; 4]) {
        (
            &self.kind,
            &self.answers,
            self.pending.as_ref().map(|r| (&r.player, &r.kind)),
            self.stage,
            self.next_stage,
            self.staged,
            &self.scratch,
        )
    }
}

/// Two frames are the same work in progress whatever bookkeeping their runs carry: the marks and request ids.
impl PartialEq for Frame {
    fn eq(&self, other: &Frame) -> bool {
        self.position() == other.position()
    }
}

impl Eq for Frame {}

impl std::hash::Hash for Frame {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.position().hash(state);
    }
}

/// What a frame is doing. Only the core's procedures are here: card behavior runs inside them, by function index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FrameKind {
    /// The game's start (CR 103): who goes first, opening hands, mulligans.
    Opening,
    /// The current step's turn-based actions (CR 500.2), then priority if the step has it.
    BeginStep,
    /// A player has priority (CR 117).
    Priority,
    /// State-based actions until none apply, then triggered abilities onto the stack (CR 117.5).
    Settle,
    /// Taking an action a player chose with priority: playing a land, casting a spell (CR 601.2), activating an
    /// ability (CR 602.2). The action is the frame's first answer.
    Act { player: PlayerId },
    /// Resolving the top of the stack (CR 608).
    Resolve,
}

/// A question for one player.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Request {
    /// Never reused within a game.
    pub id: u32,
    pub player: PlayerId,
    pub kind: RequestKind,
    /// Asked while a spell or ability is still being announced, so the player may give it up (CR 733.1).
    pub may_cancel: bool,
}

/// The kinds of action an option can be, as the C# engine numbers them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActionKind {
    PlayLand,
    CastSpell,
    ActivateAbility,
    CancelCasting,
    /// Pay an "unless" cost now that the pool covers it.
    PayCost,
    DeclineCost,
    /// Put a unit of pool mana of this kind towards the cost.
    AssignMana,
    ConfirmCasting,
    /// While announcing (CR 601.2b): one more for X, or one more time kicked.
    IncreaseX,
    DecreaseX,
    /// While paying (CR 601.2h): tap this permanent to pay for convoke (CR 702.51a) or improvise (CR 702.126a); the
    /// option's `index` is which of `PAY_*` it pays for.
    TapToPay,
    /// While paying (CR 601.2h): exile this card from the graveyard to pay for delve (CR 702.66a).
    ExileToPay,
}

/// Which way of paying with something other than mana a `TapToPay` or `ExileToPay` option is (its `index`).
/// The `index` of an `ActivateAbility` option that is a mana ability of the player's own (CR 605.1a, 611.2a) and not of any
/// object: the nth effect that gave them one, added to this. Its `source` is the card that gave it, wherever it is now.
pub const PLAYER_MANA_ABILITY: u8 = 200;

pub const PAY_DELVE: u8 = 0;
pub const PAY_IMPROVISE: u8 = 1;
pub const PAY_CONVOKE: u8 = 2;
/// Waterbend (CR 701.67a): an untapped artifact or creature tapped pays {1}.
pub const PAY_WATERBEND: u8 = 3;

/// One thing a player could do: an option of a priority, payment or "unless" question, as a prototype the answer
/// fills in (targets, a color).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ActionOption {
    pub kind: ActionKind,
    pub source: Option<ObjectId>,
    /// For a cast, which way (0 its mana cost, n its nth alternative cost); for an activation, which ability of the
    /// source's computed ability list.
    pub index: u8,
    /// Targets the action is announced with, and who may be chosen for them (CR 601.2c): the candidates of every
    /// target slot together.
    pub targets_required: u8,
    pub targets_optional: bool,
    pub target_candidates: Vec<Target>,
    /// A mana ability's colors to choose from, when it adds one mana of any one color (CR 106.5).
    pub color_choices: ColorSet,
    /// A mana ability whose sacrificed permanent sets the mana (CR 106.7) may sacrifice one that could produce none:
    /// then it is activated naming no type, and adds no mana.
    pub names_nothing: bool,
    /// For `AssignMana`: the kind of mana.
    pub mana: Option<Mana>,
    /// For `AssignMana`: the restriction on the mana (CR 106.6), if it has one.
    pub restriction: Option<ManaRestriction>,
}

impl ActionOption {
    pub fn simple(kind: ActionKind) -> ActionOption {
        ActionOption {
            kind,
            source: None,
            index: 0,
            targets_required: 0,
            targets_optional: false,
            target_candidates: Vec::new(),
            color_choices: ColorSet::NONE,
            names_nothing: false,
            mana: None,
            restriction: None,
        }
    }

    pub fn on(kind: ActionKind, source: ObjectId, index: u8) -> ActionOption {
        ActionOption { source: Some(source), index, ..ActionOption::simple(kind) }
    }
}

/// A taken action: an option, filled in.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Action {
    pub kind: ActionKind,
    pub source: Option<ObjectId>,
    pub index: u8,
    pub targets: Vec<Target>,
    pub color: Option<Color>,
    pub mana: Option<Mana>,
    pub restriction: Option<ManaRestriction>,
}

impl Action {
    pub fn of(option: &ActionOption) -> Action {
        Action {
            kind: option.kind,
            source: option.source,
            index: option.index,
            targets: Vec::new(),
            color: None,
            mana: option.mana,
            restriction: option.restriction,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RequestKind {
    /// Act or pass (CR 117). Passing is always legal.
    Priority { options: Vec<ActionOption> },
    /// Pay what is still owed on a spell or ability (CR 601.2g-h): activate a mana ability, put pool mana towards it,
    /// confirm once paid, or give up.
    PayCosts {
        source: ObjectId,
        total: ManaCost,
        remaining: ManaCost,
        options: Vec<ActionOption>,
        /// Announcing X or how many times it is kicked (CR 601.2b), with the value so far, rather than paying.
        announcing: bool,
        x: u16,
    },
    /// "Unless [a player] pays" (CR 118.12): activate mana abilities, pay once the pool covers it, or decline.
    UnlessPays { source: ObjectId, subject: ObjectId, total: ManaCost, options: Vec<ActionOption> },
    /// Choose `count` (or up to `count`) of `eligible`, from among `cards` (all of which the player sees).
    /// `kind` is what the asking card calls what may be chosen when it says more than "card" ("artifact or creature",
    /// "nonland"), empty otherwise: the dock words the choice as the card does.
    ChooseCards { source: ObjectId, purpose: Purpose, cards: Vec<ObjectId>, eligible: Vec<ObjectId>, count: u8, up_to: bool, kind: &'static str },
    /// `least` to `count` targets are chosen (a range, "one or two target creatures", CR 601.2c); `optional` is `least == 0`.
    /// Targets for an ability as it goes on the stack, or any other "choose a [thing]" the C# engine asks as targets.
    /// `label` is what the slot is for as its card words it (`TargetSpec::label`, empty when it has nothing to add), and
    /// `slot_player` the opponent whose slot it is when the ability has one for each opponent.
    ChooseTarget { source: ObjectId, purpose: Purpose, candidates: Vec<Target>, optional: bool, least: u8, count: u8, any_number: bool, label: &'static str, slot_player: Option<PlayerId> },
    /// One or more of these labelled options (a pile, a mode), by index.
    /// `count` to `at_most` of them when `at_most` is more than `count` (zero `count` is "up to"). `affordable` is
    /// the eligible modes whose own cost the chooser could pay as well, advice for a default; `additional_cost`
    /// marks choices that are costs (Phyrexian mana paid with life, CR 107.4f) rather than modes; `repeat` lets one
    /// mode be chosen more than once (CR 700.2d).
    ChooseModes { source: ObjectId, modes: Vec<Label>, eligible: Vec<u8>, count: u8, at_most: u8, affordable: Vec<u8>, additional_cost: bool, repeat: bool },
    ChooseColors { source: ObjectId, choices: ColorSet, count: u8 },
    /// A card name (CR 201.3, 614.12a): any card the game knows. `looked_at` are the cards the player has just been
    /// shown, and `suggestion` the name an unscripted player takes (the costliest nonland card among them).
    /// With `nonland_only` the name must be a nonland card's ("choose a nonland card name").
    ChooseName { source: ObjectId, looked_at: Vec<ObjectId>, suggestion: Option<CardId>, nonland_only: bool },
    /// One subtype of `choices`, in order ("choose a creature type", CR 205.3m: the C# engine asks it as a modes
    /// question over every creature type, more than a mode index holds).
    ChooseSubtype { source: ObjectId, choices: Vec<Subtype> },
    /// "You may".
    May { source: ObjectId, purpose: Purpose },
    /// A player chosen without targeting (CR 608.2d, 614.12a): True-Name Nemesis's "as this enters, choose a player",
    /// Baleful Mastery's "an opponent draws a card". Answered with `Answer::Player`. `verb` and `amount` say what the
    /// chosen player will do, as hint text.
    ChoosePlayer { source: ObjectId, candidates: Vec<PlayerId>, verb: &'static str, amount: u16 },
    /// Tap any number of untapped creatures with total power `minimum` or more (CR 702.122a crew, CR 702.171a saddle):
    /// any subset of `cards` whose `powers` (what each counts as towards the total, CR 702.122b) add up to it.
    TapForPower { source: ObjectId, purpose: Purpose, cards: Vec<ObjectId>, powers: Vec<u8>, minimum: u8 },
    /// Collect evidence N (CR 701.59a): exile any subset of `cards` (the player's graveyard) whose `values` (their mana
    /// values) add up to `minimum` or more.
    CollectEvidence { source: ObjectId, cards: Vec<ObjectId>, values: Vec<u8>, minimum: u8 },
    /// The targets of a slot that divides a total among them, and each one's share (CR 601.2d): "distribute two +1/+1
    /// counters among up to two target creatures". At most `max_targets` of `candidates` (any number when 0), each with a
    /// share of at least 1, adding up to `total`; none at all when the slot is `optional`.
    ChooseDividedTargets { source: ObjectId, candidates: Vec<Target>, total: u16, max_targets: u8, optional: bool },
    /// Declare attackers (CR 508.1): any of `eligible`, each at one of `players`; every one of `requirements` must
    /// attack (CR 508.1d).
    /// Who attacks what (CR 508.1): each attacker names a player, a planeswalker a defending player controls, or a battle a defending player protects (`battles`,
    /// CR 310.9b, 508.1b).
    DeclareAttackers { eligible: Vec<ObjectId>, players: Vec<PlayerId>, planeswalkers: Vec<ObjectId>, battles: Vec<ObjectId>, requirements: Vec<ObjectId> },
    /// How creatures divide their combat damage (CR 510.1c-d): one division per creature with a choice.
    AssignCombatDamage { divisions: Vec<DamageDivision> },
    /// Declare blockers (CR 509.1): `allowed` lists every (blocker, attacker) pair the rules allow.
    DeclareBlockers { eligible: Vec<ObjectId>, attackers: Vec<ObjectId>, allowed: Vec<(ObjectId, ObjectId)> },
    /// Discard down to maximum hand size (CR 514.1).
    DiscardToHandSize { count: u8, hand: Vec<ObjectId> },
    /// Keep the opening hand or take a mulligan (CR 103.5).
    Mulligan { mulligans: u8 },
    /// Who takes the first turn (CR 103.1).
    ChooseStartingPlayer { candidates: Vec<PlayerId> },
    /// The order a player's triggered abilities go on the stack (CR 603.3b): a permutation, first on the stack first.
    OrderTriggers { sources: Vec<ObjectId>, abilities: Vec<AbilityRef> },
    /// Cards looked at, each to be put in one of the destinations (Ponder, surveil, "put one into your hand and the
    /// rest into your graveyard"). `cards` are top of the library first.
    DistributeCards { source: ObjectId, cards: Vec<ObjectId>, destinations: Vec<Destination>, may_shuffle: bool },
    /// Search a library (CR 701.23): `library` is the whole library, `eligible` what the search may find; with
    /// `optional`, finding nothing is allowed (CR 701.23b).
    /// `destination` is where the searcher is told the cards found are going; whose library it is, is the owner of the
    /// cards in `library`.
    SearchLibrary { source: ObjectId, library: Vec<ObjectId>, eligible: Vec<ObjectId>, count: u8, optional: bool, destination: SearchDestination },
    /// Which of several replacement effects applies to an event first (CR 616.1), asked of the affected player:
    /// `sources` are the objects whose effects apply and `hints` say what each does, in the order they are offered.
    /// `subject` is the object the event is about, if it is about one.
    ChooseReplacement { subject: Option<ObjectId>, sources: Vec<ObjectId>, hints: Vec<&'static str> },
}

/// Where the cards a library search finds are going (CR 701.23), as the searcher is told.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SearchDestination {
    Hand,
    TopOfLibrary,
    Graveyard,
    Battlefield,
    Exile,
    ExileFaceDown,
    /// Cast by the searcher as the effect resolves.
    Cast,
    /// Where it goes depends on the card found (Transmute Artifact): the search says only what it looks for.
    /// Onto the battlefield tapped (Nissa, Winds of Abandon).
    BattlefieldTapped,
    /// Cast by the searcher without paying its mana cost (Sunforger).
    CastFree,
    Found,
}

/// One creature's combat damage to divide among the creatures it is blocked by (and, with trample, the player or
/// planeswalker it attacks), with what is lethal to each and the engine's own suggestion.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DamageDivision {
    pub source: ObjectId,
    pub damage: u16,
    pub creatures: Vec<ObjectId>,
    pub lethal: Vec<u16>,
    pub may_assign_to_defender: bool,
    pub defender: Target,
    pub suggested: Vec<u16>,
    pub suggested_to_defender: u16,
}

impl DamageDivision {
    /// The engine's own division of every creature's combat damage, as an answer's shares: lethal to each in turn,
    /// the rest to the defender.
    pub fn suggested_shares(divisions: &[DamageDivision]) -> Vec<(ObjectId, Option<ObjectId>, u16)> {
        let mut shares = Vec::new();
        for d in divisions {
            for (i, &c) in d.creatures.iter().enumerate() {
                if d.suggested[i] > 0 {
                    shares.push((d.source, Some(c), d.suggested[i]));
                }
            }
            if d.suggested_to_defender > 0 {
                shares.push((d.source, None, d.suggested_to_defender));
            }
        }
        shares
    }
}

/// One place the cards of a `DistributeCards` can go, and how many.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Destination {
    pub kind: DestinationKind,
    pub min: u8,
    /// `u8::MAX` for "the rest".
    pub max: u8,
    /// The cards it may take; every card looked at, when it takes any.
    pub eligible: Vec<ObjectId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DestinationKind {
    LibraryTop,
    LibraryBottom,
    Graveyard,
    Hand,
    Exile,
    /// Onto the battlefield face down (CR 701.40a, 708.3): manifest.
    BattlefieldFaceDown,
}

/// What a gift is (CR 702.174d-i): the effect the chosen opponent gets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Gift {
    /// "Gift a Food": the chosen player creates a Food token (CR 702.174d).
    Food,
    /// "Gift a card": the chosen player draws a card (CR 702.174e).
    Card,
    /// "Gift a tapped Fish": the chosen player creates a tapped 1/1 blue Fish creature token (CR 702.174f).
    TappedFish,
    /// "Gift an extra turn": the chosen player takes an extra turn after this one (CR 702.174g).
    ExtraTurn,
    /// "Gift a Treasure": the chosen player creates a Treasure token (CR 702.174h).
    Treasure,
    /// "Gift an Octopus": the chosen player creates an 8/8 blue Octopus creature token (CR 702.174i).
    Octopus,
}

/// Why a player is choosing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Purpose {
    Discard,
    Sacrifice,
    /// A card chosen from a revealed hand (Duress).
    TakeFromHand,
    /// Which objects go in the first pile (CR 700.3).
    SeparatePiles,
    /// Which pile.
    ChoosePile,
    /// Enter as a copy (CR 707.2).
    CopyAsEnters,
    /// A creature to copy.
    CopyWhich,
    /// The targets of a triggered or activated ability.
    Targets,
    Color,
    /// A cost's sacrifice (Fireblast's Mountains).
    SacrificeCost,
    /// Permanents returned to their owners' hands as a cost (Gush's Islands).
    ReturnToHand,
    /// Cards put back on top of a library (Brainstorm).
    PutOnLibrary,
    /// A card put onto the battlefield (Show and Tell).
    PutOntoBattlefield,
    /// A plain "you may" with nothing more to say about it.
    Optional,
    /// The rest of the C# engine's purposes (CardChoicePurpose), one for one.
    PutOnTop,
    KeepLegend,
    Exile,
    Untap,
    ExileToPay,
    Keep,
    PutIntoHand,
    Cast,
    TapToPay,
    OrderSimultaneous,
    Destroy,
    Vote,
    PutIntoGraveyard,
    Play,
    Splice,
    PutOnBottom,
    Attach,
    /// A creature to exile a card with cipher encoded on (CR 702.99a).
    Encode,
    /// Whether to promise a gift (CR 702.174a), and which gift it is.
    PromiseGift(Gift),
    /// "You may reveal this card from your hand as you draw it" (CR 702.94a): miracle.
    RevealForMiracle,
    /// "You may cast it by paying its miracle cost" (CR 702.94a), as the linked ability resolves.
    CastForMiracleCost,
    /// Whether to pay another cost rather than an activated ability's (CR 118.9).
    AlternativeCost,
    /// What a cost beholds (CR 701.4a): a permanent chosen or a card revealed from the hand.
    Behold,
    /// The creature a cost blights (CR 701.68a).
    Blight,
    /// The permanent that gets this many counters of this kind, chosen without targeting (CR 122.1, 608.2d): which Jace
    /// planeswalker token empower Jace puts its loyalty counters on (Theorist's Sanctum, Fatehold Charm, Countersculpt),
    /// which Army amass puts its +1/+1 counters on (CR 701.47a; Orcish Bowmasters).
    PutCounters(CounterKind, u8),
    /// The creature harmonize taps, if any (CR 702.180a-b): up to one untapped creature, whose power comes off the generic
    /// mana of the total cost.
    TapToReduce,
    /// "You may cast it without paying its mana cost" as an effect resolves (CR 118.9, 608.2g), the card staying where it is
    /// if not: suspend, rebound, hideaway, Malcolm.
    CastFree,
    /// "You may cast a copy of it without paying its mana cost" (CR 707.12): paradigm, cipher.
    CastCopyFree,
    /// "You may cast it transformed without paying its mana cost" (CR 310.12b): a Siege defeated.
    CastTransformedFree,
    /// "You may cast it by paying its madness cost" (CR 702.35a).
    CastForMadness,
    /// "You may cast it without paying its mana cost. Put it into your hand if you don't" (Beseech the Mirror).
    CastFoundOrHand,
    /// "You may cast it without paying its mana cost. If you don't, it goes on the bottom of your library" (cascade).
    CastOrBottom,
    /// "You may cast any number of copies without paying their mana costs" within a budget (Uldaros Theorix): a copy chosen to be
    /// cast, with this much mana value left to spend in all.
    CastCopyFreeWithin(u8),
}

/// The text of one option, as an index, so no string is built at runtime.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Label {
    /// The nth label of the source card's definition.
    Card(CardId, u8),
    /// A card's name: an option that is one of several cards, named as the C# engine offers them.
    Name(CardId),
    /// One of two piles (CR 700.3), and whether it has nothing in it.
    Pile { first: bool, empty: bool },
    /// The nth mode of a modal spell (its ability has index `u8::MAX`) or ability.
    Mode(AbilityRef, u8),
    /// The nth way of paying the source card's additional cost (`CardDef::additional_choice`).
    Extra(CardId, u8),
    /// "Pay 2 life for {W/P}": one Phyrexian symbol paid with life (CR 107.4f).
    PayLife(Color),
    /// A kind of counter, to remove one of as a cost (CR 118.3).
    Counter(CounterKind),
    /// One of the faces a card may be cast as (CR 709.3, 715.3, 712.8a), named by the face's name.
    Face(CardId),
}

/// An answer to a request.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Answer {
    Pass,
    Act(Action),
    Cards(Vec<ObjectId>),
    Targets(Vec<Target>),
    Modes(Vec<u8>),
    Colors(Vec<Color>),
    /// "You may", and a mulligan (yes is take one).
    Yes,
    No,
    Attack(Vec<(ObjectId, Target)>),
    Block(Vec<(ObjectId, ObjectId)>),
    Player(PlayerId),
    /// A card name chosen (CR 201.3).
    Name(Option<CardId>),
    /// A subtype chosen.
    Subtype(Subtype),
    /// Indices in order: the order to put triggered abilities on the stack.
    Order(Vec<u8>),
    /// Combat damage shares: (source, the creature it is dealt to, or `None` for the defender, amount).
    Shares(Vec<(ObjectId, Option<ObjectId>, u16)>),
    /// Where each looked-at card goes: one list per destination, in placing order (for a library top, the first is
    /// the top card); or, where allowed, shuffle instead.
    Placements { placements: Vec<Vec<ObjectId>>, shuffle: bool },
    /// A divided slot's targets, each with its share (CR 601.2d).
    Divided(Vec<(Target, u16)>),
    /// Give up the spell or ability being announced (CR 733.1).
    Cancel,
    /// The index of the option chosen, for a question of one-of-several (which replacement effect applies first).
    Index(u8),
}
