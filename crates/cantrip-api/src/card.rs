//! What a card is: its characteristics, and its behavior as code, reached by function pointer. Nothing about a card is
//! read from data at runtime, and nothing is parsed.

use cantrip_state::*;

use crate::chars::Chars;
use crate::event::{Event, EventMask};
use crate::game::{Game, Query, Suspend};
use crate::replace::{GrantedReplacementDef, ReplacementDef};

/// A spell or ability's effect as it resolves (CR 608.2).
pub type Effect = fn(&mut dyn Game, &StackItem) -> Result<(), Suspend>;

/// Whether a target is legal for a target slot (CR 115).
pub type TargetFilter = fn(&dyn Query, &Asker, Target) -> bool;

/// Whether an event triggers an ability of `source` (CR 603.2), and what the ability should carry of it.
pub type TriggerMatch = fn(&dyn Query, ObjectId, &Event) -> Option<Fired>;

/// What a triggered ability carries of the event that triggered it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Fired {
    /// The player or object the event was about: "that player", "that card". Not a target.
    pub subject: Target,
    pub data: [u32; 2],
}

impl Fired {
    pub const PLAIN: Option<Fired> = Some(Fired { subject: Target::None, data: [0; 2] });

    pub fn about(subject: Target) -> Option<Fired> {
        Some(Fired { subject, data: [0; 2] })
    }
}

/// An intervening "if" (CR 603.4): checked as the ability triggers and again as it resolves.
pub type TriggerCondition = fn(&dyn Query, &Asker) -> bool;

/// "As [this] enters" and "[this] enters with" (CR 614.1c-d, 614.12): applied to the object about to enter, which
/// is not on the battlefield yet.
pub type Enters = fn(&mut dyn Game, &mut Entering) -> Result<(), Suspend>;

/// A permanent about to enter the battlefield, and how.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entering {
    pub object: ObjectId,
    pub controller: PlayerId,
    pub tapped: bool,
    pub counters: [i16; CounterKind::COUNT],
    pub copy: Option<Copiable>,
    /// It enters prepared (CR 722.2).
    pub prepared: bool,
    /// The color or type chosen as it enters (CR 614.12), kept on the permanent.
    pub chosen_color: Option<Color>,
    pub chosen_subtype: Option<Subtype>,
    /// The player chosen as it enters (CR 614.12a): True-Name Nemesis.
    pub chosen_player: Option<PlayerId>,
    /// The card name chosen as it enters (CR 614.12a).
    pub chosen_card: Option<CardId>,
    /// Odd (true) or even (false), chosen as it enters (CR 614.12a): Gollum, Riddle Master.
    pub chosen_odd: Option<bool>,
    /// It enters attached to this permanent (CR 303.4f): an Aura entering from the stack, or choosing its host as it
    /// is put onto the battlefield (CR 303.4g).
    pub attach: Option<ObjectId>,
    /// It enters enchanting this player (CR 303.4f): an Aura with "enchant player" entering from the stack.
    pub attach_player: Option<PlayerId>,
    /// The X the spell it was cast as announced (CR 107.3m), which "enters with X counters" reads.
    pub x: u16,
    /// It enters with its back face up (CR 712.14a): its entering replacements and loyalty are the back face's.
    pub transformed: bool,
    /// The player chosen to protect a battle as it enters (CR 310.9a, 310.12a), by `prepare_entry`.
    pub protector: Option<PlayerId>,
    /// It does not enter: it goes to this zone instead (CR 614.12, 614.1a), as Mox Diamond's replacement sends it to the graveyard
    /// when its controller does not discard a land.
    pub instead: Option<Zone>,
}

/// Where a spell countered by `Game::counter_to` goes instead of its owner's graveyard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CounterTo {
    Hand,
    LibraryTop,
    LibraryBottom,
    Exile,
}

/// Where cards put into their owners' libraries go (CR 401.4), for `Game::put_into_library`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibraryPlace {
    /// On top, the first of the cards given topmost.
    Top,
    /// On the bottom, the cards given in order, the last of them lowest.
    Bottom,
    /// On the bottom in a random order, drawn from the game's own random source (CR 401.4).
    BottomInRandomOrder,
    /// Into the library, which is then shuffled (CR 701.24a).
    ShuffledIn,
    /// This many cards down from the top (0 is the top): "second from the top" is `FromTop(1)`. The cards given go in
    /// order, the first highest.
    FromTop(usize),
}

/// The layers and sublayers a static ability's effect applies in (CR 613.1, 613.4), in order. Copy effects (layer 1)
/// are the object's copiable values; control (2) is the object's controller itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Layer {
    /// Layer 3: text-changing effects (CR 612), on the type line here; card code reads its rules text's words through
    /// `Query::words`.
    Text,
    /// Layer 4: card types, subtypes, supertypes.
    Type,
    /// Layer 5: colors.
    Color,
    /// Layer 6: abilities, keywords, protection, attack requirements.
    Ability,
    /// Layer 7a: characteristic-defining abilities that define power and toughness (CR 604.3).
    PtDefine,
    /// Layer 7b: power and toughness set to a value.
    PtSet,
    /// Layer 7c: power and toughness modified, counters included.
    PtModify,
    /// Layer 7d: power and toughness switched (CR 613.4d), after everything that sets or modifies them.
    PtSwitch,
    /// Past every layer: the finished characteristics.
    End,
}

/// What a static ability's functions may read while the layers are being applied: the state, and other objects'
/// characteristics as the layers before the one being applied leave them (CR 613.8 is not modelled; reading only
/// earlier layers is what keeps it finite).
pub trait LayerQuery {
    fn state(&self) -> &GameState;
    fn cards(&self) -> &CardTable;
    /// Another object's characteristics, through the layers before the one being applied.
    fn chars(&self, id: ObjectId) -> Chars;
    /// The words an object's text has now (CR 612): what a static ability names, read through its source's words.
    fn words(&self, id: ObjectId) -> Words {
        crate::words(self.state(), id)
    }
}

/// Whether a static ability applies to `affected`, given what the earlier layers made it (`so_far`).
pub type StaticAffects = fn(&dyn LayerQuery, ObjectId, ObjectId, &Chars) -> bool;

/// A static ability's effect on one affected object's characteristics.
pub type StaticApply = fn(&dyn LayerQuery, ObjectId, &mut Chars);

/// A static ability (CR 604) that generates a continuous effect on characteristics (CR 611.3, 613): "lands are
/// Swamps", "all cards are blue", "other Elves get +1/+1". It applies while its source is on the battlefield, to
/// every object `affects` accepts: permanents, or objects anywhere with `everywhere` (CR 611.3a). A
/// characteristic-defining ability (`own`, CR 604.3) applies only to its own object, and in every zone.
#[derive(Clone, Copy)]
pub struct StaticDef {
    pub text: &'static str,
    pub layer: Layer,
    pub affects: StaticAffects,
    pub apply: StaticApply,
    pub everywhere: bool,
    pub own: bool,
    /// The zone its source functions from (CR 113.6b): the battlefield, the graveyard for Anger's kind, or exile for
    /// cipher's (CR 702.99a).
    pub from: ZoneKind,
    /// For a characteristic-defining ability of power and toughness printed as stars: what the stars are worth once the
    /// ability is lost (CR 208.2a, 208.5, 613.1f), as `*/1+*` is 0/1. The card's printed power and toughness stay unset.
    pub lost_pt: Option<(i16, i16)>,
}

fn affects_nothing(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, _: &Chars) -> bool {
    false
}

fn apply_nothing(_: &dyn LayerQuery, _: ObjectId, _: &mut Chars) {}

impl StaticDef {
    pub const BLANK: StaticDef =
        StaticDef {
            text: "",
            layer: Layer::Ability,
            affects: affects_nothing,
            apply: apply_nothing,
            everywhere: false,
            own: false,
            from: ZoneKind::Battlefield,
            lost_pt: None,
        };
}

/// What a cost is being worked out for (CR 601.2f): a spell about to be cast, or an ability about to be activated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Costed {
    /// The card that is being cast.
    Spell(ObjectId),
    /// The permanent (or card) whose ability is being activated, and which of its abilities.
    Ability(ObjectId, AbilityRef),
}

/// A static ability that changes what something costs (CR 601.2f, 118.7): the change in generic mana for `costed`
/// when `player` pays, by the permanent `source` that has it; negative reduces ("cost {1} less"), positive
/// increases. Zero when it does not apply. Reductions cannot go below zero generic mana (CR 118.7c).
pub type CostChange = fn(&dyn Query, ObjectId, PlayerId, Costed) -> i8;

/// A static ability that makes abilities cost less to activate, but never below one mana (CR 601.2f, 118.7): how much
/// generic mana less the ability `ability` of `permanent` costs `player` to activate, by the permanent `source` that has
/// it; zero when it does not apply. "Activated abilities of artifacts you control cost {1} less to activate. This effect
/// can't reduce the mana in that cost to less than one mana": Forensic Gadgeteer, Zirda, the Dawnwaker. Mana abilities
/// are not asked (they use no stack, and are paid for as the cost they are part of is).
pub type AbilityDiscount = fn(&dyn Query, ObjectId, PlayerId, ObjectId, AbilityRef) -> u8;

/// A static ability that lets a player pay a different mana cost rather than an activated ability's (CR 118.9, 602.2b),
/// by the permanent `source` that has it: the cost they may pay instead for `activator` activating the ability `def` of
/// `permanent`, or none. "You may pay {0} rather than pay the equip cost of the first equip ability you activate during
/// each of your turns": Forge Anew. Offered as a "you may" as the ability is announced.
pub type AbilityAlternative = fn(&dyn Query, &Asker, PlayerId, ObjectId, &ActivatedDef) -> Option<ManaCost>;

/// A mana ability's effect: it resolves at once, never on the stack (CR 605.3). The color is the one chosen for an
/// ability that adds mana of any one color.
pub type ManaEffect = fn(&mut dyn Game, &Asker, Option<Color>) -> Result<(), Suspend>;


/// Whether an ability may be activated or a way of casting taken at all, beyond its cost and timing.
pub type Condition = fn(&dyn Query, &Asker) -> bool;

