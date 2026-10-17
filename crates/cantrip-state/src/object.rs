//! The rows the state is made of. Every one is `Copy`: plain data that names everything else by id.

use crate::vocab::*;

/// One object (CR 109): a card or token in some zone, or a copy of a card on the stack.
///
/// Characteristics are not stored here beyond the card it is: they are computed from the card, the copy effect it
/// entered with and the continuous effects that apply to it (CR 613), when asked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Obj {
    pub card: CardId,
    pub owner: PlayerId,
    pub controller: PlayerId,
    pub zone: Zone,
    pub token: bool,
    pub tapped: bool,
    /// The turn it came under its controller's control: summoning sick (CR 302.6) while that is not before the
    /// turn its controller's most recent turn began on.
    pub controlled_since: u32,
    pub damage: u16,
    /// Dealt damage by a source with deathtouch since the last state-based action check (CR 704.5h).
    pub deathtouched: bool,
    pub counters: [i16; CounterKind::COUNT],
    /// When it entered its zone (CR 613.7d), for ordering and for layer timestamps.
    pub timestamp: u32,
    /// The copy effect it entered with (CR 707.2, 613.2): the values it copied, which are its own copiable values.
    pub copy: Option<Copiable>,
    /// The turn a loyalty ability of it was last activated (CR 606.3), zero for never.
    pub loyalty_turn: u32,
    /// The new object it became when it changed zones (CR 400.7), so that a reference to the old one can be
    /// followed where the rules allow ("exile that card").
    pub became: Option<ObjectId>,
    /// If it would leave the battlefield, it is exiled instead (unearth, CR 702.84a; and the like).
    pub exile_on_leave: bool,
    /// How many times its kicker (or squad, or multikicker) cost was paid as it was cast (CR 702.33d), carried from
    /// the spell to the permanent it becomes.
    pub kicked: u8,
    /// How many times its `optional_cost` (an additional cost that is not kicker: bargain, teamwork, behold, blight,
    /// CR 601.2b) was paid as it was cast, carried from the spell to the permanent it becomes. Kept apart from `kicked`
    /// so a card with kicker and another optional cost is not ambiguous.
    pub paid: u8,
    /// The opponent its gift was promised to as it was cast (CR 702.174k), carried from the spell to the permanent it
    /// becomes: "if its gift cost was paid" (CR 702.174b). A copy of the spell has it too (CR 707.10); a permanent
    /// that enters as a copy of one does not (CR 707.9).
    pub gift: Option<PlayerId>,
    /// The mana spent to cast it, by kind (CR 601.2h): what converge and "if at least two of a color were spent"
    /// read. Carried from the spell to the permanent; a copy has none.
    pub spent: ManaPool,
    /// Which way it was cast (0 its mana cost, n its nth alternative cost), carried to the permanent: evoke's "if
    /// its evoke cost was paid".
    pub cast_way: u8,
    /// A Class's level (CR 716): 0 for never set, which reads as 1.
    pub class_level: u8,
    /// Prepared (CR 722.1): while it is, its controller may cast a copy of its prepare spell.
    pub prepared: bool,
    /// A copy of a card (CR 707.12, 722.3): not a card; it ceases to exist anywhere but where it may be, and leaves
    /// the stack for nowhere.
    pub card_copy: bool,
    /// For a prepare spell's copy in exile: the prepared permanent it belongs to.
    pub prepared_from: Option<ObjectId>,
    /// A color chosen as it entered ("as this enters, choose a color", CR 614.12).
    pub chosen_color: Option<Color>,
    /// A creature or land type chosen as it entered.
    pub chosen_subtype: Option<Subtype>,
    /// Exiled "on an adventure" (CR 715.4): its controller may cast the creature half from exile (CR 715.5).
    pub on_adventure: bool,
    /// For a spell: the kind of zone it was cast from (CR 601.2a), what rebound ("cast from your hand") and "whenever
    /// you cast a spell from exile" ask. Not carried on when it leaves the stack.
    pub cast_from: Option<ZoneKind>,
    /// The turn some of its activated abilities were last activated, and which of them (bit n is the nth of its
    /// computed ability list): "activate only once each turn" (CR 602.5b-c), counted for this object and for each
    /// instance of the ability on its own.
    pub activated_turn: u32,
    pub activated_mask: u8,
    /// What it is attached to (CR 301.5, 303.4): an Aura's or Equipment's host. Only meaningful while the host is
    /// still the object it was (`live`) and where an attachment may be; state-based actions clear it otherwise
    /// (CR 704.5m-n). Kept on the row of an object that left, as last-known information.
    pub attached_to: Option<ObjectId>,
    /// The player an Aura with "enchant player" enchants (CR 303.4, 702.5): read by "enchanted player".
    pub attached_player: Option<PlayerId>,
    /// The Aura it was put onto the battlefield with and attached to (Animate Dead's "enchant creature put onto the
    /// battlefield with this Aura").
    pub entered_with: Option<ObjectId>,
    /// The X its spell was cast with (CR 107.3m), carried to the permanent: an enters trigger that reads X.
    pub x: u16,
    /// Designations and choices a permanent carries that no counter or characteristic holds.
    pub marks: Marks,
    /// A double-faced permanent with its back face up (CR 711.2, 712.3): its characteristics are the back face's.
    pub transformed: bool,
    /// For a card in exile: the permanent whose ability exiled it, which a linked ability finds it by (CR 607.2a).
    /// A new object (CR 400.7) has none.
    pub exiled: Option<ExileLink>,
    /// Face down (CR 708): a permanent is a nameless 2/2 creature, its card's real values kept in `card` (its `copy` is the
    /// face-down values, CR 708.2a); a card in exile has no characteristics for anyone who has not seen it (CR 406.3).
    pub face_down: bool,
    /// For a face-down card in exile: the seats that may look at it, a bit each (CR 406.3). Those who could see where it
    /// came from, and those it was shown to since.
    pub seen_by: u8,
}

/// What a permanent is marked with: exerted (CR 701.43), monstrous (CR 701.37), solved (CR 719.3b), saddled (CR 702.171), a player chosen
/// as it entered (CR 614.12a), and whose it was before an effect changed its controller (CR 613.1b). All of it ends
/// when the object does: a permanent that changes zones is a new object (CR 400.7) with none.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Marks {
    /// The unlocked designations of a permanent with a shared type line (CR 709.5c): `DOOR_LEFT` is "left half unlocked",
    /// `DOOR_RIGHT` "right half unlocked". Given as it enters (CR 709.5d) or by an unlock (CR 709.5e-f); a new object
    /// (CR 400.7) has neither.
    pub doors: u8,
    /// The player whose next untap step it sits out (CR 701.43a).
    pub exerted_by: Option<PlayerId>,
    /// It has become monstrous (CR 701.37b).
    pub monstrous: bool,
    /// It has become solved (CR 719.3b): a designation, neither an ability nor a copiable value, kept until it leaves the
    /// battlefield. A permanent that becomes a copy of a solved Case is not solved; a solved Case that becomes a copy of
    /// another Case stays solved.
    pub solved: bool,
    /// The player protecting a battle (CR 310.9): chosen as it entered (CR 310.9a), the defending player of what
    /// attacks it (CR 310.9d) and the only one who may block that.
    pub protector: Option<PlayerId>,
    /// A creature spell paid for with mana that makes it gain haste (CR 106.6): the permanent it becomes has haste
    /// until end of turn.
    pub gains_haste: bool,
    /// An ability the permanent a spell becomes gains (a granted way's "if you do, it gains ...", Serra Paragon): a delayed
    /// ability of the card that gave the way, added as the spell resolves.
    pub gains: Option<AbilityRef>,
    /// A spell cast at a time a sorcery could not have been cast (CR 307.1, 117.1a), carried to the permanent it becomes: Necromancy's
    /// "if you cast it any time a sorcery couldn't have been cast".
    pub cast_untimely: bool,
    /// A spell paid for with mana that makes it uncounterable (CR 106.6, 701.5).
    pub cant_be_countered: bool,
    /// The number of the turn it was saddled (CR 702.171b): saddled until that turn ends. Zero for never.
    pub saddled_turn: u32,
    /// A player chosen as it entered ("as this enters, choose a player", CR 614.12a).
    pub chosen_player: Option<PlayerId>,
    /// A card name chosen as it entered ("choose a card name", CR 614.12): Anointed Peacekeeper's.
    pub chosen_card: Option<CardId>,
    /// Odd (true) or even (false), chosen as it entered (CR 614.12a).
    pub chosen_odd: Option<bool>,
    /// Who controlled it before the control-changing effects that apply to it now (CR 613.1b): it goes back to them
    /// when the last of those ends. `None` while no such effect applies.
    pub control_base: Option<PlayerId>,
}

/// A card exiled by a permanent's ability, and remembered as exiled with it (CR 607.2a, 610.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExileLink {
    /// The permanent whose ability exiled it, as the object it was then.
    pub by: ObjectId,
    /// Where it was exiled from, and where it returns to.
    pub from: Zone,
    /// "Until [by] leaves the battlefield" (CR 610.3): it returns as soon as `by` is no longer there.
    pub until_leaves: bool,
    /// Exiled encoded on `by` by cipher (CR 702.99b): it is encoded on that creature while both stay where they are.
    pub encoded: bool,
}

impl Obj {
    /// A fresh object in `zone`, with nothing about it set: the rules' own source for an ability no card has (the
    /// monarch's, a dungeon room's), which is in no zone's list.
    pub fn new(card: CardId, owner: PlayerId, zone: Zone) -> Obj {
        Obj {
            card,
            owner,
            controller: owner,
            zone,
            token: false,
            tapped: false,
            controlled_since: 0,
            damage: 0,
            deathtouched: false,
            counters: [0; CounterKind::COUNT],
            timestamp: 0,
            copy: None,
            loyalty_turn: 0,
            became: None,
            exile_on_leave: false,
            kicked: 0,
            paid: 0,
            gift: None,
            spent: ManaPool::default(),
            cast_way: 0,
            class_level: 0,
            prepared: false,
            card_copy: false,
            prepared_from: None,
            chosen_color: None,
            chosen_subtype: None,
            on_adventure: false,
            cast_from: None,
            activated_turn: 0,
            activated_mask: 0,
            attached_to: None,
            attached_player: None,
            entered_with: None,
            x: 0,
            marks: Marks::default(),
            transformed: false,
            exiled: None,
            face_down: false,
            seen_by: 0,
        }
    }

    pub fn counters(&self, kind: CounterKind) -> i16 {
        self.counters[kind as usize]
    }
}

/// Copiable values (CR 707.2): a card's printed values, and the exceptions copy effects have added to them
/// (CR 707.9b-d). A copy of a copy copies these, exceptions included.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Copiable {
    pub card: CardId,
    /// Card types added by exceptions: Phyrexian Metamorph is an artifact in addition to its other types.
    pub extra_types: TypeSet,
    /// Subtypes added by exceptions, as Phantasmal Image is an Illusion in addition to its other types.
    pub extra_subtypes: SubtypeSet,
    /// Abilities added by exceptions, in order.
    pub extra_abilities: [Option<AbilityRef>; 4],
    /// Keywords added by exceptions: "except it has haste".
    pub extra_keywords: Keywords,
    /// "Except it's black" (CR 707.9b): these are its colors, and a color-defining ability it copies is left behind
    /// (CR 707.9d).
    pub colors: Option<ColorSet>,
    /// "Except it's 4/4" (CR 707.9b, 707.9d): its power and toughness, whatever the copied card has or defines.
    pub pt: Option<(i16, i16)>,
    /// "Except it has no mana cost" (CR 707.9b): its mana value is 0 and it cannot be cast for a cost.
    pub no_cost: bool,
    /// "Except its name is X" (CR 707.9b): its name, whatever the copied card is called.
    pub name: Option<&'static str>,
    /// "Except it's legendary" (CR 707.9b): legendary in addition to the supertypes the copied card has.
    pub legendary: bool,
    /// The words of its text changed from the card's (CR 612.4, 612.2a): a token made by an ability whose text was
    /// changed has the colors and creature types the changed text names, and so does a copy of it.
    pub words: Words,
}

impl Copiable {
    /// A card's own printed values: a copy of it with no exceptions.
    pub const fn of(card: CardId) -> Copiable {
        Copiable {
            card,
            extra_types: TypeSet::NONE,
            extra_subtypes: SubtypeSet::NONE,
            extra_abilities: [None; 4],
            extra_keywords: Keywords::NONE,
            colors: None,
            pt: None,
            no_cost: false,
            name: None,
            legendary: false,
            words: Words::NONE,
        }
    }

    /// "Except its name is X."
    pub fn named(mut self, name: &'static str) -> Copiable {
        self.name = Some(name);
        self
    }

    /// Whether any exception is stated: a copy with none is the card itself.
    pub fn has_exceptions(&self) -> bool {
        self.extra_types.0 != 0
            || !self.extra_subtypes.is_empty()
            || self.extra_abilities.iter().any(|a| a.is_some())
            || self.extra_keywords.0 != 0
            || self.colors.is_some()
            || self.pt.is_some()
            || self.name.is_some()
            || self.no_cost
            || self.legendary
            || !self.words.is_printed()
    }

    /// "Except it's a [type] in addition to its other types."
    pub fn with_type(mut self, t: CardType) -> Copiable {
        self.extra_types = self.extra_types.with(t);
        self
    }

    pub fn with_subtype(mut self, s: Subtype) -> Copiable {
        self.extra_subtypes = self.extra_subtypes.with(s);
        self
    }

    /// "Except it has [this ability]."
    pub fn with_ability(mut self, ability: AbilityRef) -> Copiable {
        if let Some(slot) = self.extra_abilities.iter().position(|a| a.is_none()) {
            self.extra_abilities[slot] = Some(ability);
        }
        self
    }

    /// "Except it has [keyword]."
    pub fn with_keyword(mut self, k: Keyword) -> Copiable {
        self.extra_keywords = self.extra_keywords.with(k);
        self
    }

    /// "Except it's [color]."
    pub fn colored(mut self, colors: ColorSet) -> Copiable {
        self.colors = Some(colors);
        self
    }

    /// "Except it's [power]/[toughness]."
    pub fn sized(mut self, power: i16, toughness: i16) -> Copiable {
        self.pt = Some((power, toughness));
        self
    }

    /// "Except it has no mana cost."
    pub fn costless(mut self) -> Copiable {
        self.no_cost = true;
        self
    }

    /// "Except it's legendary" (CR 707.9b, 205.4): the supertype added to whatever the copied card has.
    pub fn legendary(mut self) -> Copiable {
        self.legendary = true;
        self
    }
}

/// What a target is (CR 115).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Target {
    #[default]
    None,
    Player(PlayerId),
    Object(ObjectId),
    /// A triggered or activated ability on the stack (CR 115.1, 113.7a), named by the serial it was put there with.
    Ability(u32),
}

impl Target {
    pub fn object(self) -> Option<ObjectId> {
        match self {
            Target::Object(id) => Some(id),
            _ => None,
        }
    }

    pub fn player(self) -> Option<PlayerId> {
        match self {
            Target::Player(p) => Some(p),
            _ => None,
        }
    }
}

/// The most targets one spell or ability can have: "any number of target ..." (Appa) is bounded by what a board holds.
pub const MAX_TARGETS: usize = 16;

/// What kind of thing is on the stack (CR 405).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StackKind {
    Spell,
    Activated,
    Triggered,
}