/// Whether a permanent may be sacrificed to pay a cost.
pub type ObjectFilter = fn(&dyn Query, &Asker, ObjectId) -> bool;

/// Who is doing something with what: the source and the player controlling it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Asker {
    pub source: ObjectId,
    pub controller: PlayerId,
}

/// One target slot (CR 115.1).
#[derive(Clone, Copy)]
pub struct TargetSpec {
    pub min: u8,
    pub max: u8,
    pub filter: TargetFilter,
    /// What the slot is for, said to the player choosing it: "Spell or permanent to return to its owner's hand". Empty when the card's one slot says
    /// it all. "{player}" stands for the opponent whose slot it is (`EACH_OPPONENT`).
    pub label: &'static str,
}

impl TargetSpec {
    /// The same slot with `label` as what it is for.
    pub const fn labelled(self, label: &'static str) -> TargetSpec {
        TargetSpec { label, ..self }
    }

    /// A bound of `COUNTED` is the X announced for the spell or ability, or else the number of times it was kicked:
    /// "X target lands" is `min` and `max` both `COUNTED`, "up to X targets" is `min` 0 and `max` `COUNTED`.
    pub const COUNTED: u8 = u8::MAX;

    /// A `max` of this is "for each opponent, up to one target ...": the slot stands for one slot of its own for each
    /// opponent of the controller (CR 601.2c, 115.1), at most one target each, in seat order. It is not valid inside a mode (a modal slot reads a `max` above 1 as a range). The slot for an opponent
    /// is asked about `Query::slot_opponent`, and an opponent who has left the game keeps their slot, with nothing to
    /// choose, so the others keep their places (CR 800.4a). Its `min` is 0.
    pub const EACH_OPPONENT: u8 = u8::MAX - 2;

    /// Whether a legal target must exist to announce at all (CR 601.2c): a counted slot may be of zero.
    pub const fn required(&self) -> bool {
        self.min > 0 && self.min != TargetSpec::COUNTED
    }

    /// The slot's smallest and largest number of targets, with `counted` read for a bound that is `COUNTED`.
    pub fn bounds(&self, counted: u16) -> (u8, u8) {
        let of = |v: u8| if v == TargetSpec::COUNTED { counted.min(254) as u8 } else { v };
        (of(self.min), of(self.max))
    }
}

/// A cost (CR 118).
#[derive(Clone, Copy)]
pub struct Cost {
    pub mana: ManaCost,
    /// {T} (CR 107.5).
    pub tap: bool,
    pub sacrifice_self: bool,
    /// Discard this card (cycling, CR 702.29a).
    pub discard_self: bool,
    /// Choose this many permanents you control that pass the filter, and sacrifice them or return them to their
    /// owners' hands (CR 601.2h): chosen as the cost is chosen, moved as it is paid.
    pub permanents: Option<PermanentCost>,
    /// A loyalty cost (CR 606.4): positive adds, negative removes.
    pub loyalty: Option<i8>,
    /// Pay this much life (CR 119.4): payable only with at least that much.
    pub life: u8,
    /// Remove this many counters of this kind from the source (CR 602.1, 118.3).
    pub remove_counters: Option<(CounterKind, u8)>,
    /// Remove a counter of any kind from the source (CR 118.3): the payer names the kind when it has more than one.
    pub remove_any_counter: bool,
    /// Exile this card (from the zone the ability functions in: eternalize, CR 702.129a).
    pub exile_self: bool,
    /// Discard this many cards from your hand that pass the filter (CR 701.9), chosen as the cost is chosen.
    pub discard: Option<(u8, ObjectFilter)>,
    /// Pay X life, X announced with the spell (CR 118.8, 601.2b): Toxic Deluge, Vicious Rivalry.
    pub pay_x_life: bool,
    /// Exile this many cards from your graveyard, other than the source (CR 702.138a escape, and "exile three cards
    /// from your graveyard"), chosen as the cost is chosen (CR 601.2h).
    pub exile_from_graveyard: u8,
    /// Exert this permanent (CR 701.43a): it won't untap during its controller's next untap step.
    pub exert: bool,
    /// Tap any number of untapped creatures you control, other than the source, with total power N or more
    /// (CR 702.122a crew, CR 702.171a saddle).
    pub tap_creatures: Option<TapCreatures>,
    /// Emerge (CR 702.119a): the mana in this cost comes down by the mana value of the permanents it sacrifices
    /// (`permanents`), a cost reduction that takes generic mana only (CR 118.7).
    pub emerge: bool,
    /// Exile this many cards from your hand that pass the filter, other than the source (CR 118.9: "exile a red card from
    /// your hand"), chosen as the cost is chosen.
    pub exile_from_hand: Option<(u8, ObjectFilter)>,
    /// The `permanents` are tapped instead of sacrificed or returned (CR 118.3, 701.26): "Tap an untapped creature you
    /// control", chosen as the cost is chosen and tapped as it is paid. It is not the {T} symbol (CR 107.5), so a
    /// summoning-sick creature, or the source itself, may be tapped for it.
    pub tap_permanents: bool,

    /// Unattach this Equipment (CR 701.3d, 118.3): payable only while it is attached to something, since a cost that
    /// changes nothing cannot be paid.
    pub unattach_self: bool,
    /// Return this permanent to its owner's hand (CR 118.3): payable only while it is on the battlefield.
    pub return_self: bool,
    /// A part of the cost whose size the payer decides (CR 601.2b, 601.2h).
    pub variable: Option<VariableCost>,
    /// Discard your hand (CR 701.9): every card in it, with no choice to make and payable with an empty hand.
    pub discard_hand: bool,
    /// Collect evidence N (CR 701.59a): exile cards with total mana value N or more from your graveyard, other than the
    /// source, chosen as the cost is chosen.
    pub collect_evidence: u8,
    /// Behold N (CR 701.4a): for each, choose a permanent you control or reveal a card from your hand that passes the
    /// filter ("behold a Dragon"), chosen as the cost is chosen. Nothing moves: a permanent is only chosen, a hand card
    /// is revealed as the cost is paid (CR 601.2h). Payable only with that many to choose among. Dispelling Exhale,
    /// Caustic Exhale, Kinsbaile Aspirant. Paid when a spell is cast only: an activation does not choose or pay it yet.
    /// The quality's name ("Dragon") is what the prompt says.
    pub behold: Option<(u8, ObjectFilter, &'static str)>,
    /// Blight N (CR 701.68a): put this many -1/-1 counters on a creature you control, chosen as the cost is chosen and
    /// put as it is paid. Payable only with a creature to put them on (CR 701.68b). Requiting Hex, Cinder Strike,
    /// Paid when a spell is cast only: an activation does not choose or pay it yet.
    pub blight: u8,
    /// Harmonize's additional part (CR 702.180a-b): tap up to one untapped creature you control, and the total cost comes down
    /// by generic mana equal to its power (never by colored mana, CR 601.2f). The creature is chosen as the cost is chosen
    /// (CR 601.2b), its power read as the total cost is determined (CR 601.2f), and it is tapped as the cost is paid. It is not
    /// a {T} ability's cost (CR 107.5), so a summoning-sick creature may be tapped. Only a spell's way of casting carries it.
    pub tap_to_reduce: bool,
}

/// A cost part whose size is the payer's to set, chosen as the cost is chosen.
#[derive(Clone, Copy)]
pub enum VariableCost {
    /// Sacrifice X permanents you control that pass the filter, X announced with the spell though its mana cost has no
    /// {X} (CR 601.2b, 107.3a): Immoral Bargain.
    SacrificeX(ObjectFilter),
    /// Sacrifice one or more permanents you control that pass the filter (CR 701.21a): as an optional additional cost, the
    /// spell's `paid` is how many were. Plumb the Forbidden.
    SacrificeOneOrMore(ObjectFilter),
    /// Exile any number of other cards from your graveyard with this many card types or more among them (CR 205.2a):
    /// Nethergoyf's escape.
    ExileFromGraveyardWithTypes(u8),
}

/// The cost of crew (CR 702.122a) and saddle (CR 702.171a): tap other untapped creatures with total power `power` or
/// more. Summoning sickness does not matter, since it is not their own {T} (CR 302.6).
#[derive(Clone, Copy)]
pub struct TapCreatures {
    pub power: u8,
    /// Crew's, where a creature that "crews Vehicles as though its power were 2 greater" counts for more
    /// (CR 702.122b). Saddle's does not.
    pub crew: bool,
}

#[derive(Clone, Copy)]
pub struct PermanentCost {
    pub count: u8,
    pub filter: ObjectFilter,
    pub to_hand: bool,
}

impl Cost {
    /// This cost paid `n` times over (escalate, CR 702.120a, once for each mode beyond the first): its mana, life,
    /// discards, permanents and graveyard exiles each multiplied. The parts that can only be paid once are left as
    /// they are.
    pub fn times(self, n: u8) -> Cost {
        let mut c = self;
        for _ in 1..n {
            c.mana.generic += self.mana.generic;
            for i in 0..6 {
                c.mana.symbols[i] += self.mana.symbols[i];
            }
        }
        if n == 0 {
            c.mana = ManaCost::ZERO;
        }
        c.life = self.life.saturating_mul(n);
        c.exile_from_graveyard = self.exile_from_graveyard.saturating_mul(n);
        c.blight = self.blight.saturating_mul(n);
        c.exile_from_hand = self.exile_from_hand.map(|(k, f)| (k.saturating_mul(n), f)).filter(|_| n > 0);
        c.discard = self.discard.map(|(k, f)| (k.saturating_mul(n), f)).filter(|_| n > 0);
        c.permanents = self.permanents.map(|p| PermanentCost { count: p.count.saturating_mul(n), ..p }).filter(|_| n > 0);
        c
    }