/// One spell or ability on the stack, with everything announced for it (CR 601.2, 602.2, 603.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StackItem {
    pub kind: StackKind,
    /// The spell's own object, or the ability's source as it was when the ability was put on the stack.
    pub object: ObjectId,
    /// The ability: for a spell, its card's spell ability.
    pub ability: AbilityRef,
    pub controller: PlayerId,
    pub targets: [Target; MAX_TARGETS],
    pub target_count: u8,
    /// For each target, parallel to `targets`, the target slot it was chosen for (CR 601.2c): the index of its spec in the
    /// spell's or ability's printed list (a modal item's: in the list of its chosen modes' specs, in printed order, CR
    /// 700.2c). So a slot that takes a variable number of targets ("attach any number of target Equipment you control to
    /// target creature you control") can stand anywhere in the list, and each target is judged by its own slot.
    pub spec_of: [u8; MAX_TARGETS],
    /// The X announced for it (CR 107.3); for a reflexive triggered ability, the number its "that many" counts
    /// (CR 603.12, `PendingTrigger::count`).
    pub x: u16,
    /// How many times it was kicked (CR 702.33).
    pub kicked: u8,
    /// How many times its `optional_cost` was paid (CR 601.2b, 707.10: a copy has the same).
    pub paid: u8,
    /// Which way it was cast (0 for the mana cost, n for the nth alternative cost), or activated.
    pub way: u8,
    /// The modes chosen as it was put on the stack (CR 700.2): a bit for each mode of its modal definition.
    pub modes: u8,
    /// Card code's own values chosen as it was put on the stack: a mode, a pile, a number.
    pub choice: u32,
    /// What the event that triggered it was about (CR 603.2): not a target. For an ability or a permanent spell whose cost
    /// returned an attacking creature to hand (ninjutsu, sneak: CR 702.49a, 702.190b), what that creature was attacking.
    pub subject: Target,
    /// What the event that triggered it carried (CR 603.2): the spell cast, the object that targeted.
    pub data: [u32; 2],
    /// The objects it is about that are not its targets, in `GameState::members`. For a "one or more" ability (CR 603.2c):
    /// the objects of the batch that triggered it. For an activated ability: the permanents its cost chose and tapped,
    /// sacrificed or returned (CR 601.2h by way of 602.2b), which its effect may name ("the tapped creature's power",
    /// station, CR 702.184a); a copy of the ability has the same (CR 707.10).
    pub members: Span,
    /// What names this item while it is on the stack, so that a spell or ability can target it (CR 115.1).
    pub serial: u32,
    /// For a spell cast with gift promised (CR 702.174): the opponent it was promised to.
    pub gift: Option<PlayerId>,
    /// For a spell or ability that divides a total among its targets (CR 601.2d): each target's share, parallel to
    /// `targets`.
    pub shares: [u8; MAX_TARGETS],
    /// For an instant or sorcery spell: if it would be put into a graveyard as it leaves the stack, it is exiled
    /// instead (Bilbo's "cast this way").
    pub exile_after: bool,
    /// For a spell cast with flashback that is not printed in its ways (CR 702.34a): one a permission gave (Snapcaster Mage)
    /// or a permanent's static ability (Iroh, Grand Lotus), or an aftermath half cast from a graveyard (CR 702.127a). It is
    /// exiled instead of put anywhere else any time it would leave the stack.
    pub flashback: bool,
    /// The serial of the first event in the history after it was put on the stack (the history's head then), or, for a
    /// delayed triggered ability, after that ability was created (`Delayed::since`): what an instruction to transform its
    /// source asks the history about (CR 701.27f).
    pub since: u32,
}

impl StackItem {
    pub fn targets(&self) -> &[Target] {
        &self.targets[..self.target_count as usize]
    }

    /// Where in `targets` the targets chosen for the spec at `spec` are (CR 601.2c): one run, in the order chosen. Empty when
    /// that slot chose none, or it is past the targets announced (an unchosen "up to one" at the end of the list).
    pub fn spec_targets(&self, spec: usize) -> std::ops::Range<usize> {
        let of = &self.spec_of[..self.target_count as usize];
        let start = of.iter().position(|&s| s as usize >= spec).unwrap_or(of.len());
        let end = start + of[start..].iter().take_while(|&&s| s as usize == spec).count();
        start..end
    }

    /// What target `slot` was announced to receive of a divided total (CR 601.2d).
    pub fn share(&self, slot: usize) -> u16 {
        self.shares.get(slot).copied().unwrap_or(0) as u16
    }
}