    /// This cost and another paid together: the parts of the same kind add (an additional cost's discard and an
    /// escalate cost's discard are discards of one choice). A card writes "{1}, {T}" as `cost("{1}").and(TAP)`.
    pub const fn and(self, other: Cost) -> Cost {
        let mut c = self;
        c.mana.generic += other.mana.generic;
        let mut i = 0;
        while i < 6 {
            c.mana.symbols[i] += other.mana.symbols[i];
            i += 1;
        }
        let mut i = 0;
        while i < 6 {
            if c.mana.hybrid[i].is_none() {
                c.mana.hybrid[i] = other.mana.hybrid[i];
            }
            i += 1;
        }
        let mut i = 0;
        while i < 5 {
            c.mana.phyrexian[i] += other.mana.phyrexian[i];
            i += 1;
        }
        c.mana.x += other.mana.x;
        c.tap |= other.tap;
        c.sacrifice_self |= other.sacrifice_self;
        c.discard_self |= other.discard_self;
        c.exile_self |= other.exile_self;
        c.pay_x_life |= other.pay_x_life;
        c.emerge |= other.emerge;
        c.tap_to_reduce |= other.tap_to_reduce;
        c.tap_permanents |= other.tap_permanents;
        c.remove_any_counter |= other.remove_any_counter;
        if c.loyalty.is_none() {
            c.loyalty = other.loyalty;
        }
        if c.remove_counters.is_none() {
            c.remove_counters = other.remove_counters;
        }
        c.life += other.life;
        c.exile_from_graveyard += other.exile_from_graveyard;
        c.exile_from_hand = match (self.exile_from_hand, other.exile_from_hand) {
            (Some((a, f)), Some((b, _))) => Some((a + b, f)),
            (None, b) => b,
            (a, None) => a,
        };
        c.discard = match (self.discard, other.discard) {
            (Some((a, f)), Some((b, _))) => Some((a + b, f)),
            (None, b) => b,
            (a, None) => a,
        };
        c.permanents = match (self.permanents, other.permanents) {
            (Some(a), Some(b)) => Some(PermanentCost { count: a.count + b.count, ..a }),
            (None, b) => b,
            (a, None) => a,
        };
        if c.tap_creatures.is_none() {
            c.tap_creatures = other.tap_creatures;
        }
        c.exert |= other.exert;
        c.unattach_self |= other.unattach_self;
        c.return_self |= other.return_self;
        if c.variable.is_none() {
            c.variable = other.variable;
        }
        c.discard_hand |= other.discard_hand;
        c.collect_evidence += other.collect_evidence;
        // Two beholds add only when they ask for the same quality (CR 701.4a): one filter can't stand for two.
        c.behold = match (self.behold, other.behold) {
            (Some((a, f, name)), Some((b, _, other_name))) => {
                assert!(same_str(name, other_name), "two beholds of different qualities can't be combined");
                Some((a + b, f, name))
            }
            (None, b) => b,
            (a, None) => a,
        };
        c.blight += other.blight;
        c
    }

    pub const NONE: Cost =
        Cost {
            mana: ManaCost::ZERO,
            tap: false,
            sacrifice_self: false,
            discard_self: false,
            permanents: None,
            loyalty: None,
            life: 0,
            remove_counters: None,
            remove_any_counter: false,
            exile_self: false,
            discard: None,
            pay_x_life: false,
            exile_from_graveyard: 0,
            exert: false,
            tap_creatures: None,
            emerge: false,
            exile_from_hand: None,
            tap_permanents: false,

            unattach_self: false,
            return_self: false,
            variable: None,
            discard_hand: false,
            collect_evidence: 0,
            behold: None,
            blight: 0,
            tap_to_reduce: false,
        };

    pub const fn sacrifice(count: u8, filter: ObjectFilter) -> Cost {
        Cost { permanents: Some(PermanentCost { count, filter, to_hand: false }), ..Cost::NONE }
    }

    /// Tap `count` untapped permanents you control that pass the filter, as a cost (CR 118.3).
    pub const fn tap_untapped(count: u8, filter: ObjectFilter) -> Cost {
        Cost { permanents: Some(PermanentCost { count, filter, to_hand: false }), tap_permanents: true, ..Cost::NONE }
    }

    pub const fn return_to_hand(count: u8, filter: ObjectFilter) -> Cost {
        Cost { permanents: Some(PermanentCost { count, filter, to_hand: true }), ..Cost::NONE }
    }

    pub const fn mana(mana: ManaCost) -> Cost {
        Cost { mana, ..Cost::NONE }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Timing {
    Instant,
    /// Main phase, own turn, empty stack (CR 307.1).
    Sorcery,
    /// Any time its controller has priority during their own turn: foretell (CR 702.143a).
    OwnTurn,
    /// Whenever its card could be cast from the hand: at instant speed for an instant or a card with flash, at sorcery
    /// speed for any other (CR 702.62a suspend).
    AsCast,
}

/// Suspend (CR 702.62a): the card may be exiled from the hand with `count` time counters for `cost`, and is cast for
/// free when the last is removed at the beginning of its owner's upkeeps.
#[derive(Clone, Copy)]
pub struct SuspendDef {
    pub count: u8,
    pub cost: ManaCost,
}

/// Whether two texts are the same, as the compiler can ask it.
pub(crate) const fn same_str(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// An activated ability (CR 602). A card writes one with `ability` or `mana_ability` (`crate::ability`).
#[derive(Clone, Copy)]
pub struct ActivatedDef {
    /// The ability's text, which a scenario's `Ability:` names it by.
    pub text: &'static str,
    pub cost: Cost,
    pub timing: Timing,
    pub targets: &'static [TargetSpec],
    pub effect: Effect,
    /// A mana ability's own effect (CR 605.1a), where it does more than add the mana `adds` and `adds_any` list: it
    /// resolves immediately, and `effect` is unused. One with {X} in its cost has X announced first (CR 107.3a, 605.3a);
    /// the effect reads it as `Obj::x` of its source. A mana ability with none adds what it lists
    /// (`is_mana_ability`, `add_listed`).
    pub mana: Option<ManaEffect>,
    /// A mana ability that adds mana of any one color: the activation names the color (CR 106.5).
    pub any_color: bool,
    /// The colors an `any_color` ability may name ("Add {B}, {R}, or {G}"); none means all five.
    pub colors: ColorSet,
    /// The mana types an `any_color` ability may name are those the permanent sacrificed to activate it could produce
    /// (CR 106.7): "add one mana of any type that land could produce". Named by colorless as `Action::mana` `Some(C)`; an
    /// ability that adds nothing when the permanent could produce nothing (CR 106.5).
    pub colors_from_sacrifice: bool,
    /// The colors an `any_color` ability may name, worked out from the game when it is offered ("any of the exiled
    /// card's colors": a linked ability, CR 607.2a). With no colors there is none to choose: the ability is still
    /// offered, and adds no mana (CR 106.5). Overrides `colors`.
    pub colors_from: Option<fn(&dyn Query, &Asker) -> ColorSet>,
    /// The zone it functions from: the battlefield, or the hand for cycling and the like (CR 113.6).
    pub from: ZoneKind,
    /// What a mana ability adds, for whoever wants to know before activating it: fixed kinds, and how many of one
    /// chosen color.
    pub adds: ManaPool,
    pub adds_any: u8,
    pub condition: Option<Condition>,
    /// "Activate only once each turn" (CR 602.5b-c): counted for this object and for each instance of the ability on
    /// its own, whoever activated it.
    pub once_each_turn: bool,
    /// Exhaust (CR 702.177a): "Activate each exhaust ability only once" by this permanent. Asked of the event history,
    /// where every activation of an ability, a mana ability's included, is recorded.
    pub exhaust: bool,
    /// Its modes, chosen on activation before its costs (CR 602.2b, 700.2); `effect` and `targets` are then unused.
    pub modal: Option<&'static ModalDef>,
    /// For a mana ability: its mana can only be spent as this restriction allows (CR 106.6), and goes to the pool
    /// apart from unrestricted mana.
    pub restricted: Option<ManaRestriction>,
    /// Its mana cost is a waterbend cost (CR 701.67a): each generic mana of it may be paid by tapping an untapped
    /// artifact or creature its controller has, instead.
    pub waterbend: bool,
    /// "For each mana in this ability's activation cost, you may tap an untapped creature you control rather than pay that
    /// mana" (CR 118.3, 601.2h): each generic mana of it may be paid by tapping an untapped creature (Heirloom Epic).
    pub tap_creatures_for_mana: bool,
    /// A special action (CR 116.2): taken with priority, paid for and done at once, using no stack and with no chance to
    /// respond. Plot (CR 702.170a) is one. It is not an activation (CR 602): nothing triggers on it.
    pub special_action: bool,
    /// An equip ability (CR 702.6a): "equip abilities" are named as a class by cards that change how they work.
    pub equip: bool,
}

fn no_effect(_: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    Ok(())
}

fn no_match(_: &dyn Query, _: ObjectId, _: &Event) -> Option<Fired> {
    None
}

impl ActivatedDef {
    /// A mana ability (CR 605.1a): one with an effect of its own that adds mana, or one that lists the mana it adds.
    pub const fn is_mana_ability(&self) -> bool {
        if self.mana.is_some() || self.adds_any > 0 {
            return true;
        }
        let mut i = 0;
        while i < 6 {
            if self.adds.0[i] > 0 {
                return true;
            }
            i += 1;
        }
        false
    }

    pub const BLANK: ActivatedDef = ActivatedDef {
        text: "",
        cost: Cost::NONE,
        timing: Timing::Instant,
        targets: &[],
        effect: no_effect,
        mana: None,
        any_color: false,
        colors: ColorSet::NONE,
        colors_from: None,
        colors_from_sacrifice: false,
        from: ZoneKind::Battlefield,
        adds: ManaPool([0; 6]),
        adds_any: 0,
        condition: None,
        once_each_turn: false,
        exhaust: false,
        restricted: None,
        waterbend: false,
        tap_creatures_for_mana: false,
        equip: false,
        special_action: false,
        modal: None,
    };
}

impl TriggeredDef {
    pub const BLANK: TriggeredDef = TriggeredDef {
        text: "",
        listens: EventMask(0),
        matches: no_match,
        targets: &[],
        effect: no_effect,
        look_back: false,
        anywhere: false,
        condition: None,
        once_per_batch: false,
        batch_by_data: false,
        mana_ability: false,
        sums_per_subject: false,
        triggers_each_turn: 0,
        chapter: 0,
        modal: None,
        divide: None,
        subject_chooses_targets: false,
        ward: false,
    };
}

/// A triggered ability (CR 603).
#[derive(Clone, Copy)]
pub struct TriggeredDef {
    pub text: &'static str,
    pub listens: EventMask,
    pub matches: TriggerMatch,
    pub targets: &'static [TargetSpec],
    pub effect: Effect,
    /// A leaves-the-battlefield ability (CR 603.10a): judged on the board as it was before the event, so it sees
    /// its own source and everything that left with it.
    pub look_back: bool,
    /// Functions outside the battlefield too (the ability checks the zone itself), as "at the beginning of your
    /// upkeep, if this card is in your graveyard" does.
    pub anywhere: bool,
    pub condition: Option<TriggerCondition>,
    /// A "one or more" ability (CR 603.2c): the events of one simultaneous batch that it matches trigger it once.
    /// `matches` is asked of each event and the subject objects of those that match are the batch's members, which
    /// the resolving ability reads with `Query::event_objects`.
    pub once_per_batch: bool,
    /// With `once_per_batch`: events of the batch whose `data[1]` differ are different batches, each triggering it once (CR 603.2c):
    /// "whenever one or more cards are put into a library" is one for each library (Wan Shi Tong, All-Knowing).
    pub batch_by_data: bool,
    /// A triggered mana ability (CR 605.1b): it triggers on the tapping of a permanent for mana (`Event::TappedForMana`) and resolves at
    /// once, not on the stack, as part of the mana ability that triggered it (CR 605.4a). `matches` is asked of every permanent that has
    /// one; `effect` adds the mana.
    pub mana_ability: bool,
    /// Events of one batch that are about the same subject trigger it once, with their `data[0]` added up (CR 120.2,
    /// 603.2c): damage dealt to one creature at once is one event, however many sources dealt it. "Whenever a creature
    /// you control is dealt damage ... that much damage" (The Sensational She-Hulk, Screaming Nemesis).
    pub sums_per_subject: bool,
    /// "This ability triggers only N times each turn" (CR 603.2h): 0 for no limit. A triggering that would exceed it
    /// does not happen, and one that happens uses one up.
    pub triggers_each_turn: u8,
    /// A Saga's chapter ability (CR 714.2): it triggers when lore counters are put on its source and their number
    /// becomes at least this, having been less. `listens` and `matches` are not used; the core owns the trigger.
    pub chapter: u8,
    /// A modal ability's modes (CR 700.2): "choose one". Chosen, with each chosen mode's targets, as the ability is put on
    /// the stack (CR 603.3c); `effect` and `targets` are then unused.
    pub modal: Option<&'static ModalDef>,
    /// Its first target slot divides a total among its targets (CR 601.2d).
    pub divide: Option<Divide>,
    /// "That player chooses target ..." (CR 603.3d): the player the event was about, not the ability's controller, chooses its
    /// targets (Oath of Druids and Oath of Mages at each player's upkeep). Targeting is otherwise the controller's: legality, hexproof
    /// and the ability itself stay theirs.
    pub subject_chooses_targets: bool,
    /// A ward ability (CR 702.21a): a targeting rule can stop it from triggering (`TargetingRuleKind::SuppressWard`).
    pub ward: bool,
}

#[derive(Clone, Copy)]
pub enum AbilityDef {
    Activated(ActivatedDef),
    Triggered(TriggeredDef),
    /// A static ability an effect gives an object (CR 613.1f, 613.7a): "[this creature] gains 'This creature's base power is
    /// equal to the number of cards in your hand.'" It is one of a card's `delayed` list, granted with `EffectKind::AddAbility`.
    /// It affects only the object that has it, with that object as its source (its "you" is the object's controller), in a
    /// layer after the abilities are settled (`Layer::PtDefine` on): see `StaticDef`.
    Static(StaticDef),
}

impl AbilityDef {
    pub fn text(&self) -> &'static str {
        match self {
            AbilityDef::Activated(a) => a.text,
            AbilityDef::Triggered(t) => t.text,
            AbilityDef::Static(s) => s.text,
        }
    }
}

/// Which keyword a way is, where a card asks how another was cast ("if that creature was cast for its warp cost"): the
/// way's identity, set by the keyword's own constructor. Other keywords' ways are `Other` until a card asks about them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WayKind {
    Other,
    /// Warp (CR 702.185a).
    Warp,
}

/// An alternative way to cast a card (CR 118.9): flashback (CR 702.34) from the graveyard, Fireblast's sacrifice.
#[derive(Clone, Copy)]
pub struct WayDef {
    pub text: &'static str,
    pub cost: Cost,
    /// The zone it is cast from this way.
    pub from: ZoneKind,
    /// Exiled instead of going anywhere else as it leaves the stack (flashback, CR 702.34a).
    pub exile_after: bool,
    pub condition: Option<Condition>,
    /// It may be cast this way whenever its caster could cast an instant, whatever the card's own type (sneak, CR 702.190a:
    /// "any time you could cast an instant during your declare blockers step"). `condition` says when that is.
    pub instant_speed: bool,
    /// Which keyword's way this is.
    pub kind: WayKind,
}

#[derive(Clone, Copy)]
pub struct Kicker {
    pub cost: ManaCost,
    /// Payable any number of times.
    pub multi: bool,
    /// A waterbend cost (CR 701.67a-b): as it is paid, an untapped artifact or creature may be tapped for each generic mana of
    /// it, and for no other mana of the spell's total cost. Spirit Water Revival's "you may waterbend {6}".
    pub waterbend: bool,
}

/// One way of paying an additional cost that names a choice ("discard a card or pay 3 life", CR 601.2b): its text,
/// which the question offers it by, and what paying it costs.
#[derive(Clone, Copy)]
pub struct ExtraCost {
    pub text: &'static str,
    pub cost: Cost,
}

/// A spell or ability that divides a total among its targets as they are chosen (CR 601.2d): its first target slot is
/// asked for together with each target's share, in one question. The slot's `max` is how many targets it may have.
#[derive(Clone, Copy)]
pub struct Divide {
    /// The total to divide, or `TargetSpec::COUNTED` for the X announced.
    pub total: u8,
    /// The total as a function of the X announced, where it is not `total` itself: "if X is 6 or more, twice X damage
    /// divided instead" (Shatterskull Smashing).
    pub of_x: Option<fn(u16) -> u16>,
}

/// An optional additional cost that is not mana (CR 601.2b): teamwork's "tap any number of creatures you control with
/// total power N or more". Announced once, like a kicker, by choosing between paying it and not, before any target;
/// paid with the other costs. A spell that paid it has `paid` 1 (how many, for a cost that sacrifices one or more),
/// which is not `kicked`: a card with a kicker and an optional cost of this kind is not ambiguous.
#[derive(Clone, Copy)]
pub struct OptionalCost {
    pub cost: Cost,
}

/// One mode of a modal spell or ability (CR 700.2). Its target slots are each one target (CR 700.2c); `cost` is what
/// choosing it adds to the total cost, for spree (CR 702.172a).
#[derive(Clone, Copy)]
pub struct ModeDef {
    pub text: &'static str,
    pub targets: &'static [TargetSpec],
    pub effect: Effect,
    pub cost: ManaCost,
}

/// Modes chosen as a spell is cast or an ability activated (CR 601.2b, 602.2b, 700.2): between `min` and `max` of
/// them, each at most once unless `repeat`, run in printed order as it resolves (CR 700.2d). Each chosen mode beyond the
/// first costs `escalate` more (CR 702.120a): mana, a discard, life.
#[derive(Clone, Copy)]
pub struct ModalDef {
    pub modes: &'static [ModeDef],
    pub min: u8,
    pub max: u8,
    pub escalate: Option<Cost>,
    /// "You may choose the same mode more than once" (CR 700.2d): Mystic Confluence, Fiery Confluence. Each choice is
    /// an instance of its own, with targets of its own, run in sequence.
    pub repeat: bool,
    /// Whether mode `i` may be chosen now, beyond its targets (CR 700.2): "choose one that hasn't been chosen" (Gollum,
    /// Riddle Master) reads the modes this object chose before from the event history (`Event::ModesChosen`). A
    /// triggered ability left with too few modes it may choose is removed as it would be put on the stack (CR 603.3c).
    pub allowed: Option<fn(&dyn Query, &Asker, u8) -> bool>,
    /// How many modes a spell chooses, `(min, max)`, instead of `min` to `max` when its optional additional cost is paid or it
    /// is kicked: "Choose one. If this spell's additional cost was paid, choose both instead" (Pyrrhic Strike), "If this
    /// spell was kicked, choose any number instead" (Inscription of Abundance, Insight and Ruin). The intention to pay is
    /// announced in the same step as the modes (CR 601.2b), and the number of modes is defined by it, so the cost is
    /// announced first and the modes after it; paying is offered only when that many modes could be chosen (CR 700.2a).
    /// A spell's own: a modal triggered ability (Depth Defiler's) does not read it.
    pub when_paid: Option<(u8, u8)>,
}

impl ModalDef {
    /// How many times mode `i` is chosen in a `StackItem::modes` mask. A mask holds a bit for each mode, or, for a
    /// modal that repeats (at most four modes, each chosen up to three times), two bits for each: its count.
    pub fn times(&self, mask: u8, i: usize) -> u8 {
        if self.repeat { (mask >> (2 * i)) & 3 } else { (mask >> i) & 1 }
    }

    /// The mask of mode `i` chosen once.
    pub fn once(&self, i: usize) -> u8 {
        if self.repeat { 1 << (2 * i) } else { 1 << i }
    }

    /// The mask of one more choice of mode `i` on top of `mask`.
    pub fn with(&self, mask: u8, i: usize) -> u8 {
        if self.repeat { mask + (1 << (2 * i)) } else { mask | (1 << i) }
    }