/// A continuous effect from a resolved spell or ability (CR 611), applied in its layer (CR 613).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ContinuousEffect {
    pub source: ObjectId,
    pub controller: PlayerId,
    pub timestamp: u32,
    pub affected: ObjectId,
    pub kind: EffectKind,
    pub duration: Duration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EffectKind {
    /// Protection from these colors (CR 702.16), layer 6.
    Protection(ColorSet),
    /// Protection from colorless (CR 702.16a, 105.2c), layer 6.
    ProtectionFromColorless,
    /// Protection from objects of these card types (CR 702.16a): "protection from creatures", layer 6.
    ProtectionFromTypes(TypeSet),
    /// Protection from everything (CR 702.16e), layer 6.
    ProtectionFromEverything,
    /// +N/+M (CR 613.4c), layer 7c.
    Pump(i16, i16),
    /// Gains these keywords, layer 6.
    AddKeywords(Keywords),
    /// Loses these keywords (CR 613.1f), layer 6, at the effect's timestamp: "permanents your opponents control lose hexproof
    /// and indestructible" (Shadowspear). What is granted after it is kept.
    LoseKeywords(Keywords),
    /// Gains this ability (a triggered or activated ability granted to it), layer 6.
    AddAbility(AbilityRef),
    /// Protection from a player (CR 702.16a): their spells and abilities can't target it, their creatures can't block
    /// it, and damage their sources would deal to it is prevented. Layer 6.
    ProtectionFromPlayer(PlayerId),
    /// Gains these card types (CR 205.1b, 613.1d), layer 4: a crewed Vehicle is an artifact creature, and Tezzeret's
    /// emblem makes an artifact a Robot creature.
    AddTypes(TypeSet),
    /// Its controller is this player (CR 613.1b, layer 2). The latest of an object's control effects applies.
    Control(PlayerId),
    /// "Attacks each combat if able" (CR 508.1d): an attack requirement.
    MustAttack,
    /// "Attacks that player this combat if able" (CR 508.1d): an attack requirement that names the player to attack. It also
    /// makes the creature attack each combat if able, for as long as it lasts.
    MustAttackPlayer(PlayerId),
    /// It becomes this subtype in addition to its other subtypes (CR 613.1d), layer 4.
    AddSubtype(Subtype),
    /// Its creature type becomes this one (CR 205.1a, 613.1d), layer 4: every creature type it had is replaced, its card types
    /// and its other subtypes (artifact types, land types) stay: "becomes a green Wurm" (Scale Up).
    SetCreatureType(Subtype),
    /// It gains all creature types (CR 205.3m, 613.1d), layer 4, in addition to its other subtypes: "becomes a 2/2
    /// creature with all creature types" (Mutavault, Soulstone Sanctuary). Only a creature or a Kindred can have creature
    /// types (CR 205.3d), so it gives them to an object that is one by its timestamp. It is not Changeling: the object
    /// gains no ability (the Mutavault ruling).
    AddAllCreatureTypes,
    /// It loses all creature types (CR 205.3m, 613.1d), layer 4, at its timestamp: its other subtypes and its card types stay,
    /// and a creature type an effect gives it later is had (the Nameless Inversion rulings). Changeling's types are taken
    /// too, the ability having applied first (CR 613.3); the ability itself stays.
    LoseAllCreatureTypes,
    /// Its power and toughness are switched (CR 613.4d), layer 7d: applied after every setting and modifying effect and
    /// the counters.
    SwitchPt,
    /// Its power and toughness become these (CR 613.4b), layer 7b: "it becomes a 0/0 creature".
    SetPt(i16, i16),
    /// Is exactly these card types (CR 613.1d), layer 4; the subtypes it had belonged to the types it lost.
    SetTypes(TypeSet),
    /// Is exactly these colors (CR 105.3, 613.1e), layer 5.
    SetColors(ColorSet),
    /// "Loses all abilities" (CR 613.1f), layer 6: at its timestamp, so what is granted later is kept.
    LoseAllAbilities,
    /// Goaded by the effect's controller (CR 701.15a): attacks each combat if able, and attacks a player other than
    /// the goader if able (CR 701.15b).
    Goaded,
    /// "Can't block" (CR 509.1b) for as long as the effect lasts.
    CantBlock,
    /// "Can't attack" (CR 508.1c) for as long as the effect lasts: a restriction on declaring the creature as an attacker.
    CantAttack,
    /// The permanent is harnessed (CR 701.64): a designation it keeps as long as it stays where it is
    /// (`Duration::Indefinite`), which turns its infinity abilities on (CR 702.186).
    Harnessed,
    /// The permanent is suspected (CR 701.60): a designation it keeps until it leaves the battlefield or an effect ends it
    /// (`Duration::Indefinite`), neither an ability nor a copiable value (CR 701.60b). While it lasts the permanent has
    /// menace and "This creature can't block" (CR 701.60c), given in layer 6 at the effect's timestamp, so "loses all
    /// abilities" later takes them though it stays suspected (the rulings).
    Suspected,
    /// The effect's controller has hexproof (CR 702.11c): their opponents' spells and abilities can't target them. It
    /// is about the player and outlives its `affected` object, which is only the source.
    PlayerHexproof,
    /// The effect's controller has protection from everything (CR 702.16j): nothing can target them, whoever controls it,
    /// and damage from any source is prevented. About the player, not its `affected` object, which is only the source.
    PlayerProtectionFromEverything,
    /// "You may cast [spells of these types] as though they had flash" (CR 307.5, 702.8): the effect's controller may
    /// cast them whenever they could cast an instant. Outlives its `affected` object, the source.
    CastAsThoughFlash(TypeSet),
    /// "You may activate loyalty abilities of [Jace] planeswalkers you control on any player's turn any time you could cast
    /// an instant" (CR 606.3, 611.2c; Jace's Machinations): the effect's controller may activate the loyalty abilities of the
    /// planeswalkers they control that have this planeswalker type (every planeswalker for `None`) whenever they have
    /// priority. Only the timing changes: one loyalty ability per permanent each turn still holds. A rule about the player,
    /// so it reaches planeswalkers that arrive later and outlives its `affected` object, which is only the source.
    /// It is in the core, though one printed card has it as a resolved effect, because it must outlive the spell, belong to
    /// the player and reach a planeswalker that does not exist yet, and only the core checks timing. The other cards that
    /// grant this (Teferi, Master of Time; The Wandering Emperor; the Teferi emblems) are statics: `CardDef.instant_activation`.
    InstantLoyalty(Option<Subtype>),
    /// The effect's controller has no maximum hand size (CR 402.2) for as long as it lasts: Spirit Water Revival's "for the rest
    /// of the game" (`Duration::Indefinite`). About the player, so it outlives its `affected` object, which is only the source.
    NoMaximumHandSize,
    /// The player can't gain life (CR 119.7): Screaming Nemesis's "for the rest of the game" (`Duration::Indefinite`). It is
    /// about the player and outlives its `affected` object, which is only the source.
    CantGainLife(PlayerId),
    /// The player can't cast spells from these zones (CR 101.2, 601.3, 611.2c): Avatar's Wrath's "until your next turn, your
    /// opponents can't cast spells from anywhere other than their hands", one effect for each player affected, who are the
    /// opponents as it resolves. Only casting is stopped: an activated ability is no spell, and a copy of a spell put onto
    /// the stack is not cast (CR 707.10). It is about the player, so it reaches cards that arrive in those zones later, and
    /// outlives its `affected` object, which is only the source. A static ability with the same words (Drannith Magistrate)
    /// is a `CardDef::cast_rules` entry, which is told the zone too.
    CantCastSpellsFrom(PlayerId, ZoneKinds),
    /// The creature can attack as though it didn't have defender (CR 702.3b, 609.4) for as long as the effect lasts.
    CanAttackDespiteDefender,
    /// "Creatures [this player] controls can't block this turn" (CR 509.1b, 611.2c): a rule about a player, so it reaches creatures
    /// that arrive later too and grants no ability. Its `affected` object is only the source.
    CreaturesCantBlock(PlayerId),
    /// A regeneration shield (CR 701.19a, 614.8): the next time the permanent would be destroyed this turn, it is
    /// tapped, its damage removed and it is removed from combat instead. One effect for each shield.
    RegenerationShield,
    /// A replacement effect (CR 611.2, 614) a resolving spell or ability made about its `affected` object, for as long
    /// as its duration lasts: the nth of the source card's `granted_replacements`. It ends with the object, which is a
    /// new one when it changes zones (CR 400.7): "if that creature would die this turn, exile it instead".
    Replacement(u8),
    /// The same, about a player or the game and not an object ("if a card would be put into your graveyard from
    /// anywhere this turn, exile it instead"): its `affected` object is only the source, and it outlives it.
    ReplacementRule(u8),
    /// "You may play lands and cast spells from your graveyard" (CR 305.1, 601.3) for the effect's controller. About the
    /// player, and outlives its `affected` object, which is only the source.
    PlayFromGraveyard,
    /// A mana ability the effect's controller has of their own until the effect ends (CR 605.1a, 611.2a): an ability of the
    /// source card's `delayed` list. About the player, so it outlives its `affected` object, which is only the source (Channel).
    PlayerManaAbility(AbilityRef),
    /// Marks that a copy effect until end of turn (CR 707.2, 611.2b) changed the `affected` permanent's copiable values, and
    /// holds what they were: put back when the effect ends. The copy itself is in `Obj::copy`.
    RestoreCopy(Option<Copiable>),
    /// A text-changing effect (CR 612, layer 3): every instance of `from` in the object's rules text and type line is
    /// `to` (Trait Doctoring, Artificial Evolution). Card code reads its words through `Query::words`; the layers change
    /// the type line.
    ChangeText { from: Word, to: Word },
    /// "[It] becomes ... named [name]" (CR 612.8): a text-changing effect (layer 3) that sets the object's name, so it
    /// loses any names it had and has only this one, for every rule and effect that asks its name (the legend rule,
    /// "with the same name", a chosen name). Its rules text is not changed: text that refers to the object by name still
    /// means the object (CR 201.5). The Irencrag's "become a legendary Equipment artifact named Everflame, Heroes'
    /// Legacy".
    SetName(&'static str),
    /// "The next spell you cast this turn can't be countered" (CR 611.2c, 601.2, 701.5a; Mistrise Village): a rule about the
    /// effect's controller that reaches a spell that does not exist yet, so it outlives its `affected` object, which is only the
    /// source. It marks the next spell that player casts, as it is cast and before anyone has priority, and is used up by it.
    /// It is in the core, though one printed card has it, because only the core is there while a spell is being cast: a
    /// card file can't mark a spell during casting, and a delayed trigger would go on the stack above the spell and could
    /// be answered. A kind of spell ("the next creature spell") is a filter to add when a printed card needs one.
    NextSpellCantBeCountered,
    /// "[This player] skips their next turn" (CR 500.11, 614.10): a replacement effect of the beginning of that player's
    /// next turn, used up by the turn it skips. "Skips their next N turns" is N of them, each waiting for the first turn
    /// the ones before it have not skipped (CR 614.10a). About the player, so it outlives its `affected` object, which is
    /// only the source.
    SkipNextTurn(PlayerId),
}

impl EffectKind {
    /// Whether the effect is about a player or a rule, not its `affected` object: it goes on when that object is gone
    /// (CR 611.2c).
    pub fn outlives_affected(self) -> bool {
        matches!(
            self,
            EffectKind::PlayerHexproof
                | EffectKind::PlayerProtectionFromEverything
                | EffectKind::CastAsThoughFlash(_)
                | EffectKind::InstantLoyalty(_)
                | EffectKind::NoMaximumHandSize
                | EffectKind::CantGainLife(_)
                | EffectKind::CantCastSpellsFrom(..)
                | EffectKind::ReplacementRule(_)
                | EffectKind::CreaturesCantBlock(_)
                | EffectKind::PlayFromGraveyard
                | EffectKind::PlayerManaAbility(_)
                | EffectKind::NextSpellCantBeCountered
                | EffectKind::SkipNextTurn(_)
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Duration {
    /// Until the cleanup step (CR 514.2).
    EndOfTurn,
    /// As long as the affected object stays where it is.
    Indefinite,
    /// "For as long as [the source] remains on the battlefield" (CR 611.2b): it ends when the effect's source is no
    /// longer the permanent it was, on the battlefield.
    WhileSourceOnBattlefield,
    /// Until the next turn of this player begins (CR 611.2b): "until your next turn".
    UntilNextTurnOf(PlayerId),
    /// "Until this card is cast from exile" (CR 611.2a, 400.7): until this object, a card in exile, becomes cast (CR
    /// 601.2i) - after its costs are paid, so a mana ability the effect grants may help pay for it. A card that leaves
    /// exile any other way is a new object (CR 400.7) that can never be cast as this one, so the effect then lasts as long
    /// as its affected object does. Emrakul, the Exigent Doom and Streets of New Capenna's Glamorous Outlaw cycle.
    UntilCastFromExile(ObjectId),
}

/// When a permission to play a card ends (CR 611.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Expiry {
    /// For as long as the card stays where it is: a card that changes zones is a new object (CR 400.7).
    Never,
    /// "This turn": the cleanup step (CR 514.2).
    EndOfTurn,
    /// "Until the end of your next turn": the cleanup step of the permitted player's next turn.
    EndOfYourNextTurn,
    /// "Until your next end step" (CR 500.4): ends as the holder's next end step begins, before its triggers.
    UntilYourNextEndStep,
}

/// What a permission to play a card is, apart from who holds it and for which card.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlayGrant {
    /// The cost paid instead of the card's mana cost (CR 118.9): `Some(ManaCost::ZERO)` casts it for free.
    pub instead: Option<ManaCost>,
    /// "Play", not "cast" (CR 305.1): a land among them may be played as a land play too.
    pub lands: bool,
    pub expiry: Expiry,
    /// Mana of any type can be spent to cast it (CR 609.4b, 118.14): every symbol of its cost is paid as generic.
    pub any_type: bool,
    /// Generic mana added to what casting it costs (CR 601.2f): Elite Spellbinder's "costs {2} more to cast".
    pub more: u8,
    /// It may be played only on a turn after the one it was granted on: warp's "cast this card from exile on a later
    /// turn" (CR 702.185a).
    pub later_turns: bool,
    /// An instant or sorcery cast this way is exiled instead of put anywhere else as it leaves the stack (CR 702.34a):
    /// a flashback granted by Snapcaster Mage or Flashback.
    pub exile_after: bool,
    /// It may be played only while this holds (CR 611.2, 601.3): "during any turn you attacked with a Rogue".
    pub when: Option<PlayCondition>,
    /// It is cast only at sorcery speed, whatever it is (CR 702.170d): plot's "cast it as a sorcery".
    pub sorcery_speed: bool,
}

/// A condition on a standing permission to play a card, worked out whenever the permission is asked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayCondition {
    /// A creature of this type was declared as an attacker by the permitted player this turn (CR 508.1): Robber of the Rich.
    AttackedWithThisTurn(Subtype),
}

impl PlayGrant {
    /// "You may cast it": for as long as it stays where it is, for its mana cost.
    pub const CAST: PlayGrant = PlayGrant { instead: None, lands: false, expiry: Expiry::Never, any_type: false, more: 0, later_turns: false, exile_after: false, when: None, sorcery_speed: false };
}

/// A standing permission to play or cast one card from a zone it could not be played from (CR 601.3, 305.1): "until
/// the end of your next turn, you may play that card", airbend's "may cast it for {2}".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlayPermission {
    pub card: ObjectId,
    pub player: PlayerId,
    pub grant: PlayGrant,
    /// The turn it was granted on, for `Expiry::EndOfYourNextTurn`.
    pub turn: u32,
}