    /// How many modes a mask chooses, repeats included.
    pub fn chosen(&self, mask: u8) -> u8 {
        (0..self.modes.len()).map(|i| self.times(mask, i)).sum()
    }
}

impl SpellDef {
    /// The spell ability of a modal spell: it has no effect of its own (`CardDef::modal` holds the modes).
    pub const MODAL: SpellDef = SpellDef { targets: &[], effect: no_effect };
}

/// A spell's own ability (CR 113.3a): what an instant or sorcery does.
#[derive(Clone, Copy)]
pub struct SpellDef {
    pub targets: &'static [TargetSpec],
    pub effect: Effect,
}

/// The most modes a modal spell may choose, worked out from the board as the spell is cast (CR 601.2b, 700.2): "choose one.
/// If you control a Wizard as you cast this spell, you may choose two instead". Asked once, as the modes are announced.
pub type ModeCount = fn(&dyn Query, &Asker) -> u8;

/// The cost a card in a graveyard may be cast for by a static ability's say-so (CR 113.6, 118.9), or none when it may not
/// be: asked of the granting permanent (`Asker`), the card in the graveyard and the player casting it. Underworld Breach's
/// escape is the card's mana cost and exiling three other cards; mayhem is its mana cost for a card discarded this turn.
pub type GrantedCost = fn(&dyn Query, &Asker, ObjectId, PlayerId) -> Option<Cost>;

/// The ability index of the marker the history keeps of a granted way having been used: an activation, by the permanent that
/// gave the way, of this index (past any ability a card prints). "Once during each of your turns" reads it (CR 601.3).
pub const GRANTED_WAY_USED: u8 = AbilityRef::GRANTED_WAY_USED;

/// A way of casting cards from graveyards that a permanent gives (CR 604, 118.9).
#[derive(Clone, Copy)]
pub struct GrantedWay {
    /// What the way is called, which a scenario's `Ability:` names it by.
    pub text: &'static str,
    pub cost: GrantedCost,
    /// The way also lets a land card in the graveyard be played (CR 305.1) rather than cast: `cost` is asked of the land
    /// card too, and any `Some` allows the play (the land has no cost). It uses the land drop, and the history marks the
    /// use as it marks a cast's, so "once during each of your turns" is shared between the play and the casts. Serra Paragon.
    pub lands: bool,
    /// "If you do, it gains [ability]": the index in the granting card's `delayed` abilities of the ability the permanent
    /// resulting from the play or the cast has from then on (CR 400.7a, 113.6). Serra Paragon's exile-and-gain-2-life.
    pub gains: Option<u8>,
}

impl GrantedWay {
    /// A way that casts only and grants nothing, for a card's file to start from.
    pub const CAST: GrantedWay = GrantedWay { text: "", cost: |_, _, _, _| None, lands: false, gains: None };
}

/// A static ability that says whether a player may cast spells right now (CR 101.2, 601.3): false forbids it, and
/// forbids it for every spell, for a cast an effect offers during resolution too.
pub type CastRule = fn(&dyn Query, &Asker, PlayerId) -> bool;

/// A static ability that limits who may block whom (CR 509.1b): asked of the object that has it, with the blocker and the
/// attacker of a possible block, either of which may be the object itself; false forbids the block. "This creature can't
/// block or be blocked by non-Spirit creatures" (Wan Shi Tong's Spirits).
pub type BlockRule = fn(&dyn Query, ObjectId, ObjectId, ObjectId) -> bool;

/// What a [`TargetingRule`] changes about targeting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetingRuleKind {
    /// The creatures it names "can be the targets of spells and abilities as though they didn't have hexproof" (CR 702.11b,
    /// 609.4): hexproof, granted before or after, does not stop targeting them. Shroud and protection still do.
    IgnoreHexproof,
    /// "Ward abilities of those creatures don't trigger" (CR 702.21a): a ward ability of the object it names does not trigger
    /// when it becomes a target.
    SuppressWard,
}

/// A static ability that changes targeting rules for the objects it names (CR 604, 609.4), while its source is on the
/// battlefield: `applies` is asked of the source and of an object that is or would be a target, and sees the layers as they
/// are before the abilities layer. "Creatures your opponents control can be the targets of spells and abilities as though they
/// didn't have hexproof. Ward abilities of those creatures don't trigger" (Nowhere to Run).
#[derive(Clone, Copy)]
pub struct TargetingRule {
    pub kind: TargetingRuleKind,
    pub applies: fn(&dyn LayerQuery, ObjectId, ObjectId) -> bool,
}

/// A replacement of a turn beginning (CR 614.10, 500.11): "if you would begin your turn while [condition], you may skip
/// that turn instead. If you do, [on_skip]". Time Vault. A skipped turn is not taken: its steps, its untap and its
/// number are not used, and the next player's turn begins in its place.
#[derive(Clone, Copy)]
pub struct TurnBeginRule {
    /// Whether it applies to `player`, whose turn would begin, judged on the board as it is.
    pub applies: fn(&dyn Query, &Asker, PlayerId) -> bool,
    /// What the effect does once its controller has chosen to skip the turn.
    pub on_skip: fn(&mut dyn Game, &Asker) -> Result<(), Suspend>,
}

/// A static ability that says whether a spell or ability may cause a player to discard cards or sacrifice permanents
/// (CR 101.2): true allows it. `causer` is the controller of the spell or ability, `player` who would discard or
/// sacrifice. "Spells and abilities your opponents control can't cause you to discard cards or sacrifice permanents."
pub type CauseRule = fn(&dyn Query, &Asker, PlayerId, PlayerId, bool) -> bool;

/// A static ability that says some damage can't be prevented (CR 615.12): true for the damage a source is about to deal,
/// `combat` or not. "Combat damage that would be dealt by creatures you control can't be prevented."
pub type UnpreventableRule = fn(&dyn Query, &Asker, ObjectId, bool) -> bool;

/// Whether a static ability makes this creature assign combat damage equal to its toughness rather than its power
/// (CR 510.1a): given the rule's own asker.
pub type DamageByToughnessRule = fn(&dyn Query, &Asker, ObjectId) -> bool;

/// Whether a static ability lets its controller spend mana as though it were mana of any color to activate the abilities
/// of this permanent (CR 609.4b): given the rule's own asker and the permanent whose ability is activated.
pub type SpendAnyColorRule = fn(&dyn Query, &Asker, ObjectId) -> bool;

/// A static ability that lets an ability that can only be activated at sorcery speed (a loyalty ability, CR 606.3) be
/// activated whenever its activator could cast an instant: asked of the ability's source, its index in the source's
/// computed list and the activator.
pub type InstantActivationRule = fn(&dyn Query, &Asker, ObjectId, u8, PlayerId) -> bool;

/// A static ability that lets its controller cast some spells as though they had flash (CR 307.5, 702.8): asked of the spell
/// (a card in the zone it is cast from) and the player casting it. Sigarda's Aid's Auras and Equipment.
pub type FlashRule = fn(&dyn Query, &Asker, ObjectId, PlayerId) -> bool;

/// A static ability that says whether an activated ability may be activated at all (CR 602.5, 605): asked of the permanent
/// that has the rule, the object whose ability it would be (in whatever zone it functions from) and that ability. False
/// forbids it, mana abilities included. "Activated abilities of artifacts, creatures, and planeswalkers can't be
/// activated" (Clarion Conqueror); Null Rod's and Stony Silence's "of artifacts" is the same rule over fewer types.
pub type ActivationRule = fn(&dyn Query, &Asker, ObjectId, &ActivatedDef) -> bool;

/// A permanent's standing rule about what its controller may play (CR 305.2, 305.1).
#[derive(Clone, Copy, Debug)]
pub enum PlayRule {
    /// "You may play an additional land on each of your turns" (CR 305.2): Icetill Explorer.
    ExtraLandPlays(u8),
    /// "You may play any number of lands on each of your turns" (CR 305.2): Fastbond.
    AnyNumberOfLandPlays,
    /// "You may play lands from your graveyard" (CR 305.1): Icetill Explorer.
    LandsFromGraveyard,
    /// "You may play lands and cast spells from the top of your library. If you cast a spell this way, pay life equal
    /// to its mana value rather than pay its mana cost" (CR 305.1, 601.3, 118.9): Bolas's Citadel.
    LibraryTop,
    /// "You may cast [spells that pass the filter] from the top of your library" (CR 601.3), for their mana cost: Mystic Forge's
    /// artifact and colorless spells. Lands are not played from there.
    LibraryTopSpells(fn(&dyn Query, ObjectId) -> bool),
    /// "You have no maximum hand size" (CR 402.2): Proft's Eidetic Memory.
    NoMaximumHandSize,
}

/// What else may pay a spell's total cost besides mana (CR 601.2f-h): convoke (CR 702.51a), delve (CR 702.66a) and
/// improvise (CR 702.126a). Characteristics of the card, generated from its Oracle text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct PayWith {
    pub convoke: bool,
    pub delve: bool,
    pub improvise: bool,
}

impl PayWith {
    pub const NONE: PayWith = PayWith { convoke: false, delve: false, improvise: false };

    pub const fn any(self) -> bool {
        self.convoke || self.delve || self.improvise
    }
}

/// What a card in an opening hand may do before the game begins (CR 103.6a): "begin the game with it on the battlefield". The
/// players decide in turn order, the starting player first, after mulligans.
#[derive(Clone, Copy)]
pub struct OpeningHandAction {
    /// Whether this player may take it: Gemstone Caverns's "if you're not the starting player".
    pub condition: Condition,
    /// The counter it enters with.
    pub counter: Option<(CounterKind, u8)>,
    /// "If you do, exile this many cards from your hand."
    pub exile_from_hand: u8,
}