/// A triggered ability that has triggered and waits to be put on the stack (CR 603.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PendingTrigger {
    pub source: ObjectId,
    pub ability: AbilityRef,
    pub controller: PlayerId,
    /// What the event was about (the player or object "it" and "that player" refer to): no target, never checked.
    pub subject: Target,
    pub data: [u32; 2],
    /// The batch it was triggered by, for a "one or more" ability (CR 603.2c).
    pub members: Span,
    /// For a reflexive triggered ability whose effect counts what its first part did ("tap up to that many target
    /// creatures", CR 603.12): that number. A `TargetSpec::COUNTED` bound reads it, and it is the stack item's `x`.
    pub count: Option<u16>,
    /// For a delayed triggered ability, the history's head when it was created (`Delayed::since`), which its stack item
    /// keeps (CR 701.27f); none for any other, whose item takes the head as it is put on the stack.
    pub since: Option<u32>,
}

/// A run of `GameState::members`: the objects of one batch of simultaneous events, kept for the ability they
/// triggered (CR 603.2c, 608.2h).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Span {
    pub start: u32,
    pub len: u16,
}

/// A delayed triggered ability waiting for its moment (CR 603.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Delayed {
    pub when: DelayedWhen,
    pub source: ObjectId,
    pub controller: PlayerId,
    /// The ability that triggers: a card's delayed ability (`AbilityRef::delayed`).
    pub ability: AbilityRef,
    pub subject: Target,
    /// The objects the ability is about (CR 603.7c), kept in `GameState::members`: a mobilize ability's Warriors, which it
    /// sacrifices together. `StackItem::members` once it triggers.
    pub members: Span,
    /// The history's head when it was created: "since that delayed triggered ability was created" (CR 701.27f).
    pub since: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DelayedWhen {
    /// At the beginning of the next such step, of anyone's turn or one player's. Once.
    Step { step: Step, on_turn_of: Option<PlayerId> },
    /// "At the beginning of each [step] ... for the rest of the game" (CR 603.7, 702.192a): at every such step, of anyone's
    /// turn or one player's, and never used up. It is no ability of an object, so it goes on whatever happens to the object
    /// it came from (paradigm's ruling: "it doesn't matter what happens to the card in exile").
    EachStep { step: Step, on_turn_of: Option<PlayerId> },
    /// "Whenever [event] this turn" (CR 603.7b): triggers each time the ability's own `listens`/`matches` accept an
    /// event, until the cleanup step ends the turn. The ability's `matches` is asked of `source`.
    ThisTurn,
    /// "Until [event]" (CR 610.3, 603.7): triggers once, the next time its `listens`/`matches` accept an event, whenever that is, and is
    /// then used up. The ability's `matches` is asked of `source`, which may be gone by then.
    UntilEvent,
    /// "Until your next turn, whenever [event]" (CR 603.7b, 611.2b): as `ThisTurn`, until that player's next turn begins.
    UntilNextTurnOf(PlayerId),
}

impl DelayedWhen {
    /// Whether it triggers on events, as many times as they happen, rather than once at a step.
    pub fn is_event(self) -> bool {
        matches!(self, DelayedWhen::ThisTurn | DelayedWhen::UntilEvent | DelayedWhen::UntilNextTurnOf(_))
    }
}

/// An object's characteristics as it last existed on the battlefield (CR 608.2h, 603.10), kept when it leaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LastKnown {
    pub id: ObjectId,
    pub types: TypeSet,
    pub subtypes: SubtypeSet,
    pub colors: ColorSet,
    pub power: Option<i16>,
    pub toughness: Option<i16>,
    pub keywords: Keywords,
    /// Its abilities as it last had them, those granted by effects included: a leaves-the-battlefield ability it was given still
    /// triggers (CR 603.10a).
    pub abilities: [Option<AbilityRef>; 8],
    /// The words of its text as it last had them (CR 612): an ability of it that resolves after it has gone reads them.
    pub words: Words,
}

/// A player (CR 102).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Player {
    pub life: i32,
    /// The life total the player began the game with (CR 103.4): "half your starting life total".
    pub starting_life: i32,
    pub pool: ManaPool,
    /// How much of each kind in the pool lasts until the end of combat (CR 106.4, 500.5a): firebending's mana. Spent
    /// last, since the ordinary mana of a kind is spent first, which is never worse for the player; a count above
    /// what the pool holds is stale, and `lasting_mana` clamps it.
    pub lasting: ManaPool,
    pub lands_played: u8,
    /// Drew from an empty library since the last state-based action check (CR 704.5b).
    pub drew_from_empty: bool,
    pub lost: bool,
    pub mulligans: u8,
    pub kept: bool,
    /// The number of the turn this player's most recent turn was.
    pub turn_began: u32,
    /// Turns this player has begun (CR 500): "your first three turns of the game".
    pub turns_taken: u16,
    /// Poison counters (CR 122.1f): ten lose the game (CR 704.5c).
    pub poison: u8,
    /// Energy counters (CR 107.14, 122.1): a player's own store, gained and paid.
    pub energy: u8,
    /// Experience counters (CR 122.1): a player's own store, only ever gained.
    pub experience: u8,
    /// The city's blessing (CR 702.131): had for the rest of the game once gained.
    pub citys_blessing: bool,
    /// An enduring story (CR 702.195b): had for the rest of the game once gained.
    pub enduring_story: bool,
    /// The monarch (CR 725): at most one player is, once a game has had one.
    pub monarch: bool,
    /// The player with the initiative (CR 726): at most one.
    pub initiative: bool,
    /// The room of the Undercity this player's venture marker is on (CR 309.4, 701.49): 0 for none, else the room's
    /// index plus one.
    pub room: u8,
    /// Dungeons this player has completed (CR 309.7).
    pub dungeons_completed: u8,
}

/// Mana in a player's pool with a restriction on what it may be spent on (CR 106.6): one entry per player and
/// restriction, kept in `GameState::restricted_mana` apart from the player, so that a player stays plain data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RestrictedMana {
    pub player: PlayerId,
    pub restriction: ManaRestriction,
    pub pool: ManaPool,
}