/// A card: characteristics from the Oracle data, behavior as code.
#[derive(Clone, Copy)]
pub struct CardDef {
    pub name: &'static str,
    pub cost: ManaCost,
    /// Whether it has a mana cost at all (CR 118.6): a card without one cannot be cast for it.
    pub has_cost: bool,
    /// Its color indicator or defined colors, where they are not its cost's.
    pub colors: Option<ColorSet>,
    pub types: TypeSet,
    pub supertypes: Supertypes,
    pub subtypes: SubtypeSet,
    pub power: Option<i16>,
    pub toughness: Option<i16>,
    pub loyalty: Option<i16>,
    /// A battle's printed defense (CR 310.4a): it enters with that many defense counters (CR 310.4b).
    pub defense: Option<i16>,
    pub keywords: Keywords,
    /// Convoke, delve, improvise: what besides mana may pay its total cost.
    pub pay_with: PayWith,
    pub spell: Option<SpellDef>,
    pub abilities: &'static [AbilityDef],
    /// Alternative ways to cast it; way n is `ways[n - 1]`, way 0 its mana cost.
    pub ways: &'static [WayDef],
    /// Overload (CR 702.96a): the cost it may be cast for instead of its mana cost, changing every "target" in its text to
    /// "each". Cast that way (`WAY_OVERLOAD`) it has no targets, and its effect reads `item.way` to reach everything the
    /// target would have been chosen from.
    pub overload: Option<ManaCost>,
    /// An optional additional cost paid once, or any number of times (kicker, multikicker, squad: CR 702.33,
    /// 702.157).
    pub kicker: Option<Kicker>,
    /// An optional additional cost that is not mana (teamwork), announced as a kicker is. The question offers it by
    /// `labels`: the first is "Cast without it", the second the cost's text.
    pub optional_cost: Option<OptionalCost>,
    /// Its spell's first target slot divides a total among its targets (CR 601.2d), as Pyrokinesis's damage does.
    pub divide: Option<Divide>,
    /// Its modes, chosen as it is cast (CR 601.2b, 700.2); its `spell` is `SpellDef::MODAL`.
    pub modal: Option<&'static ModalDef>,
    /// The most modes the board lets it choose as it is cast, where that is more than `modal.max` (CR 700.2): Flame of Anor.
    pub modes_up_to: Option<ModeCount>,
    /// Its targets are chosen after the kicker and X are announced, because what may be targeted depends on them
    /// (CR 601.2b-c): Bloodchief's Thirst's kicked target.
    pub late_targets: bool,
    /// Delayed abilities created as it is cast for a way, with no stack item of their own (CR 608.3g, 603.7a): a pair
    /// of the way (as `ways[n - 1]`) and the index in `delayed`. The ability triggers at the beginning of the next end
    /// step, aimed at the permanent the spell becomes: dash's "return it to its owner's hand" (CR 702.109a), blitz's.
    pub end_step_riders: &'static [(u8, u8)],
    /// A cost it always adds to its mana cost (CR 118.8, 601.2b): Tinker's "sacrifice an artifact".
    pub additional_cost: Option<Cost>,
    /// An additional cost of which one of these is paid, chosen as it is cast (CR 601.2b): Bitter Triumph.
    pub additional_choice: &'static [ExtraCost],
    /// Its own entering replacements, applied as it enters from anywhere (CR 614.12).
    pub enters: Option<Enters>,
    /// Abilities its effects create to trigger later (CR 603.7), as `AbilityRef::delayed(card, n)`.
    pub delayed: &'static [AbilityDef],
    /// Its prepare spell (CR 722): what a copy cast while it is prepared is a copy of.
    pub prepare: Option<&'static CardDef>,
    /// Madness (CR 702.35): the cost it may be cast for when discarded. A discard puts it into exile instead of the
    /// graveyard, and a triggered ability then lets its owner cast it for this or puts it into the graveyard.
    pub madness: Option<ManaCost>,
    /// Miracle (CR 702.94): the cost it may be cast for when revealed as the first card drawn in a turn.
    pub miracle: Option<ManaCost>,
    /// "You may cast this card from your graveyard using its bestow ability" (CR 702.103): the bestowed Aura is cast from
    /// the graveyard for its bestow cost, and the card is not otherwise cast from there.
    pub bestow_from_graveyard: bool,
    /// A restriction on when this spell may be cast at all (CR 601.3): "Cast this spell only before the combat damage
    /// step". Judged for every way of casting it, from any zone.
    pub cast_only_if: Option<Condition>,
    /// A split card (CR 709): its two halves, each a card of its own that is the spell on the stack when it is cast
    /// (CR 709.3). The card itself has both halves' characteristics together in every other zone (CR 709.4). The
    /// definitions must be in the card table (listed in the card's `EXTRA`).
    pub split: Option<(&'static CardDef, &'static CardDef)>,
    /// Aftermath (CR 702.127a): this half may be cast only from its owner's graveyard, and is exiled when it leaves the
    /// stack.
    pub aftermath: bool,
    /// Reconfigure (CR 702.151): an Equipment creature with it may be attached to a creature (CR 301.5c).
    pub reconfigure: bool,
    /// Foretell (CR 702.143): the cost it is cast for from exile, on a later turn, once foretold.
    pub foretell: Option<ManaCost>,
    /// Suspend (CR 702.62).
    pub suspend: Option<SuspendDef>,
    /// Its Adventure (CR 715): a second set of characteristics it can be cast with from hand. The definition must be
    /// in the card table (listed in the card's `EXTRA`).
    pub adventure: Option<&'static CardDef>,
    /// Bestow (CR 702.103): the Aura it is while it is bestowed, an Aura enchantment with "enchant creature" and the bestow
    /// cost as its mana cost, which it is on the stack and on the battlefield until it ceases to be bestowed (CR 702.103e-f).
    /// The definition must be in the card table (listed in the card's `EXTRA`), and gives the Aura everything it has that the
    /// creature does: its other abilities as well as "enchanted creature gets ...".
    pub bestow: Option<&'static CardDef>,
    /// Rebound (CR 702.88a): cast from its caster's hand and resolving, it is exiled instead of put into the
    /// graveyard, and its controller may cast it from exile at the beginning of their next upkeep (`REBOUND_ABILITY`).
    pub rebound: bool,
    /// Paradigm (CR 702.170a): as it resolves it is exiled instead of going to the graveyard, and its owner may cast copies of it
    /// from exile (a triggered ability of the card, which works from exile).
    pub paradigm: bool,
    /// Its standing rules about playing lands and cards from other zones, in force while it is on the battlefield.
    pub play_rules: &'static [PlayRule],
    /// Texts its questions offer as options (`Label::Card`).
    pub labels: &'static [&'static str],
    /// Its static abilities that change characteristics (CR 604, 613).
    pub statics: &'static [StaticDef],
    /// Its replacement effects (CR 614): "if this would happen, that happens instead". Functioning while it is on the
    /// battlefield, or in every zone for its own move (`ReplacementDef::own_everywhere`).
    pub replacements: &'static [ReplacementDef],
    /// The replacement effects its resolving spells and abilities make for a duration (CR 611.2, 614): `EffectKind::Replacement(n)`
    /// and `ReplacementRule(n)` name the nth. Not standing abilities: they function only while such an effect lasts.
    pub granted_replacements: &'static [GrantedReplacementDef],
    /// "Each player can't cast more than this many spells each turn" (CR 101.2), while it is on the battlefield; 0 for
    /// no such limit. Archon of Emeria, Rule of Law.
    pub cast_limit: u8,
    /// Its static abilities that make other things cost more or less (CR 601.2f), while it is on the battlefield.
    pub cost_changes: &'static [CostChange],
    /// A spell's own cost changes (CR 601.2f): "this spell costs {1} less to cast for each ..." (affinity, Embercleave).
    /// They work wherever the spell is cast from, and the spell is their `source`.
    pub own_cost_changes: &'static [CostChange],
    /// Its static abilities that make activated abilities cost less, with at least one mana left (CR 601.2f), while it is
    /// on the battlefield.
    pub ability_discounts: &'static [AbilityDiscount],
    /// Its static abilities that let a player pay another mana cost rather than an activated ability's (CR 118.9), while
    /// it is on the battlefield.
    pub ability_alternatives: &'static [AbilityAlternative],
    /// Ways it gives cards in graveyards to be cast, while it is on the battlefield (CR 604, 118.9).
    pub granted_ways: &'static [GrantedWay],
    /// What it may do from an opening hand (CR 103.6a): begin the game on the battlefield.
    pub opening_hand: Option<OpeningHandAction>,
    /// Its back face (CR 711): the characteristics of the object when it is transformed. A double-faced card's
    /// back face is not a card of its own: nothing can be drawn or cast as it.
    pub back: Option<&'static CardDef>,
    /// Its back face is a land that may be played from where the card could be played as a land (CR 712.4, 305.1): a
    /// modal double-faced card. The land enters as the back face (CR 712.14a).
    pub back_land: bool,
    /// Its printed basic land types' mana, in printed order: the order its intrinsic mana abilities have (CR 305.6).
    pub basic_land_types: &'static [Mana],
    pub text: &'static str,
    /// What an Aura may enchant (CR 303.4a, 702.5): the one target of the Aura spell, and what a legal host is for as
    /// long as it is attached (CR 704.5m).
    pub enchant: Option<TargetSpec>,
    /// The enchant ability it gains when it becomes an Aura (CR 303.4, 702.5a, 613.1d), by an effect that adds the Aura subtype
    /// to it: Necromancy's \"enchant creature put onto the battlefield with Necromancy\". It is not an Aura spell, so it targets nothing.
    pub becomes_aura_enchant: Option<TargetSpec>,
    /// \"You may cast this spell as though it had flash\" (CR 307.5, 702.8): it may be cast whenever its caster could cast an instant.
    pub cast_as_though_flash: bool,
    /// Ascend (CR 702.131): while a permanent of this card is on the battlefield and its controller controls ten or more
    /// permanents, they get the city's blessing.
    pub ascend: bool,
    /// A permanent of this card whose controller sacrifices it at the beginning of the cleanup step when this says so (CR 514.1, 603.7):
    /// Necromancy's \"if you cast it any time a sorcery couldn't have been cast\".
    pub sacrifice_at_cleanup: Option<fn(&dyn Query, ObjectId) -> bool>,
    /// A token definition that other token definitions share a name with (three Golems that differ in a keyword): the
    /// card table keeps such definitions apart instead of merging definitions of one name.
    pub shares_name: bool,
    /// Splice onto Arcane (CR 702.47): the cost paid to add this card's spell effect to an Arcane spell as it is cast. The
    /// spell ability it adds must have no targets.
    pub splice: Option<ManaCost>,
    /// A dungeon card (CR 309): for each room, the rooms the arrows from it lead to (CR 309.5a); a room with none is
    /// the bottommost. The room's ability is the ability of the same index, and its name and text the label.
    pub dungeon: &'static [&'static [u8]],
    /// Its static abilities that forbid casting spells (CR 101.2).
    pub cast_rules: &'static [CastRule],
    /// Its static abilities that limit blocking (CR 509.1b), while it is on the battlefield.
    pub block_rules: &'static [BlockRule],
    /// Its static abilities that change who can target what and whether ward triggers, while it is on the battlefield.
    pub targeting_rules: &'static [TargetingRule],
    /// Its static abilities that make some damage unpreventable (CR 615.12).
    pub unpreventable_damage: &'static [UnpreventableRule],
    /// "Each creature you control assigns combat damage equal to its toughness rather than its power" (CR 510.1a):
    /// Assault Formation, Doran, the Siege Tower.
    pub damage_by_toughness: &'static [DamageByToughnessRule],
    /// "You may spend mana as though it were mana of any color to activate abilities of creatures you control" (CR 609.4b):
    /// Agatha's Soul Cauldron, Mycosynth Lattice's kind.
    pub spend_any_color: &'static [SpendAnyColorRule],
    /// Its static abilities that forbid spells and abilities to cause discards or sacrifices (CR 101.2).
    pub cause_rules: &'static [CauseRule],
    /// Its replacements of its controller's turn beginning (CR 614.10).
    pub turn_begin: &'static [TurnBeginRule],
    /// Its static abilities that allow sorcery-speed abilities at instant speed (CR 606.3).
    pub instant_activation: &'static [InstantActivationRule],
    /// Its static abilities that let spells be cast as though they had flash (CR 307.5): "You may cast Aura and Equipment spells
    /// as though they had flash" (Sigarda's Aid). The battlefield is searched for these only when something is cast.
    pub flash_rules: &'static [FlashRule],
    /// Its static abilities that forbid activating abilities (CR 602.5): Clarion Conqueror, Null Rod, Stony Silence.
    pub activation_rules: &'static [ActivationRule],
    /// Gift (CR 702.174): an optional additional cost of promising an opponent a gift, given as it resolves, before its
    /// other effects (an instant or sorcery), or when it enters (a permanent).
    pub gift: Option<GiftDef>,
}

/// A card's gift (CR 702.174a): which gift it is, for the promise to name, and what giving it does to the chosen
/// player. `cantrip_mechanics::gift` has one for each kind.
#[derive(Clone, Copy)]
pub struct GiftDef {
    pub kind: Gift,
    /// Gives the gift to the chosen opponent (CR 702.174d-i).
    pub give: fn(&mut dyn Game, PlayerId) -> Result<(), Suspend>,
}

impl CardDef {
    pub const BLANK: CardDef = CardDef {
        name: "",
        cost: ManaCost::ZERO,
        has_cost: true,
        colors: None,
        types: TypeSet::NONE,
        supertypes: Supertypes { basic: false, legendary: false, snow: false },
        subtypes: SubtypeSet::NONE,
        power: None,
        toughness: None,
        loyalty: None,
        defense: None,
        keywords: Keywords::NONE,
        pay_with: PayWith::NONE,
        spell: None,
        abilities: &[],
        ways: &[],
        kicker: None,
        overload: None,
        optional_cost: None,
        divide: None,
        modal: None,
        modes_up_to: None,
        end_step_riders: &[],
        late_targets: false,
        additional_cost: None,
        additional_choice: &[],
        enters: None,
        delayed: &[],
        prepare: None,
        madness: None,
        miracle: None,
        bestow_from_graveyard: false,
        cast_only_if: None,
        split: None,
        aftermath: false,
        reconfigure: false,
        foretell: None,
        suspend: None,
        adventure: None,
        bestow: None,
        rebound: false,
        paradigm: false,
        play_rules: &[],
        labels: &[],
        statics: &[],
        replacements: &[],
        granted_replacements: &[],
        cast_limit: 0,
        cost_changes: &[],
        own_cost_changes: &[],
        ability_discounts: &[],
        ability_alternatives: &[],
        granted_ways: &[],
        opening_hand: None,
        back: None,
        back_land: false,
        basic_land_types: &[],
        text: "",
        enchant: None,
        becomes_aura_enchant: None,
        cast_as_though_flash: false,
        ascend: false,
        sacrifice_at_cleanup: None,
        shares_name: false,
        splice: None,
        dungeon: &[],
        cast_rules: &[],
        block_rules: &[],
        targeting_rules: &[],
        unpreventable_damage: &[],
        damage_by_toughness: &[],
        spend_any_color: &[],
        cause_rules: &[],
        turn_begin: &[],
        instant_activation: &[],
        flash_rules: &[],
        activation_rules: &[],
        gift: None,
    };

    pub fn colors(&self) -> ColorSet {
        self.colors.unwrap_or_else(|| self.cost.colors())
    }

    /// A Room (CR 709.5): a split card whose halves share a type line, each half a door.
    pub fn is_room(&self) -> bool {
        self.split.is_some() && self.subtypes.has(Subtype::Room)
    }

    /// The doors of a Room that the unlocked designations `doors` name, left first (CR 709.5c).
    pub fn unlocked_doors(&self, doors: u8) -> impl Iterator<Item = &'static CardDef> {
        let (left, right) = self.split.map_or((None, None), |(l, r)| (Some(l), Some(r)));
        [left.filter(|_| doors & DOOR_LEFT != 0), right.filter(|_| doors & DOOR_RIGHT != 0)].into_iter().flatten()
    }

    /// The mana cost of a Room permanent with the unlocked designations `doors`: the combined costs of its unlocked
    /// doors, and none at all when neither is (CR 709.5, 709.4b).
    pub fn room_cost(&self, doors: u8) -> ManaCost {
        self.unlocked_doors(doors).fold(ManaCost::ZERO, |sum, half| sum.plus(&half.cost))
    }

    pub fn is(&self, t: CardType) -> bool {
        self.types.has(t)
    }

    /// The target slots of its spell (CR 601.2c): an instant's or sorcery's own, or the one an Aura spell has in its
    /// enchant ability (CR 303.4a).
    pub fn spell_targets(&'static self) -> &'static [TargetSpec] {
        match (&self.spell, &self.enchant) {
            (Some(spell), _) => spell.targets,
            (None, Some(enchant)) => std::slice::from_ref(enchant),
            _ => &[],
        }
    }
}

/// Every card the game knows, built once per process and shared, never copied.
pub struct CardTable {
    pub cards: Vec<&'static CardDef>,
    /// Which cards have a triggered ability that functions outside the battlefield, so the rest are never looked at
    /// there.
    pub anywhere: Vec<bool>,
    /// Which cards have static abilities, and whether any does: the layers look for sources only when one might be.
    pub statics: Vec<bool>,
    pub any_statics: bool,
    /// Whether any card has a static ability that functions from a graveyard, so graveyards are looked in at all.
    pub graveyard_statics: bool,
    /// Whether any card has a static ability that functions from exile (cipher's, CR 702.99a), so exile is looked in.
    pub exile_statics: bool,
    /// Which cards have play rules, and whether any does: land plays look for them only when one might.
    pub play_rules: Vec<bool>,
    pub any_play_rules: bool,
    /// Which cards have replacement effects, and whether any does: events look for sources only when one might be.
    pub replaces: Vec<bool>,
    pub any_replaces: bool,
    /// Whether any card limits the spells cast each turn.
    pub any_cast_limit: bool,
    /// Which cards have block rules (CR 509.1b): looked at only for the blocker and the attacker of a possible block.
    pub block_rules: Vec<bool>,
    /// Per card: it has a targeting rule. Read to find the few battlefield sources of one, never to ask the table.
    pub targeting_rules: Vec<bool>,
    /// Which cards have a triggered mana ability: looked for among the permanents only when a permanent is tapped for mana.
    pub mana_triggers: Vec<bool>,
    /// Which cards have cost changes (CR 601.2f): the board is looked at for their sources, never for the card table's
    /// sake (a table holds every card, so "does any card" is always true).
    pub cost_changes: Vec<bool>,
    /// Which cards forbid activating abilities (CR 602.5): their sources are found on the board, as cost changes' are.
    pub activation_rules: Vec<bool>,
    /// The rules' own abilities (`crate::intrinsic`): what a basic land type gives a land (CR 305.6).
    pub intrinsic: CardId,
    /// What a face-down permanent is (CR 708.2a): its copiable values.
    pub face_down: CardId,
    /// A Room's two doors (CR 709.5), by the Room's id: the ids of its halves, whose abilities its permanent has while
    /// they are unlocked.
    pub halves: Vec<Option<(CardId, CardId)>>,
    /// A double-faced card's back face, by the front's id (CR 711).
    pub backs: Vec<Option<CardId>>,
    /// The front face of a back face, by the back's id: a decklist may name either face.
    pub fronts: Vec<Option<CardId>>,
    /// The Undercity dungeon card, which the initiative ventures into (CR 726.2), if the game has it.
    pub undercity: Option<CardId>,
    /// A card's characteristics when nothing acts on it (the layers' fast path), made on first use: `[0]` before the
    /// ability layer, `[1]` with the abilities the rules give lands and Sieges. A pure function of the card alone.
    pub printed: Vec<[std::sync::OnceLock<Box<Chars>>; 2]>,
}