/// A limit on what some mana may be spent on (CR 106.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ManaRestriction {
    /// "This mana can't be spent to cast nonartifact spells": it pays artifact spells and everything that is not a
    /// spell.
    NotNonartifactSpells,
    /// "If that mana is spent on a creature spell, it gains haste until end of turn" (CR 106.6; Arena of Glory): it may
    /// be spent on anything, and does that to a creature spell it pays for.
    GainsHaste,
    /// "Spend this mana only to cast a legendary spell, and that spell can't be countered" (CR 106.6, 701.5; Delighted
    /// Halfling): a limit and an effect, each independent of the other, carried by the mana itself.
    LegendaryUncounterable,
    /// "Spend this mana only to cast [type] spells" (CR 106.6): only on casting a spell (CR 601.2g) that has one of these
    /// card types - not on an ability, nor on any other spell, nor on a cost paid for a spell that is not casting it
    /// ("unless its controller pays", CR 118.12). A spell with flashback or kicker is still the spell it is.
    /// Artifact spells: Mishra's Workshop, Steelswarm Operator. Instant or sorcery spells: Great Hall of the Biblioplex,
    /// Tablet of Discovery (and the same limit on Abstract Paintmage and Cormela, Glamour Thief). Planeswalker spells:
    /// Gideon's Memorial.
    Spells(TypeSet),
    /// "Spend this mana only to activate abilities [of [type] sources]" (CR 106.6, 602.2b): only on the cost of activating
    /// an activated ability - a mana ability's included (CR 605.3a), wherever the ability functions from (a land card's
    /// channel or cycling in the hand) - whose source has one of these card types as the ability is activated; with no
    /// types, of any source. Never on a spell, a special action (CR 116.2: turning a face-down permanent face up,
    /// unlocking a door), a triggered ability's cost or "unless ... pays" (CR 118.12).
    /// Any source: Omen Hawker, Pit Automaton, The Enigma Jewel. Land sources: Sunken Citadel, Eumidian Lifeseed.
    /// Artifact sources: Soldevi Machinist ("of artifacts"), Steelswarm Operator.
    Abilities(TypeSet),
    /// "Spend this mana only to cast [a] [subtype] [or [subtype]] [creature] spell[s][, and that spell can't be
    /// countered]" (CR 106.6, 205.3): only on casting a spell that has one of these subtypes as it is on the stack (its
    /// characteristics there, CR 601.2g: a changeling creature spell is every creature type, CR 702.73a; a type an effect
    /// gave it or took away counts). See `SubtypeSpells`.
    Subtypes(SubtypeSpells),
    /// "Spend this mana only to cast [a] non[type] spell[s]" (CR 106.6, 205.2a): only on casting a spell (CR 601.2g) that
    /// has none of these card types as it is on the stack - an artifact creature spell is a creature spell, so no
    /// noncreature spell. Never on an ability. Noncreature: Chandra, Chill of Compliance, Narset of the Ancient Way, The
    /// Emperor of Palamecia, Immortus, Master of Eternity, Nardole, Resourceful Cyborg.
    NonSpells(TypeSet),
    /// "Spend this mana only to cast spells with mana value N or greater [or spells with {X} in their mana costs]"
    /// (CR 106.6, 202.3): only on casting a spell whose mana value on the stack is at least N - with each {X} the number
    /// chosen for it (CR 202.3e, 107.3), a face-down spell's 0 (CR 708.4), the cast half's alone (CR 709.3b) - or, where it
    /// says so, whose mana cost has {X} in it. See `ManaValueSpells`.
    ManaValue(ManaValueSpells),
}

/// "Spend this mana only to cast spells with mana value N or greater [or spells with {X} in their mana costs]"
/// (CR 106.6, 202.3, 107.3). Mana value 4 or greater: Ashling, Rimebound. Mana value 5 or greater or {X}: Troyan, Gutsy
/// Explorer. (Helga, Skittish Seer's is for creature spells only, which this does not say.)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ManaValueSpells {
    /// The least mana value the spell may have.
    pub at_least: u8,
    /// "... or spells with {X} in their mana costs": a spell whose mana cost has an {X}, whatever its mana value.
    pub or_x: bool,
}

impl ManaValueSpells {
    /// Whether a spell of this mana value, with or without {X} in its mana cost, may be paid for with the mana.
    fn admits(self, mana_value: u32, x: bool) -> bool {
        mana_value >= self.at_least as u32 || (self.or_x && x)
    }
}

/// "Spend this mana only to cast [a] [subtype] [or [subtype]] [creature] spell[s]" (CR 106.6, 205.3).
/// Dragon spells: Desolation of Smaug, Sarkhan, Fireblood. A Dragon spell or an Omen spell: Maelstrom of the Spirit
/// Dragon. A creature spell of the chosen type: Unclaimed Territory, Pillar of Origins, and Cavern of Souls, whose spell
/// can't be countered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SubtypeSpells {
    /// The subtypes, any one of which the spell must have. Read through the words of the text that names them (CR 612)
    /// as the mana is added.
    pub subtypes: [Option<Subtype>; 2],
    /// "Of the chosen type" (CR 205.3m, 607.2a): as the mana is added, `subtypes` becomes the type chosen for the source
    /// as it entered, or nothing at all if none was, which then no spell has.
    pub chosen: bool,
    /// Only a creature spell (CR 302): "a creature spell of the chosen type", "Dragon creature spells".
    pub creature: bool,
    /// "... and that spell can't be countered" (CR 106.6, 701.6): the effect the mana has on the spell it pays for.
    pub uncounterable: bool,
}

impl SubtypeSpells {
    /// "Spend this mana only to cast a creature spell of the chosen type."
    pub const CHOSEN_TYPE_CREATURE: SubtypeSpells =
        SubtypeSpells { subtypes: [None, None], chosen: true, creature: true, uncounterable: false };

    /// "Spend this mana only to cast [subtype] spells."
    pub const fn of(subtype: Subtype) -> SubtypeSpells {
        SubtypeSpells { subtypes: [Some(subtype), None], chosen: false, creature: false, uncounterable: false }
    }

    /// "Spend this mana only to cast a [subtype] spell or a [subtype] spell."
    pub const fn either(a: Subtype, b: Subtype) -> SubtypeSpells {
        SubtypeSpells { subtypes: [Some(a), Some(b)], chosen: false, creature: false, uncounterable: false }
    }

    /// "... and that spell can't be countered."
    pub const fn uncounterable(self) -> SubtypeSpells {
        SubtypeSpells { uncounterable: true, ..self }
    }

    /// Whether a spell with these characteristics may be paid for with the mana.
    fn admits(self, types: TypeSet, subtypes: &SubtypeSet) -> bool {
        (!self.creature || types.has(CardType::Creature)) && self.subtypes.iter().flatten().any(|&s| subtypes.has(s))
    }
}

/// What some mana is being spent on, as far as a restriction on it is concerned (CR 106.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManaPayment {
    /// Casting a spell with these card types (CR 601.2g-h). Its subtypes are worked out only where some mana at hand is
    /// restricted by subtype (`ManaRestriction::Subtypes`), and are empty otherwise. `mana_value` is its mana value on the
    /// stack (CR 202.3), and `x` whether its mana cost has {X} in it (CR 107.3); for the advice whether a card could be
    /// cast, its mana value with X at 0, which `x` says could be raised.
    Spell { types: TypeSet, subtypes: SubtypeSet, legendary: bool, mana_value: u32, x: bool },
    /// Activating an ability whose source has these card types (CR 602.2b, 605.3a).
    Ability { source: TypeSet },
    /// Anything else: a special action, "unless ... pays", a cost a resolving effect asks for.
    Other,
}

/// What mana spent on a spell does to it (CR 106.6), as bits of `ManaRestriction::effects`: it gains haste, it can't be
/// countered.
pub const MANA_GIVES_HASTE: u8 = 1;
pub const MANA_MAKES_UNCOUNTERABLE: u8 = 2;