impl CardTable {
    pub fn new(cards: impl IntoIterator<Item = &'static CardDef>) -> CardTable {
        let mut cards: Vec<&'static CardDef> = cards.into_iter().collect();
        cards.push(&crate::intrinsic::INTRINSIC);
        cards.push(&crate::intrinsic::FACE_DOWN);
        // A back face is registered with its front (the first of a name wins, as for every card).
        let backs: Vec<&'static CardDef> = cards.iter().filter_map(|c| c.back).collect();
        cards.extend(backs);
        // Stable, and the first of a name kept: a card's own file comes before its characteristics-only definition.
        cards.sort_by(|a, b| a.name.cmp(b.name));
        // Only the same definition twice is merged: two tokens of one name (a Germ, a Golem) are two definitions,
        // and the registry already leaves out a characteristics-only card that has a file of its own.
        cards.dedup_by(|a, b| std::ptr::eq(*a, *b));
        // A granted static ability's effect is applied in the layers from 7a on, after the layer-6 pass that decides which
        // abilities an object has (`layers.rs`): one for an earlier layer would be quietly absent, so it is refused.
        for card in &cards {
            for ability in card.abilities.iter().chain(card.delayed) {
                if let AbilityDef::Static(s) = ability {
                    assert!(s.layer >= Layer::PtDefine, "{}: a granted static ability applies in layer 7 only (CR 613.1f)", card.name);
                }
            }
        }
        let anywhere = cards
            .iter()
            .map(|c| c.abilities.iter().any(|a| matches!(a, AbilityDef::Triggered(t) if t.anywhere)))
            .collect();
        let statics: Vec<bool> = cards.iter().map(|c| c.statics.iter().any(|s| !s.own)).collect();
        let any_statics = statics.iter().any(|&s| s);
        let graveyard_statics = cards.iter().any(|c| c.statics.iter().any(|s| s.from == ZoneKind::Graveyard));
        let exile_statics = cards.iter().any(|c| c.statics.iter().any(|s| s.from == ZoneKind::Exile));
        let play_rules: Vec<bool> = cards.iter().map(|c| !c.play_rules.is_empty()).collect();
        let any_play_rules = play_rules.iter().any(|&s| s);
        let replaces: Vec<bool> = cards.iter().map(|c| !c.replacements.is_empty()).collect();
        let any_replaces = replaces.iter().any(|&r| r);
        let any_cast_limit = cards.iter().any(|c| c.cast_limit > 0);
        let cost_changes: Vec<bool> = cards.iter().map(|c| !c.cost_changes.is_empty() || !c.ability_discounts.is_empty()).collect();
        let activation_rules: Vec<bool> = cards.iter().map(|c| !c.activation_rules.is_empty()).collect();
        let block_rules: Vec<bool> = cards.iter().map(|c| !c.block_rules.is_empty()).collect();
        let targeting_rules: Vec<bool> = cards.iter().map(|c| !c.targeting_rules.is_empty()).collect();
        let mana_triggers: Vec<bool> =
            cards.iter().map(|c| c.abilities.iter().any(|a| matches!(a, AbilityDef::Triggered(t) if t.mana_ability))).collect();
        let intrinsic = CardId(
            cards.iter().position(|c| std::ptr::eq(*c, &crate::intrinsic::INTRINSIC)).expect("the intrinsic abilities") as u16,
        );
        let face_down = CardId(
            cards.iter().position(|c| std::ptr::eq(*c, &crate::intrinsic::FACE_DOWN)).expect("the face-down values") as u16,
        );
        let undercity = cards.iter().position(|c| c.name == "Undercity" && !c.dungeon.is_empty()).map(|i| CardId(i as u16));
        let mut table = CardTable {
            cards,
            anywhere,
            statics,
            any_statics,
            graveyard_statics,
            exile_statics,
            play_rules,
            any_play_rules,
            replaces,
            any_replaces,
            any_cast_limit,
            block_rules,
            targeting_rules,
            mana_triggers,
            cost_changes,
            activation_rules,
            intrinsic,
            face_down,
            undercity,
            backs: Vec::new(),
            halves: Vec::new(),
            fronts: Vec::new(),
            printed: Vec::new(),
        };
        let count = table.cards.len();
        table.printed = (0..count).map(|_| [std::sync::OnceLock::new(), std::sync::OnceLock::new()]).collect();
        table.backs = vec![None; count];
        table.halves = vec![None; count];
        table.fronts = vec![None; count];
        for front in 0..count {
            if let Some(back) = table.cards[front].back {
                let back = table.find(back.name).expect("a back face is in the table");
                table.backs[front] = Some(back);
                table.fronts[back.0 as usize] = Some(CardId(front as u16));
            }
            // A Room's doors (CR 709.5): the abilities of an unlocked one are the permanent's. What a door's text does
            // beyond abilities (static abilities, replacement effects, entering rules) is read off the card, not off the
            // door, so a door that has any is refused until those follow the designations: never quietly absent.
            let card = table.cards[front];
            if card.is_room() {
                let (left, right) = card.split.expect("a Room is a split card");
                for half in [left, right] {
                    assert!(
                        half.statics.is_empty() && half.replacements.is_empty() && half.enters.is_none(),
                        "{}: a door with static abilities, replacement effects or an enters rule is not supported yet (CR 709.5)",
                        half.name
                    );
                }
                let ids = (table.find(left.name), table.find(right.name));
                table.halves[front] = Some((ids.0.expect("a door is in the card table"), ids.1.expect("a door is in the card table")));
            }
        }
        table
    }

    pub fn get(&self, id: CardId) -> &'static CardDef {
        self.cards[id.0 as usize]
    }

    /// The definition an object currently has: a copy's, else the face showing (CR 707.2, 712.3). The printed card of a
    /// transformed permanent is its front.
    pub fn current(&self, obj: &Obj) -> CardId {
        obj.copy.map(|c| c.card).unwrap_or_else(|| self.face(obj.card, obj.transformed))
    }

    /// The face of a card that shows when it is transformed (CR 712.3), or the card itself when it has no back.
    pub fn face(&self, id: CardId, transformed: bool) -> CardId {
        if transformed { self.backs[id.0 as usize].unwrap_or(id) } else { id }
    }

    /// The first card of a name, in the order the registry gave (a card's own definition before the Aura face it has while
    /// bestowed, which shares its name).
    pub fn find(&self, name: &str) -> Option<CardId> {
        let first = self.cards.partition_point(|card| card.name < name);
        (self.cards.get(first).is_some_and(|card| card.name == name).then_some(first))
            .or_else(|| self.cards.iter().position(|card| card.name.eq_ignore_ascii_case(name)))
            .map(|i| CardId(i as u16))
    }

    pub fn ability(&self, ability: AbilityRef) -> &'static AbilityDef {
        if ability.is_madness() {
            return &MADNESS_ABILITY;
        }
        if ability.is_rebound() {
            return &REBOUND_ABILITY;
        }
        if ability.is_miracle() {
            return &MIRACLE_ABILITY;
        }
        let card = self.get(ability.card);
        if ability.is_delayed() {
            &card.delayed[(ability.index - AbilityRef::DELAYED) as usize]
        } else {
            &card.abilities[ability.index as usize]
        }
    }

    /// The id a definition has in this table: for token definitions and other cards' own references.
    pub fn id_of(&self, def: &'static CardDef) -> CardId {
        CardId(self.cards.iter().position(|c| std::ptr::eq(*c, def)).expect("a definition in the table") as u16)
    }
}

/// Madness's linked triggered ability (CR 702.35a), the core's own: "when this card is exiled this way, its owner may
/// cast it by paying [its madness cost] rather than its mana cost; if they don't, they put it into their graveyard."
/// The card is the ability's source, in exile where the discard put it, and its cost is the card's `madness`.
pub static MADNESS_ABILITY: AbilityDef = AbilityDef::Triggered(TriggeredDef {
    text: "Madness: cast it for its madness cost or put it into your graveyard.",
    effect: madness_resolves,
    ..TriggeredDef::BLANK
});

fn madness_resolves(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let card = item.object;
    let in_exile = |g: &dyn Game| crate::game::is_live(g, card) && g.obj(card).zone.is(ZoneKind::Exile);
    // Nothing to do for a card something moved out of exile while the ability waited (CR 400.7).
    if !in_exile(g) {
        return Ok(());
    }
    let cost = g.card(g.obj(card).card).madness;
    // Nobody is asked about a card that could not be cast at all (a land, no legal target): it is simply not cast.
    if let Some(cost) = cost
        && g.could_cast_during_resolution(card)
        && matches!(g.ask(item.controller, RequestKind::May { source: card, purpose: Purpose::CastForMadness })?, Answer::Yes)
    {
        g.cast_during_resolution(card, Some(cost))?;
    }
    // "If that player doesn't, they put this card into their graveyard": still in exile means it was not cast.
    if in_exile(g) {
        let owner = g.obj(card).owner;
        g.move_to(card, Zone::graveyard(owner));
    }
    Ok(())
}

/// Miracle's linked triggered ability (CR 702.94a), the core's own: "you may cast it by paying [its miracle cost] rather
/// than its mana cost." The card is the ability's source, still in the hand it was revealed from; nobody is asked about
/// a card that could not be cast at all, and a "no", or a cast given up, leaves it where it is.
pub static MIRACLE_ABILITY: AbilityDef = AbilityDef::Triggered(TriggeredDef {
    text: "Miracle: cast it by paying its miracle cost.",
    effect: miracle_resolves,
    ..TriggeredDef::BLANK
});

fn miracle_resolves(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let card = item.object;
    let Some(cost) = g.card(g.obj(card).card).miracle else { return Ok(()) };
    if g.could_cast_during_resolution(card)
        && matches!(g.ask(item.controller, RequestKind::May { source: card, purpose: Purpose::CastForMiracleCost })?, Answer::Yes)
    {
        g.cast_during_resolution(card, Some(cost))?;
    }
    Ok(())
}

/// Rebound's delayed triggered ability (CR 702.88a), the core's own: "at the beginning of your next upkeep, you may
/// cast this card from exile without paying its mana cost." The card is the ability's source, in exile where its
/// resolution put it. A card that could not be cast (no legal target), a "may" answered no and a cast given up all leave
/// it in exile, and it is not offered again.
pub static REBOUND_ABILITY: AbilityDef = AbilityDef::Triggered(TriggeredDef {
    text: "Rebound: you may cast this card from exile without paying its mana cost.",
    effect: rebound_resolves,
    ..TriggeredDef::BLANK
});

fn rebound_resolves(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let card = item.object;
    // Nothing to do for a card something moved out of exile while the ability waited (CR 603.7c, 400.7).
    if !crate::game::is_live(g, card) || !g.obj(card).zone.is(ZoneKind::Exile) {
        return Ok(());
    }
    if g.could_cast_during_resolution(card)
        && matches!(g.ask(item.controller, RequestKind::May { source: card, purpose: Purpose::CastFree })?, Answer::Yes)
    {
        g.cast_during_resolution(card, Some(ManaCost::ZERO))?;
    }
    Ok(())
}