impl ManaRestriction {
    pub const ARTIFACT_SPELLS: ManaRestriction = ManaRestriction::Spells(TypeSet::of(&[CardType::Artifact]));
    pub const INSTANT_OR_SORCERY_SPELLS: ManaRestriction =
        ManaRestriction::Spells(TypeSet::of(&[CardType::Instant, CardType::Sorcery]));
    pub const PLANESWALKER_SPELLS: ManaRestriction = ManaRestriction::Spells(TypeSet::of(&[CardType::Planeswalker]));
    pub const ABILITIES: ManaRestriction = ManaRestriction::Abilities(TypeSet::NONE);
    pub const LAND_ABILITIES: ManaRestriction = ManaRestriction::Abilities(TypeSet::of(&[CardType::Land]));
    pub const ARTIFACT_ABILITIES: ManaRestriction = ManaRestriction::Abilities(TypeSet::of(&[CardType::Artifact]));
    pub const NONCREATURE_SPELLS: ManaRestriction = ManaRestriction::NonSpells(TypeSet::of(&[CardType::Creature]));

    /// "Spend this mana only to cast spells with mana value `n` or greater."
    pub const fn mana_value_at_least(n: u8) -> ManaRestriction {
        ManaRestriction::ManaValue(ManaValueSpells { at_least: n, or_x: false })
    }

    /// "Spend this mana only to cast spells with mana value `n` or greater or spells with {X} in their mana costs."
    pub const fn mana_value_at_least_or_x(n: u8) -> ManaRestriction {
        ManaRestriction::ManaValue(ManaValueSpells { at_least: n, or_x: true })
    }

    /// The restrictions with no subtype or number in them, each with the name a scenario and the dump write it by. Those
    /// of `Subtypes` are named for their subtypes and those of `ManaValue` for their number (`name`). A card naming a
    /// restriction of card types not here fails the build (`Adds::only_for` is checked at compile time), since the host
    /// has no word for it on the wire.
    pub const NAMED: [(ManaRestriction, &'static str); 10] = [
        (ManaRestriction::NotNonartifactSpells, "NotNonartifactSpells"),
        (ManaRestriction::GainsHaste, "GainsHaste"),
        (ManaRestriction::LegendaryUncounterable, "LegendarySpells, CantBeCountered"),
        (ManaRestriction::ARTIFACT_SPELLS, "ArtifactSpells"),
        (ManaRestriction::INSTANT_OR_SORCERY_SPELLS, "InstantOrSorcerySpells"),
        (ManaRestriction::PLANESWALKER_SPELLS, "PlaneswalkerSpells"),
        (ManaRestriction::ABILITIES, "Abilities"),
        (ManaRestriction::LAND_ABILITIES, "LandAbilities"),
        (ManaRestriction::ARTIFACT_ABILITIES, "ArtifactAbilities"),
        (ManaRestriction::NONCREATURE_SPELLS, "NoncreatureSpells"),
    ];

    const fn same(self, other: ManaRestriction) -> bool {
        match (self, other) {
            (ManaRestriction::NotNonartifactSpells, ManaRestriction::NotNonartifactSpells)
            | (ManaRestriction::GainsHaste, ManaRestriction::GainsHaste)
            | (ManaRestriction::LegendaryUncounterable, ManaRestriction::LegendaryUncounterable) => true,
            (ManaRestriction::Spells(a), ManaRestriction::Spells(b))
            | (ManaRestriction::Abilities(a), ManaRestriction::Abilities(b))
            | (ManaRestriction::NonSpells(a), ManaRestriction::NonSpells(b)) => a.0 == b.0,
            _ => false,
        }
    }

    /// Whether the wire has a word for it: one of `NAMED`, or any of `Subtypes` or `ManaValue`.
    pub const fn is_known(self) -> bool {
        if let ManaRestriction::Subtypes(_) | ManaRestriction::ManaValue(_) = self {
            return true;
        }
        let mut i = 0;
        while i < ManaRestriction::NAMED.len() {
            if self.same(ManaRestriction::NAMED[i].0) {
                return true;
            }
            i += 1;
        }
        false
    }

    /// Whether it limits mana by the subtypes of the spell it pays for, which only then need working out.
    pub fn names_subtypes(self) -> bool {
        matches!(self, ManaRestriction::Subtypes(_))
    }

    /// The least mana value of a spell it pays for, where it limits mana by that (`ManaValue`): what a spell with {X}
    /// still to be announced could reach (advice).
    pub fn least_mana_value(self) -> Option<u8> {
        match self {
            ManaRestriction::ManaValue(m) => Some(m.at_least),
            _ => None,
        }
    }

    /// Whether mana under this restriction may be spent on `payment` (CR 106.6).
    pub fn may_pay(self, payment: &ManaPayment) -> bool {
        match (self, *payment) {
            (ManaRestriction::NonSpells(barred), ManaPayment::Spell { types, .. }) => barred.0 & types.0 == 0,
            (ManaRestriction::ManaValue(m), ManaPayment::Spell { mana_value, x, .. }) => m.admits(mana_value, x),
            // "Can't be spent to cast nonartifact spells": everything but casting a nonartifact spell.
            (ManaRestriction::NotNonartifactSpells, ManaPayment::Spell { types, .. }) => types.has(CardType::Artifact),
            (ManaRestriction::NotNonartifactSpells, _) => true,
            (ManaRestriction::GainsHaste, _) => true,
            (ManaRestriction::LegendaryUncounterable, ManaPayment::Spell { legendary, .. }) => legendary,
            (ManaRestriction::Spells(allowed), ManaPayment::Spell { types, .. }) => allowed.0 & types.0 != 0,
            (ManaRestriction::Abilities(allowed), ManaPayment::Ability { source }) => allowed.0 == 0 || allowed.0 & source.0 != 0,
            (ManaRestriction::Subtypes(s), ManaPayment::Spell { types, ref subtypes, .. }) => s.admits(types, subtypes),
            _ => false,
        }
    }

    /// What spending this mana on a spell does to it (CR 106.6): `MANA_GIVES_HASTE`, `MANA_MAKES_UNCOUNTERABLE`.
    pub fn effects(self) -> u8 {
        match self {
            ManaRestriction::GainsHaste => MANA_GIVES_HASTE,
            ManaRestriction::LegendaryUncounterable => MANA_MAKES_UNCOUNTERABLE,
            ManaRestriction::Subtypes(s) if s.uncounterable => MANA_MAKES_UNCOUNTERABLE,
            _ => 0,
        }
    }

    /// The name a scenario's `Spend:` and the state dump write it by: the limit and the effect, as the C# engine does.
    /// One of `Subtypes` is its subtypes (spaces left out) and "Spells" ("DragonSpells", "DragonOrOmenSpells", "ElfCreatureSpells"),
    /// "NoTypeCreatureSpells" for a chosen type never chosen, with ", CantBeCountered" after it for its effect.
    pub fn name(self) -> String {
        if let ManaRestriction::ManaValue(m) = self {
            return format!("ManaValue{}OrGreater{}Spells", m.at_least, if m.or_x { "OrX" } else { "" });
        }
        if let ManaRestriction::Subtypes(s) = self {
            let types: Vec<String> = s.subtypes.iter().flatten().map(|t| t.name().replace(' ', "")).collect();
            let mut name = if types.is_empty() { "NoType".to_string() } else { types.join("Or") };
            if s.chosen {
                name = "ChosenType".to_string();
            }
            name.push_str(if s.creature { "CreatureSpells" } else { "Spells" });
            if s.uncounterable {
                name.push_str(", CantBeCountered");
            }
            return name;
        }
        ManaRestriction::NAMED.iter().find(|(r, _)| *r == self).map_or_else(|| format!("{self:?}"), |(_, n)| n.to_string())
    }

    /// Reads the words of a `Spend:` piece after the mana symbol: each a name of the limit or the effect.
    pub fn parse<'a>(words: impl Iterator<Item = &'a str>) -> Option<ManaRestriction> {
        let text: Vec<String> = words.map(|w| w.trim_matches(',').to_ascii_lowercase()).filter(|w| !w.is_empty()).collect();
        let matches = |name: &str| {
            let own: Vec<String> = name.split(',').map(|w| w.trim().to_ascii_lowercase()).collect();
            own.len() == text.len() && own.iter().all(|w| text.contains(w))
        };
        if let Some((r, _)) = ManaRestriction::NAMED.iter().find(|(_, n)| matches(n)) {
            return Some(*r);
        }
        // A mana value restriction: "ManaValue4OrGreaterSpells", "ManaValue5OrGreaterOrXSpells".
        if let [one] = text.as_slice()
            && let Some(rest) = one.strip_prefix("manavalue")
        {
            let (n, or_x) = match rest.strip_suffix("orgreaterorxspells") {
                Some(n) => (n, true),
                None => (rest.strip_suffix("orgreaterspells")?, false),
            };
            return Some(ManaRestriction::ManaValue(ManaValueSpells { at_least: n.parse().ok()?, or_x }));
        }
        // A subtype restriction: "DragonSpells", "DragonOrOmenSpells", "ElfCreatureSpells", with "CantBeCountered".
        let uncounterable = text.iter().any(|w| w == "cantbecountered");
        if text.len() != 1 + uncounterable as usize {
            return None;
        }
        let limit = text.iter().find(|w| *w != "cantbecountered")?;
        let (body, creature) = match limit.strip_suffix("creaturespells") {
            Some(body) => (body, true),
            None => (limit.strip_suffix("spells")?, false),
        };
        // One subtype's name, or two joined by "Or": read against the names themselves, since a name may hold "or" (Orc,
        // Warrior, Horror).
        let named = |text: &str| Subtype::ALL.into_iter().find(|t| t.name().replace(' ', "").eq_ignore_ascii_case(text));
        let subtypes = if body == "notype" {
            [None, None]
        } else if let Some(one) = named(body) {
            [Some(one), None]
        } else {
            Subtype::ALL.into_iter().find_map(|first| {
                let rest = body.strip_prefix(first.name().replace(' ', "").to_ascii_lowercase().as_str())?.strip_prefix("or")?;
                Some([Some(first), Some(named(rest)?)])
            })?
        };
        Some(ManaRestriction::Subtypes(SubtypeSpells { subtypes, chosen: false, creature, uncounterable }))
    }
}

impl Player {
    /// Spends `n` mana of one kind from the pool, the ordinary mana of that kind before the mana that lasts.
    /// The mana in the pool that lasts until the end of combat.
    pub fn lasting_mana(&self) -> ManaPool {
        let mut lasting = self.lasting;
        for (l, p) in lasting.0.iter_mut().zip(self.pool.0) {
            *l = (*l).min(p);
        }
        lasting
    }

    pub fn spend_mana(&mut self, mana: Mana, n: u8) {
        let at = mana as usize;
        self.pool.0[at] -= n;
        self.lasting.0[at] = self.lasting.0[at].min(self.pool.0[at]);
    }
}

/// Where the turn is (CR 500).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Turn {
    pub number: u32,
    pub active: PlayerId,
    pub step_raw: u8,
    pub priority: PlayerId,
    /// Players who have passed in succession with nothing new on the stack (CR 117.4).
    pub passes: u8,
    /// Whether the step's turn-based actions have been done.
    pub began: bool,
    /// The player who took the first turn of the game, who skips their first draw (CR 103.8a).
    pub starting: PlayerId,
    /// Whether combat damage has been dealt this combat; whether attackers were declared at all.
    pub attacked: bool,
    /// Combat damage steps done this combat: 1 after a first-strike step (CR 510.4), 2 after the regular one.
    pub damage_round: u8,
    /// The number of the turn "damage can't be prevented this turn" was made (CR 615.12), 0 for never.
    pub no_prevention_turn: u32,
    /// The number of the turn "creatures can't block this turn" was made (CR 509.1b, 611.2c), 0 for never: it reaches
    /// every creature, those that arrive later included.
    pub no_blocks_turn: u32,
    /// Day or night (CR 731.1): a fact about the game, kept here with the other facts about its turns.
    pub day_night: DayNight,
    /// The player who took the previous turn (CR 502.2, 731.2): whose spells the untap step counts.
    pub previous: PlayerId,
    /// Additional combat phases still to come this turn (CR 500.8): each follows the combat phase it was created in, or the
    /// precombat main phase's, as the end of combat step ends.
    pub extra_combats: u8,
    /// The player whose turn in the ordinary rotation was taken last, while extra turns are being taken (CR 500.7: an
    /// extra turn is taken between turns and does not move the rotation); none while no extra turn has been taken.
    pub rotation: Option<PlayerId>,
    /// An effect has ended the turn (CR 724.1d): the step under way ends with no player given priority again, and the
    /// game skips straight to the cleanup step.
    pub ending: bool,
    /// Players were given priority in this cleanup step (CR 514.3a), so another cleanup step follows it.
    pub cleanup_again: bool,
}

impl Turn {
    pub fn step(&self) -> Step {
        // Safety of the cast is the enum's own range: steps are only ever written through `set_step`.
        match self.step_raw {
            0 => Step::Untap,
            1 => Step::Upkeep,
            2 => Step::Draw,
            3 => Step::PrecombatMain,
            4 => Step::BeginCombat,
            5 => Step::DeclareAttackers,
            6 => Step::DeclareBlockers,
            7 => Step::CombatDamage,
            8 => Step::EndOfCombat,
            9 => Step::PostcombatMain,
            10 => Step::End,
            _ => Step::Cleanup,
        }
    }

    pub fn set_step(&mut self, step: Step) {
        self.step_raw = step as u8;
    }
}

/// An attacking creature and what it attacks (CR 506.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Attacker {
    pub id: ObjectId,
    pub target: Target,
    pub blocked: bool,
}

/// A blocking creature and the attacker it blocks (CR 509.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Block {
    pub blocker: ObjectId,
    pub attacker: ObjectId,
}

/// How the game ended, if it has (CR 104).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Outcome {
    #[default]
    InProgress,
    Won(PlayerId),
    Draw,
}

/// The shuffler and roller, in the state so that a game replays exactly (PCG32).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rng {
    pub state: u64,
    pub inc: u64,
    /// Die rolls a script has fixed in advance, five bits each (the face, 1 to 20), the next in the low bits: a scenario
    /// has to be able to say what a die shows. Used up before the generator is.
    pub forced: u64,
    pub forced_len: u8,
}

impl Rng {
    pub fn new(seed: u64) -> Rng {
        let mut rng = Rng { state: 0, inc: (seed << 1) | 1, forced: 0, forced_len: 0 };
        rng.next_u32();
        rng.state = rng.state.wrapping_add(seed ^ 0x853c_49e6_748f_ea9b);
        rng.next_u32();
        rng
    }

    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(self.inc);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    /// The next die roll, 1 to `sides` (CR 706.2): the next face fixed for the game, if any, else a random one.
    pub fn roll(&mut self, sides: u32) -> u32 {
        if self.forced_len > 0 {
            let face = (self.forced & 0b11111) as u32;
            self.forced >>= 5;
            self.forced_len -= 1;
            return face.clamp(1, sides.max(1));
        }
        self.below(sides) + 1
    }

    /// Uniform in `0..bound`, without modulo bias.
    pub fn below(&mut self, bound: u32) -> u32 {
        if bound <= 1 {
            return 0;
        }
        let threshold = bound.wrapping_neg() % bound;
        loop {
            let r = self.next_u32();
            if r >= threshold {
                return r % bound;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A subtype restriction's name reads back as the same restriction, whatever its subtypes hold: "or" inside a name
    /// (Orc, Warrior, Horror) and spaces (Time Lord) included.
    #[test]
    fn subtype_restriction_names_read_back() {
        let one = |t| ManaRestriction::Subtypes(SubtypeSpells::of(t));
        let mut all: Vec<ManaRestriction> = Subtype::ALL.into_iter().map(one).collect();
        all.push(ManaRestriction::Subtypes(SubtypeSpells::either(Subtype::Orc, Subtype::Warrior)));
        all.push(ManaRestriction::Subtypes(SubtypeSpells::either(Subtype::Horror, Subtype::Orc)));
        all.push(ManaRestriction::Subtypes(SubtypeSpells::either(Subtype::Dragon, Subtype::Omen)));
        all.push(ManaRestriction::Subtypes(SubtypeSpells { subtypes: [Some(Subtype::Elf), None], chosen: false, creature: true, uncounterable: true }));
        all.push(ManaRestriction::Subtypes(SubtypeSpells { subtypes: [None, None], chosen: false, creature: true, uncounterable: false }));
        all.push(ManaRestriction::mana_value_at_least(4));
        all.push(ManaRestriction::mana_value_at_least_or_x(5));
        all.push(ManaRestriction::mana_value_at_least(12));
        all.extend(ManaRestriction::NAMED.map(|(r, _)| r));
        for r in all {
            let name = r.name();
            assert_eq!(ManaRestriction::parse(name.split_whitespace()), Some(r), "{name}");
        }
    }
}
