//! What card code may do: read anything ([`Query`]), and change the game only through the operations of [`Game`],
//! each of which is a rules procedure the core owns (zone changes go through replacement, damage through
//! prevention, everything through the journal).

use cantrip_state::*;

use crate::card::{CounterTo, LibraryPlace};
use crate::card::{AbilityDef, ActivatedDef, Asker, CardDef};
use crate::chars::Chars;
use crate::history::{History, Record};

/// A die roll (CR 706.2): the face the die showed, and the result once the roll was modified (CR 706.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Roll {
    pub natural: u32,
    pub result: i32,
}

/// The effect has asked a question that has no answer yet. Returned with `?` all the way out of the card's code; the
/// engine records the question and runs the effect again once it is answered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Suspend;

/// Whether an object is still the object it was: it has not changed zones (CR 400.7) or ceased to exist.
pub fn is_live(q: &dyn Query, id: ObjectId) -> bool {
    let obj = q.obj(id);
    obj.became.is_none() && !obj.zone.is(ZoneKind::Gone)
}

/// The defending player of an attack at `attack` (CR 508.5, 310.9d): the player attacked, the controller of the
/// planeswalker attacked, or the protector of the battle attacked. None for anything else.
pub fn defending_player_of(state: &GameState, attack: Target) -> Option<PlayerId> {
    match attack {
        Target::Player(p) => Some(p),
        Target::Object(id) => {
            let obj = state.obj(id);
            obj.marks.protector.or(Some(obj.controller))
        }
        _ => None,
    }
}

/// What a cast made as an effect resolves (CR 608.2g) is cast for, beyond its being cast.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CastTerms {
    /// The cost paid instead of the mana cost (CR 118.9).
    pub instead: Option<ManaCost>,
    /// Mana of any type may pay for it (CR 609.4b, 118.14).
    pub any_type: bool,
    /// Generic mana added to its cost (CR 601.2f).
    pub more: u8,
    /// Life paid with its cost (CR 118.9, 119.4): Bolas's Citadel's mana value.
    pub life: u8,
    /// An instant or sorcery spell cast this way is exiled instead of put into a graveyard.
    pub exile_after: bool,
    /// Cast a copy of the card, which stays where it is (CR 707.12, 702.170a): the copy is a card_copy in the same zone, and
    /// ceases to exist when it leaves the stack or the cast is given up.
    pub copy: bool,
    /// The way it is cast (CR 709.3, 715.3, 712.8a): one of `CastWay::way` from `cast_ways_during_resolution`, for a card that
    /// can be cast as a half, its Adventure or its back face. Left out, a card with one way is cast that way and one with
    /// several asks its caster which, as they choose it as they cast it.
    pub way: Option<u8>,
}

/// One way a card could be cast as an effect resolves (CR 608.2g): the card as itself, one half of a split card (CR 709.3),
/// its Adventure (CR 715.3) or the back face of a modal double-faced card (CR 712.8a). Only that face is evaluated to see if
/// it can be cast (CR 709.3a, 715.3a, 712.8a) and only it is on the stack (CR 709.3b), so its mana value is the spell's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CastWay {
    /// What `CastTerms::way` names.
    pub way: u8,
    /// The face that would be cast.
    pub face: CardId,
    /// Its mana value as the spell on the stack: the printed cost's, an X in it being 0 (CR 107.3b).
    pub mana_value: u32,
}

/// Whether a spell or ability controlled by `causer` may cause `player` to discard (or, with `sacrifice`, to sacrifice)
/// (CR 101.2): nothing on the battlefield forbids it. Judged when the effect would ask or move, so an impossible discard
/// or sacrifice is not offered as a choice either.
pub fn may_cause_discard_or_sacrifice(q: &dyn Query, causer: PlayerId, player: PlayerId, sacrifice: bool) -> bool {
    for &id in q.state().battlefield() {
        let o = q.obj(id);
        let def = q.card(o.copy.map_or(o.card, |c| c.card));
        // A permanent that has lost all its abilities has no such rule (CR 613.1f).
        if def.cause_rules.is_empty() || q.state().lost_all_abilities(id) {
            continue;
        }
        let asker = Asker { source: id, controller: o.controller };
        if def.cause_rules.iter().any(|rule| !rule(q, &asker, causer, player, sacrifice)) {
            return false;
        }
    }
    true
}

/// Reading the game.
pub trait Query {
    fn state(&self) -> &GameState;

    /// "Defending player" of an attack at `attack` - an event's or an attacker's defender (CR 508.5, 310.9d).
    fn defending_player(&self, attack: Target) -> Option<PlayerId> {
        defending_player_of(self.state(), attack)
    }

    fn card(&self, id: CardId) -> &'static CardDef;

    /// The activated ability (CR 602) an ability reference names, if it names one: what an activation recorded in the
    /// history was, or one of an object's abilities (`chars(id).abilities()`) is - a printed one, or one an effect grants
    /// (`AbilityRef::delayed`, in the granting card's `delayed` list: Deconstruction Hammer's). The core's own abilities and
    /// the history's markers (madness, rebound, miracle, a granted way used, extra loyalty activations allowed) are none.
    fn activated_ability(&self, ability: AbilityRef) -> Option<&'static ActivatedDef> {
        let card = self.card(ability.card);
        let def = if ability.is_delayed() {
            card.delayed.get((ability.index - AbilityRef::DELAYED) as usize)
        } else {
            card.abilities.get(ability.index as usize)
        };
        match def {
            Some(AbilityDef::Activated(def)) => Some(def),
            _ => None,
        }
    }

    /// An object's characteristics after every continuous effect (CR 613).
    fn chars(&self, id: ObjectId) -> Chars;

    /// The words an object's text has now (CR 612): every color word, basic land type and creature type its rules text
    /// names is read through these, `words(source).subtype(Subtype::Goblin)` for "Goblin", so a text-changing effect
    /// changes what the card does.
    fn words(&self, id: ObjectId) -> Words {
        crate::words(self.state(), id)
    }

    /// The id of a definition in the card table (a token's, an exception's).
    fn card_id(&self, def: &'static CardDef) -> CardId;

    /// Every event since the game began, in order (`crate::history`).
    fn events(&self) -> &[Record];

    /// The game's event history, for any question about what has happened (`crate::history`).
    fn history(&self) -> History<'_> {
        History { records: self.events(), turn: self.state().turn.number }
    }

    /// The objects of the batch a "one or more" ability (`TriggeredDef::once_per_batch`) was triggered by
    /// (CR 603.2c, 608.2h).
    fn event_objects(&self, item: &StackItem) -> &[ObjectId] {
        self.state().members(item.members)
    }

    fn obj(&self, id: ObjectId) -> &Obj {
        self.state().obj(id)
    }

    fn is_on_battlefield(&self, id: ObjectId) -> bool {
        self.obj(id).zone.is(ZoneKind::Battlefield)
    }

    fn controller(&self, id: ObjectId) -> PlayerId {
        self.obj(id).controller
    }

    /// An object's mana cost (CR 202.1) as its characteristics give it: the card's or the one it copies, none for a copy
    /// that has none (CR 707.9b) or a face-down permanent (CR 708.2a), and a Room's unlocked doors on the battlefield
    /// (CR 709.5). Its {X} symbols are counted in `x`, not valued.
    fn mana_cost(&self, id: ObjectId) -> ManaCost {
        let c = self.chars(id);
        if c.no_cost {
            return ManaCost::ZERO;
        }
        let def = self.card(c.card);
        // A Room on the battlefield has the mana cost of its unlocked doors alone (CR 709.5); anywhere else the card's.
        if def.is_room() && self.obj(id).zone.is(ZoneKind::Battlefield) { def.room_cost(self.obj(id).marks.doors) } else { def.cost }
    }

    /// An object's mana value (CR 202.3): its mana cost's, or 0 for a copy that has none (CR 707.9b). Each {X} in it
    /// is the value chosen while it is a spell on the stack, as it last was there once it has left (CR 202.3e,
    /// 608.2h), and 0 anywhere else.
    fn mana_value(&self, id: ObjectId) -> u32 {
        let cost = self.mana_cost(id);
        let x = if self.obj(id).zone.is(ZoneKind::Stack) {
            self.obj(id).x
        } else {
            self.state().left_stack(id).map_or(0, |item| item.x)
        };
        cost.mana_value() + cost.x as u32 * x as u32
    }

    /// An object's copiable values (CR 707.2): what a copy of it copies, the exceptions of its own copy effect
    /// included, and the face that is up for a double-faced permanent (CR 707.8).
    fn copiable_values(&self, id: ObjectId) -> Copiable {
        let obj = self.obj(id);
        obj.copy.unwrap_or_else(|| Copiable::of(self.chars(id).card))
    }

    /// The permanents a player controls, in battlefield order.
    fn permanents_of(&self, player: PlayerId) -> Vec<ObjectId> {
        self.state().battlefield().iter().copied().filter(|&id| self.obj(id).controller == player).collect()
    }

    /// The targets already chosen for the earlier slots of the spell or ability whose target is being chosen or
    /// checked (CR 601.2c): "up to one target creature they control", where "they" is the player an earlier slot
    /// chose. Empty outside a target filter.
    fn chosen_targets(&self) -> &[Target] {
        &[]
    }

    /// The opponent the target slot being chosen or checked is for, when it is one of a `TargetSpec::EACH_OPPONENT`
    /// ("for each opponent, up to one target creature that player controls", CR 601.2c). `None` outside such a slot.
    /// Whether the targets are being judged as if the gift of the spell they are for were promised (CR 702.174m): asked of a
    /// gift spell with no target without it, to see whether the promise would give it one. `gift::promised` reads it.
    fn gift_assumed(&self) -> bool {
        false
    }

    fn slot_opponent(&self) -> Option<PlayerId> {
        None
    }

    /// The subject of the event a triggered ability triggered on (`Fired::subject`), for the filter of its targets: "target
    /// planeswalker that player controls", where the player is the one the damage was dealt to (CR 603.3d). `Target::None`
    /// outside a trigger's target filter.
    fn event_subject(&self) -> Target {
        Target::None
    }

    /// The cards in exile that were exiled with `by` (CR 607.2a): what its linked abilities refer to. A permanent that
    /// left and came back is a new object (CR 400.7) with none; in exile order.
    fn exiled_with(&self, by: ObjectId) -> Vec<ObjectId> {
        let s = self.state();
        s.seats().flat_map(|p| s.zone(Zone::exile(p)).iter().copied()).filter(|&id| self.obj(id).exiled.is_some_and(|l| l.by == by)).collect()
    }

    /// The object `left` became when it moved to a public zone, if that object is still where the move put it (CR 400.7e):
    /// "return it to the battlefield" from a "when this dies" ability finds the card in the graveyard it went to, and
    /// nothing once that card has moved again (a new object, CR 400.7). Persist, earthbend, Enduring Innocence and Enduring Curiosity return it.
    fn found_after_move(&self, left: ObjectId) -> Option<ObjectId> {
        self.obj(left).became.filter(|&new| self.obj(new).became.is_none() && !self.obj(new).zone.is(ZoneKind::Gone))
    }

    /// What an Aura or Equipment is attached to, if that is still the permanent it was and is on the battlefield.
    fn attached_to(&self, id: ObjectId) -> Option<ObjectId> {
        let host = self.obj(id).attached_to?;
        let h = self.obj(host);
        (h.became.is_none() && h.zone.is(ZoneKind::Battlefield)).then_some(host)
    }

    /// Whether a creature was declared as an attacker this turn (CR 508.1): still attacking or not, but still the
    /// object that attacked.
    fn attacked_this_turn(&self, id: ObjectId) -> bool {
        self.obj(id).became.is_none() && self.history().attacked_this_turn(id)
    }

    /// Whether an attacking creature is attacking (CR 506.3): declared as an attacker, or put onto the battlefield
    /// attacking (CR 508.4), and not removed from combat since.
    fn is_attacking(&self, id: ObjectId) -> bool {
        self.state().attackers.iter().any(|a| a.id == id)
    }

    /// Whether an attacking creature has been blocked (CR 509.1h): it stays blocked when its blockers leave.
    fn is_blocked(&self, id: ObjectId) -> bool {
        self.state().attackers.iter().any(|a| a.id == id && a.blocked)
    }

    /// Whether a permanent is saddled (CR 702.171b): it was this turn. One that has left the battlefield is as it last
    /// was there (CR 608.2h).
    fn is_saddled(&self, id: ObjectId) -> bool {
        let o = self.obj(id);
        let here = o.zone.is(ZoneKind::Battlefield) && o.became.is_none();
        o.marks.saddled_turn == self.state().turn.number && (here || self.state().last_known(id).is_some())
    }

    /// Whether `target` could be chosen by `asker` for a spell or ability: hexproof and protection (CR 702.11,
    /// 702.16b). The slot's own filter is the card's; this is the rules'.
    fn can_target(&self, asker: &Asker, target: Target) -> bool;
}

/// Changing the game.
pub trait Game: Query {
    /// Asks `player` a question. The first time, the effect stops here (`Err(Suspend)`); once answered, the effect
    /// runs again from its start and this returns the answer.
    fn ask(&mut self, player: PlayerId, kind: RequestKind) -> Result<Answer, Suspend>;

    /// For an effect written as explicit stages: the stage it is at, and the answer that brought it there.
    fn stage(&self) -> u16;
    fn staged_answer(&self) -> Option<Answer>;
    fn scratch(&self) -> [u32; 4];
    fn set_scratch(&mut self, scratch: [u32; 4]);
    /// Asks, and continues at `next` once answered, without running the stages before it again.
    fn ask_then(&mut self, next: u16, player: PlayerId, kind: RequestKind) -> Suspend;

    fn draw(&mut self, player: PlayerId, count: u32);
    fn discard(&mut self, card: ObjectId);
    /// Discards cards at once (CR 701.9): one event for what watches "one or more" discards.
    ///
    /// A card its owner can't be made to discard by this spell or ability stays (CR 101.2), and madness applies
    /// (CR 702.35a). The new objects, in order, of the cards that were discarded.
    fn discard_together(&mut self, cards: &[ObjectId]) -> Vec<ObjectId>;
    /// `player` discards their hand (CR 701.9): every card in it, at once, through `discard_together`. The new objects.
    fn discard_hand(&mut self, player: PlayerId) -> Vec<ObjectId> {
        let hand = self.state().zone(Zone::hand(player)).to_vec();
        self.discard_together(&hand)
    }
    /// Mills `count` cards (CR 701.17): the top cards of the library together. The new objects, in order, of the cards
    /// that were put into a graveyard or, by a replacement effect, into exile instead (CR 701.17c): the cards "from among
    /// them" an effect looks at.
    fn mill(&mut self, player: PlayerId, count: u32) -> Vec<ObjectId>;
    /// Deals damage (CR 120.3) from `source` to `target`: marked on a creature, removed as loyalty from a planeswalker,
    /// lost as life by a player, each after prevention and replacement. How much was dealt, which is none when the damage
    /// was prevented or the target is gone.
    fn damage(&mut self, source: ObjectId, target: Target, amount: u16) -> u16;
    /// Deals damage to several recipients at once (CR 120.2, 603.2c): "N damage to each creature and each opponent" is one
    /// event, so "whenever this deals damage" triggers once. Each entry is a source, a recipient and an amount; an effect
    /// that heals the damage already marked does not heal what the event itself dealt. How much each was dealt, in order.
    fn damage_together(&mut self, damage: &[(ObjectId, Target, u16)]) -> Vec<u16>;
    /// Destroys permanents at once (CR 701.8a, 101.4, 603.10a): a leaves-the-battlefield ability sees all of them leave. An
    /// indestructible one stays (CR 702.12b); a regeneration shield replaces its destruction (CR 701.19a); the rest go to
    /// their owners' graveyards. Something that is no longer the object it was is not destroyed (CR 400.7). The new objects,
    /// in order, of what was destroyed.
    fn destroy_all(&mut self, permanents: &[ObjectId]) -> Vec<ObjectId>;
    /// `destroy_all` for permanents that "can't be regenerated" (CR 701.19c): a regeneration shield does not save them.
    fn destroy_all_no_regen(&mut self, permanents: &[ObjectId]) -> Vec<ObjectId>;
    /// Destroys a permanent (CR 701.8a): the one-object case of `destroy_all`.
    fn destroy(&mut self, permanent: ObjectId) {
        self.destroy_all(&[permanent]);
    }
    /// Destroys a permanent that "can't be regenerated" (CR 701.19c): the one-object case of `destroy_all_no_regen`.
    fn destroy_no_regen(&mut self, permanent: ObjectId) {
        self.destroy_all_no_regen(&[permanent]);
    }
    /// Sacrifices permanents (CR 701.21a), together, in the order given. One an opponent's spell or ability can't make its
    /// controller sacrifice stays (CR 101.2). The new objects, in order, of what was sacrificed: "if you do" asks whether
    /// this is empty.
    fn sacrifice(&mut self, permanents: &[ObjectId]) -> Vec<ObjectId>;
    /// Returns objects to their owners' hands at once (CR 400.3, 101.4): from the battlefield, a graveyard, exile or
    /// anywhere else, whoever controls them. A token that leaves the battlefield ceases to exist (CR 111.7, 704.5d). Something
    /// that is no longer the object it was is not returned (CR 400.7). The new objects, in order.
    fn return_to_hand(&mut self, objects: &[ObjectId]) -> Vec<ObjectId>;
    /// Puts objects into their owners' libraries (CR 401.4, 400.3) at once, in `place`. The new objects, in order, of those
    /// that arrived in a library: a replacement may have sent one elsewhere, and it is not arranged.
    fn put_into_library(&mut self, objects: &[ObjectId], place: LibraryPlace) -> Vec<ObjectId>;
    /// Exiles objects at once (CR 406.2, 101.4), each to its owner's exile zone. Something that is no longer the object it
    /// was is not exiled (CR 400.7). The new objects, in order, of what was exiled.
    fn exile_all(&mut self, objects: &[ObjectId]) -> Vec<ObjectId>;
    /// Puts objects into their owners' graveyards at once (CR 400.3, 101.4), in the order given, from wherever they are but
    /// the battlefield: "put the rest into your graveyard" is no discard, no mill and no destruction. Something that is no
    /// longer the object it was is not moved (CR 400.7). The new objects, in order. Malevolent Rumble and Satyr Wayfinder
    /// put what they did not take there.
    fn put_into_graveyard(&mut self, objects: &[ObjectId]) -> Vec<ObjectId>;
    /// Moves an object to a zone (top of a library), returning the new object it becomes. Not for the battlefield:
    /// entering it is `put_onto_battlefield`, which applies entering replacements.
    fn move_to(&mut self, object: ObjectId, to: Zone) -> ObjectId;
    /// Moves several objects at once (CR 603.10a: a leaves-the-battlefield ability sees all of them leave), for a
    /// reason triggers can read. The new objects, in order.
    fn move_together(&mut self, moves: &[(ObjectId, Zone)], reason: ZoneReason) -> Vec<ObjectId>;
    /// Puts an object onto the battlefield under `controller` (CR 110.2a), its entering replacements applied.
    fn put_onto_battlefield(&mut self, object: ObjectId, controller: PlayerId, tapped: bool) -> Result<ObjectId, Suspend>;
    /// `put_onto_battlefield`, the object entering with these counters as well (CR 122.6, 614.1c): persist's and undying's
    /// "return it with a counter on it". They are part of how it enters, so a replacement of counters applies (CR 614.1).
    fn put_onto_battlefield_with_counters(&mut self, object: ObjectId, controller: PlayerId, tapped: bool, counters: &[(CounterKind, i16)]) -> Result<ObjectId, Suspend>;
    /// Puts an object onto the battlefield tapped and attacking `defender` (CR 508.4, 702.49a): attacking, but never
    /// declared as an attacker, so nothing that watches attackers being declared sees it. Outside combat it is just
    /// tapped. Returns the new object.
    fn put_onto_battlefield_attacking(&mut self, object: ObjectId, controller: PlayerId, defender: Target) -> Result<ObjectId, Suspend>;
    /// Puts an object onto the battlefield and attaches an Aura already on the battlefield to it, which now
    /// enchants "the creature put onto the battlefield with this Aura" (Animate Dead, Necromancy). Returns the new
    /// object; nothing is attached if the Aura has left or the object did not arrive.
    fn put_onto_battlefield_with_aura(&mut self, object: ObjectId, controller: PlayerId, aura: ObjectId) -> Result<ObjectId, Suspend>;
    /// Puts several objects onto the battlefield at once (CR 110.2a), each under the player given.
    fn put_all_onto_battlefield(&mut self, objects: &[(ObjectId, PlayerId)]) -> Result<Vec<ObjectId>, Suspend>;
    /// The same, each with whether it enters tapped: Winds of Abandon puts the basic lands it found onto the battlefield
    /// tapped, together.
    fn put_all_onto_battlefield_as(&mut self, objects: &[(ObjectId, PlayerId, bool)]) -> Result<Vec<ObjectId>, Suspend>;
    /// Sacrifices permanents and puts cards onto the battlefield at the same moment (CR 101.4): Goblin Welder.
    fn sacrifice_and_return(&mut self, sacrificed: &[ObjectId], returned: &[(ObjectId, PlayerId)]) -> Result<(), Suspend>;
    /// Puts an object onto the battlefield transformed, its back face up (CR 712.14a): "exile it, then return it to
    /// the battlefield transformed". A card with no back face stays where it is (CR 712.14a).
    fn put_onto_battlefield_transformed(&mut self, object: ObjectId, controller: PlayerId) -> Result<ObjectId, Suspend>;
    /// Transforms a permanent (CR 701.28): the other face is up. It stays the same object. Nothing if it is not a
    /// double-faced permanent on the battlefield.
    fn transform(&mut self, permanent: ObjectId);
    /// Creates `count` tokens that are copies of a permanent (CR 707.2, 111.1), all at once, under `controller`.
    fn create_token_copies(&mut self, of: ObjectId, controller: PlayerId, count: u32) -> Result<Vec<ObjectId>, Suspend>;
    /// Creates `count` tokens that are copies of these copiable values (CR 707.2, 111.1), the exceptions the effect
    /// states (CR 707.9b) among them: eternalize's "except it's a black Zombie 4/4 with no mana cost".
    fn create_token_copies_of(&mut self, values: Copiable, controller: PlayerId, count: u32) -> Result<Vec<ObjectId>, Suspend>;
    /// The player gets an emblem (CR 114.2): a new object in the command zone, owned by them, with the abilities
    /// of this definition. It never leaves.
    fn create_emblem(&mut self, def: CardId, player: PlayerId) -> ObjectId;
    /// Creates `count` tokens of a definition (CR 111.2), under `controller`.
    /// Creates tokens of several definitions as one instruction (CR 111.1, 101.4): they enter together, so a "one or
    /// more" ability sees one event. Each is a definition and who controls it.
    fn create_tokens_together(&mut self, tokens: &[(CardId, PlayerId)]) -> Result<Vec<ObjectId>, Suspend>;
    fn create_tokens(&mut self, def: CardId, controller: PlayerId, count: u32, tapped: bool) -> Result<Vec<ObjectId>, Suspend>;
    /// `create_tokens`, each token entering with these counters (CR 122.6, 111.1): "create a token with two +1/+1
    /// counters on it".
    fn create_tokens_with_counters(&mut self, def: CardId, controller: PlayerId, count: u32, tapped: bool, counters: &[(CounterKind, i16)]) -> Result<Vec<ObjectId>, Suspend>;
    /// Creates a delayed triggered ability (CR 603.7), controlled by `controller`, about `subject`.
    fn delayed_trigger(&mut self, when: DelayedWhen, source: ObjectId, controller: PlayerId, ability: AbilityRef, subject: Target);
    /// `delayed_trigger`, carrying the objects the ability is about to the moment it triggers (`event_objects` of its
    /// stack item): every Warrior a mobilize ability created, which it sacrifices together (CR 603.7c).
    fn delayed_trigger_about(&mut self, when: DelayedWhen, source: ObjectId, controller: PlayerId, ability: AbilityRef, objects: &[ObjectId]);
    /// Copies a spell on the stack (CR 707.10) for `copier`, not cast; with `new_targets` the copier may choose new
    /// targets (CR 707.10c). The copy, or nothing if the spell is not on the stack.
    fn copy_spell(&mut self, spell: ObjectId, copier: PlayerId, new_targets: bool) -> Result<Option<ObjectId>, Suspend>;
    /// A reflexive triggered ability (CR 603.12): "when you do, ...", triggered as the effect that made it resolves and
    /// put on the stack, targets chosen, the next time a player would receive priority. `ability` is one of the
    /// source card's `delayed` abilities.
    fn reflexive_trigger(&mut self, source: ObjectId, controller: PlayerId, ability: AbilityRef, subject: Target);
    /// `reflexive_trigger` for an ability that counts what the first part did (CR 603.12): "When you discard one or
    /// more nonland cards this way, tap up to that many target creatures". `count` is that many: a
    /// `TargetSpec::COUNTED` bound of the ability's target slots reads it as the ability is put on the stack, and its
    /// stack item's `x` is it, for an effect that does "that many" of something (and for a copy of the ability).
    fn reflexive_trigger_counted(&mut self, source: ObjectId, controller: PlayerId, ability: AbilityRef, subject: Target, count: u16);
    /// It becomes day (CR 731.1): every daybound or nightbound permanent whose face disagrees is turned over. Nothing if
    /// it already is day; a game that has had a designation always has one.
    fn become_day(&mut self);
    fn become_night(&mut self);
    /// Records that the controller did what the ability lets them do only once each turn (CR 603.2h), for
    /// `History::done`.
    fn note_done(&mut self, source: ObjectId, ability: AbilityRef);
    /// Marks a permanent to be exiled instead if it would leave the battlefield (CR 702.84a).
    fn exile_on_leave(&mut self, permanent: ObjectId);
    fn shuffle(&mut self, player: PlayerId);
    /// A number below `bound`, from the game's own random source (CR 701.9c "at random"): journalled, so the same
    /// game and answers give the same result.
    fn random(&mut self, bound: u32) -> u32;
    /// `player` rolls a die with this many sides (CR 706.2) and the roll is recorded as `Event::DieRolled`, which
    /// triggers "whenever you roll a die". The natural result is the face, 1 to `sides`; it is the result too.
    fn roll_die(&mut self, player: PlayerId, sides: u32) -> Roll {
        self.roll_die_modified(player, sides, 0)
    }
    /// A roll whose result is modified (CR 706.4): `modifier` added to the natural result.
    fn roll_die_modified(&mut self, player: PlayerId, sides: u32, modifier: i32) -> Roll;
    /// One of `count` things is chosen at random (CR 701.9c): its index, below `count`; recorded as
    /// `Event::ChosenAtRandom`. Nothing is drawn when there is no choice (`count` of 0 or 1 gives 0).
    fn choose_at_random(&mut self, count: usize) -> usize;
    /// `player` flips a coin (CR 705.1): true if they win the flip (CR 705.2). A flip is a two-sided die whose first face
    /// wins; it is recorded as `Event::CoinFlipped` and is not a die roll.
    fn flip_coin(&mut self, player: PlayerId) -> bool;
    /// Moves a card already in a library to `from_top` cards from its top (0 is the top): "second from the top".
    fn place_in_library(&mut self, card: ObjectId, from_top: usize);
    /// "After this phase, there is an additional combat phase" (CR 500.8), from the precombat main phase or a combat phase: a
    /// whole combat phase, from its beginning, follows the combat phase that is under way (or the one that is next).
    fn add_combat_phase(&mut self);
    /// "End the turn" (CR 724.1), from a resolving spell or ability: triggered abilities waiting to be put on the stack
    /// cease to exist (724.1a); every object on the stack is exiled, the resolving one included (724.1b), whose
    /// remaining instructions still happen after this; state-based actions are checked with no player given priority
    /// and no triggered ability put on the stack (724.1c); and once the resolution is over the step under way ends, every
    /// creature and planeswalker is removed from combat, and the game skips straight to the cleanup step (724.1d) - a new
    /// cleanup step when it is ended during one. Abilities that trigger from here on go on the stack in that cleanup step,
    /// and players then get priority, with another cleanup step after it (724.1f, 514.3a).
    fn end_the_turn(&mut self) -> Result<(), Suspend>;

    /// `player` wins the game (CR 104.2b): their opponents lose and the game ends at once. Nothing if they have lost, or
    /// are losing to a state-based action (CR 104.3f).
    /// `permanent` becomes a copy of these copiable values until end of turn (CR 707.2, 611.2b): the same object, as another card.
    /// `source` is the spell or ability doing it.
    fn become_copy(&mut self, source: ObjectId, permanent: ObjectId, values: Copiable);
    fn win_game(&mut self, player: PlayerId);
    /// An effect states that `player` loses the game (CR 104.3e): they are out at once, and the last one left has won.
    fn lose_game(&mut self, player: PlayerId);
    /// `player` gets poison counters (CR 122.1f); ten lose the game (CR 704.5c).
    fn give_poison(&mut self, player: PlayerId, amount: u32);
    /// `player` gets experience counters (CR 122.1).
    fn give_experience(&mut self, player: PlayerId, amount: u32);
    /// `count` extra turns for `player` after the current one (CR 500.7); the most recently created is taken first.
    fn add_extra_turns(&mut self, player: PlayerId, count: u32);
    /// A permanent with a prepare spell becomes prepared (CR 722.2): a copy of the spell is put in exile, which its
    /// controller may cast while it stays prepared. Nothing if it already is, or is not on the battlefield.
    fn become_prepared(&mut self, permanent: ObjectId);
    /// A Class becomes a level (CR 716.2a), announced to what watches it. Nothing if it is not on the battlefield.
    fn set_class_level(&mut self, permanent: ObjectId, level: u8);
    /// Puts cards of a player's library on its top in this order, the first on top (CR 401.4), or on its bottom,
    /// the first lowest. The cards are already in the library; nothing changes zones.
    fn arrange_library(&mut self, player: PlayerId, top: &[ObjectId], bottom: &[ObjectId]);
    /// Puts cards already in a player's library on its bottom in a random order (CR 401.4), drawn from the game's own
    /// random source: "put the rest on the bottom of your library in a random order". Nothing changes zones.
    fn bottom_in_random_order(&mut self, player: PlayerId, cards: &[ObjectId]);
    /// Records that `player` searched the library of `library` (CR 701.23, 701.23f: a player may search another
    /// player's library, and "searches their library" is only the case where the two are the same), for whatever
    /// watches searches.
    fn searched(&mut self, player: PlayerId, library: PlayerId);
    /// `player` gave a gift to `to` (CR 702.174c): the gift triggered ability of a permanent they control resolved. (A
    /// spell's gift is announced by the core as the spell resolves.)
    fn gave_gift(&mut self, player: PlayerId, to: PlayerId, gift: Gift);
    /// Attaches an Aura or Equipment to a permanent (CR 701.3a): nothing happens if it could not be attached there
    /// (CR 701.3b) or already is. It becomes a new object with respect to what it is attached to (CR 701.3c, 613.7e).
    fn attach(&mut self, attachment: ObjectId, host: ObjectId);
    /// Unattaches an Aura or Equipment (CR 701.3d); it stays on the battlefield.
    fn unattach(&mut self, attachment: ObjectId);
    fn tap(&mut self, permanent: ObjectId);
    /// Untaps a permanent (CR 701.26a). One with a stun counter on it has one removed instead (CR 122.1d).
    fn untap(&mut self, permanent: ObjectId);
    /// Exerts a permanent (CR 701.43a): it won't untap during its controller's next untap step.
    fn exert(&mut self, permanent: ObjectId);
    /// Monstrosity N (CR 701.37a): if the permanent is not monstrous, it gets N +1/+1 counters and becomes monstrous,
    /// which abilities that watch for it see.
    fn monstrosity(&mut self, permanent: ObjectId, counters: i16);
    /// A creature becomes saddled until end of turn (CR 702.171b).
    fn become_saddled(&mut self, permanent: ObjectId);
    /// Two creatures fight (CR 701.12a): each deals damage equal to its power to the other. Nothing happens unless both
    /// are creatures on the battlefield (CR 701.12b).
    fn fight(&mut self, first: ObjectId, second: ObjectId);
    /// Creates tokens that enter tapped and attacking `defender` (CR 508.4): attacking, but never declared as
    /// attackers, so nothing that watches attackers being declared sees them. They are put onto the battlefield
    /// together, as one instruction. Outside combat they are just tapped tokens.
    fn create_tokens_attacking(&mut self, tokens: &[(CardId, PlayerId)], defender: Target) -> Result<Vec<ObjectId>, Suspend>;
    /// Creates tokens together whose text has `words` (CR 612.4), tapped, or tapped and attacking `attacking`. Every other
    /// way of creating tokens gives them the words of the text making them, so a text change reaches the colors and
    /// creature types it names (CR 612.2a); a keyword's own token is the rules' (its reminder text is no rules text),
    /// so For Mirrodin!'s Rebel is made with `Words::NONE`.
    fn create_tokens_worded(&mut self, tokens: &[(CardId, PlayerId)], tapped: bool, attacking: Option<Target>, words: Words) -> Result<Vec<ObjectId>, Suspend>;
    /// Exiles cards together and remembers them as exiled with `by` (CR 607.2a, 610.3). With `until_leaves` they return
    /// to where they were exiled from the moment `by` is no longer on the battlefield (CR 610.3), no stack and nothing
    /// to respond to; and if `by` has already left, nothing is exiled at all (CR 610.3b). The new objects.
    fn exile_with(&mut self, cards: &[ObjectId], by: ObjectId, until_leaves: bool) -> Vec<ObjectId>;
    /// Exiles a spell card encoded on a creature (CR 702.99a): linked to it as `exile_with` links, and marked encoded
    /// (`ExileLink::encoded`), which only cipher's encoding is. The card in exile, if it got there.
    fn encode(&mut self, card: ObjectId, creature: ObjectId) -> Option<ObjectId>;
    /// Exiles cards together face down (CR 406.3), linked to `by` if an ability exiled them (CR 607.2a). Each is hidden
    /// from every player who could not see where it came from; `look_at` shows it to one more. The new objects.
    fn exile_face_down(&mut self, cards: &[ObjectId], by: Option<ObjectId>) -> Vec<ObjectId>;
    /// `player` looks at face-down cards in exile (CR 406.3), and may from now on.
    fn look_at(&mut self, player: PlayerId, cards: &[ObjectId]);
    /// Puts an object onto the battlefield face down (CR 708.3): a 2/2 creature with no name, types or abilities
    /// (CR 708.2a), its controller able to turn it face up for its mana cost if it is a creature card (CR 701.40b).
    fn put_onto_battlefield_face_down(&mut self, object: ObjectId, controller: PlayerId) -> Result<ObjectId, Suspend>;
    /// Turns a face-down permanent face up (CR 701.40b, 708.2). Nothing if it is not one.
    fn turn_face_up(&mut self, permanent: ObjectId);
    /// `player` unlocks a door of a Room (CR 709.5f): the permanent is given that half's unlocked designation, which
    /// triggers "when you unlock this door" (CR 709.5h). `door` is `DOOR_LEFT` or `DOOR_RIGHT`. Nothing if the permanent is
    /// not a Room on the battlefield or the door is already unlocked.
    fn unlock_door(&mut self, permanent: ObjectId, door: u8, player: PlayerId);
    /// A card name chosen for a permanent as an ability of it resolves, remembered by that permanent (CR 607.2d): "When this
    /// land enters, choose a land card name" (Petrified Hamlet), "II — Choose a card name" (Medomai's Prophecy). Its
    /// abilities that refer to "the chosen name" read it from `Marks::chosen_card`, as they read a name chosen as it entered
    /// (`Entering::chosen_card`, CR 614.12a). It is the object's alone: a new object the permanent becomes has no chosen
    /// name (CR 400.7), and a later choice for the same object replaces the earlier one. Nothing if the permanent is no
    /// longer on the battlefield.
    fn remember_chosen_name(&mut self, permanent: ObjectId, name: CardId);
    fn exile(&mut self, object: ObjectId) -> ObjectId {
        let owner = self.obj(object).owner;
        self.move_to(object, Zone::exile(owner))
    }
    /// `player` gains life (CR 119.3): none, if an effect or a static ability says they can't (CR 119.7,
    /// `EffectKind::CantGainLife`, `CardDef::cant_gain_life`).
    fn gain_life(&mut self, player: PlayerId, amount: i32);
    fn lose_life(&mut self, player: PlayerId, amount: i32);
    /// "`player`'s life total becomes `total`" (CR 119.5): they gain or lose the difference, so a player who can't gain life
    /// keeps a total lower than `total` (CR 119.7).
    fn set_life_total(&mut self, player: PlayerId, total: i32);
    /// Each opponent of `player` who is still in the game loses `amount` life (CR 119.3), in turn order. The total they lost.
    fn each_opponent_loses_life(&mut self, player: PlayerId, amount: i32) -> i32;
    fn add_mana(&mut self, player: PlayerId, mana: Mana, amount: u8);
    /// Adds mana that lasts until the end of combat (CR 106.4, 500.5a): not lost as the steps before it end.
    fn add_mana_until_end_of_combat(&mut self, player: PlayerId, mana: Mana, amount: u8);
    /// Adds mana that can only be spent as `restriction` allows (CR 106.6).
    fn add_restricted_mana(&mut self, player: PlayerId, mana: Mana, amount: u8, restriction: ManaRestriction);
    fn add_counters(&mut self, object: ObjectId, kind: CounterKind, amount: i16);
    /// Removes up to `amount` counters of one kind from an object (CR 122.1) and says how many came off: none from an
    /// object that has changed zones (CR 400.7). It answers "if you do" and "for each counter removed this way", as Dawn of a
    /// New Age and Coalition Relic ask.
    fn remove_counters(&mut self, object: ObjectId, kind: CounterKind, amount: i16) -> i16 {
        let before = self.obj(object).counters(kind);
        self.add_counters(object, kind, -amount);
        before - self.obj(object).counters(kind)
    }
    fn add_effect(&mut self, source: ObjectId, affected: ObjectId, kind: EffectKind, duration: Duration);
    /// An effect about `controller` and not an object (`EffectKind::outlives_affected`), for the player who controls the
    /// resolving ability or spell (CR 109.5: "you") even if its source has since changed hands or left.
    fn add_player_effect(&mut self, controller: PlayerId, source: ObjectId, kind: EffectKind, duration: Duration);
    /// `player` gains control of a permanent (CR 108.4, 613.1b) for `duration`, by `source`'s spell or ability.
    fn gain_control(&mut self, source: ObjectId, permanent: ObjectId, player: PlayerId, duration: Duration) {
        self.add_effect(source, permanent, EffectKind::Control(player), duration);
    }
    /// The controllers of two permanents exchange control of them (CR 701.12a), with no end. The exchange is all or nothing:
    /// if either is not on the battlefield, or both are controlled by one player (CR 701.12b), nothing changes hands. Whether
    /// it happened.
    fn exchange_control(&mut self, source: ObjectId, first: ObjectId, second: ObjectId) -> bool {
        let on_battlefield = |g: &Self, id: ObjectId| g.obj(id).became.is_none() && g.is_on_battlefield(id);
        if !on_battlefield(self, first) || !on_battlefield(self, second) {
            return false;
        }
        let (a, b) = (self.controller(first), self.controller(second));
        if a == b {
            return false;
        }
        self.gain_control(source, first, b, Duration::Indefinite);
        self.gain_control(source, second, a, Duration::Indefinite);
        true
    }
    /// Counters a spell (CR 701.5): it leaves the stack for its owner's graveyard, or exile if it was flashed back.
    fn counter(&mut self, spell: ObjectId);
    /// Counters a spell and puts it in its owner's hand, on top or bottom of their library, or in exile (CR 701.6a).
    fn counter_to(&mut self, spell: ObjectId, to: CounterTo);
    /// A spell on the stack changes controllers (CR 112.2): it resolves for `player`, and "you" in it means them. Its
    /// targets, modes and X stay as they were chosen.
    fn change_controller(&mut self, spell: ObjectId, player: PlayerId);
    /// `chooser` may choose new targets for a spell on the stack (CR 115.7d): each slot is offered every legal target it
    /// has now, its current one among them; a slot with no other choice is not asked.
    fn choose_new_targets(&mut self, spell: ObjectId, chooser: PlayerId) -> Result<(), Suspend>;
    /// "Change the target of target spell or ability" (CR 115.7a): the spell or ability (a `Target::Object` spell or a
    /// `Target::Ability`) on the stack has each of its targets changed to another legal one, `chooser` choosing; if any has no
    /// other legal target, none changes. A target with one other choice is not asked.
    fn change_target(&mut self, of: Target, chooser: PlayerId) -> Result<(), Suspend>;
    /// A spell leaves the stack without being countered (CR 400.3) for its owner's hand, library or exile: Sink into
    /// Stupor, Subtlety. A spell that can't be countered goes too.
    fn remove_spell(&mut self, spell: ObjectId, to: CounterTo);
    /// Counters whatever spell or ability of `object` is on the stack (CR 701.6), at `hint` if it is still there.
    fn counter_item(&mut self, object: ObjectId, hint: usize);
    fn reveal(&mut self, player: PlayerId, cards: &[ObjectId]);
    /// Whether target `slot` of the resolving item is still legal (CR 608.2b).
    fn target_legal(&self, item: &StackItem, slot: usize) -> bool;
    /// "Unless [payer] pays [cost]" (CR 118.12): asks, and pays if they choose to and can. Whether it was paid.
    fn unless_pays(&mut self, payer: PlayerId, source: ObjectId, subject: ObjectId, cost: ManaCost) -> Result<bool, Suspend>;
    /// Whether `card` could be cast now by the ability's controller as an effect resolves (CR 608.2g), from where it
    /// is: it is not a land and has a legal target for every target it needs. The mana is not asked: a player who
    /// finds they cannot pay gives the cast up (CR 733.1).
    fn could_cast_during_resolution(&self, card: ObjectId) -> bool;
    /// Whether the resolving effect's controller could play `card` as a land now (CR 305.1, 608.2g): it is their own turn
    /// (CR 305.3) and they have a land play left (CR 305.2); the phase and the stack do not matter.
    fn could_play_land_during_resolution(&self, card: ObjectId) -> bool;
    /// The resolving effect's controller plays `card` as a land from where it is (CR 305.1, 608.2g), counted as one of the
    /// turn's land plays (CR 305.2a). The effect has already asked whether to ("you may play"). The new object, or none if
    /// it could not be played.
    fn play_land_during_resolution(&mut self, card: ObjectId) -> Result<Option<ObjectId>, Suspend>;
    /// The resolving effect's controller casts `card` from where it is (CR 608.2g, 601.2): its targets are chosen and
    /// its costs paid as for any cast, for `instead` rather than its mana cost if given (CR 118.9: madness). Whether
    /// it was cast; a cast the player gives up (CR 733.1) leaves the card where it was.
    fn cast_during_resolution(&mut self, card: ObjectId, instead: Option<ManaCost>) -> Result<bool, Suspend>;
    /// The same, with terms (CR 608.2g): mana of any type may pay for it, an instant or sorcery is exiled instead of
    /// going to a graveyard.
    fn cast_during_resolution_as(&mut self, card: ObjectId, terms: CastTerms) -> Result<bool, Suspend>;
    /// Creates a copy of an object in exile, owned and controlled by `player` (CR 707.12, 702.192a: "create a copy of this
    /// object in exile"), and returns it. `of` may have left the zone it was in, a spell that has resolved: the copy is of it
    /// as it last existed (CR 608.2h). The copy is a copy of a card, which ceases to exist as a state-based action unless it
    /// is cast (CR 707.10a, 704.5e).
    fn create_copy_in_exile(&mut self, of: ObjectId, player: PlayerId) -> ObjectId;
    /// Each way the resolving effect's controller could cast `card` now (CR 608.2g), the faces it offers judged one at a time
    /// (CR 709.3a, 715.3a): a split card's halves, an aftermath half only from a graveyard (CR 702.127a); an Adventurer as itself
    /// and as its Adventure; a modal double-faced card's two faces. Fuse is no way here: it applies in a hand only
    /// (CR 702.102a). Empty when `could_cast_during_resolution` is false.
    fn cast_ways_during_resolution(&self, card: ObjectId) -> Vec<CastWay>;
    /// `player` may play or cast `card` from where it is (CR 601.3, 305.1) under `grant`: a card in exile or a
    /// graveyard, a new object after a move (CR 400.7), so the permission is for this object only.
    fn allow_play(&mut self, card: ObjectId, player: PlayerId, grant: PlayGrant);
    /// "Creatures can't block this turn" (CR 509.1b): a rule of the game until the turn ends, reaching creatures that
    /// arrive later too (CR 611.2c).
    fn creatures_cant_block(&mut self);
    /// `player` becomes the monarch (CR 725.1) and whoever was stops being it (CR 725.3).
    fn become_monarch(&mut self, player: PlayerId);
    /// `player` takes the initiative (CR 726.1): whoever had it stops, and the taker ventures into Undercity
    /// (CR 726.2), even one who already had it (CR 726.5).
    fn take_initiative(&mut self, player: PlayerId);
    /// `player` ventures into Undercity (CR 701.49): enters it, or moves on to the next room, their choice when
    /// there are two. The room's ability triggers (CR 309.4c). The one who ventures out of the bottommost room
    /// completes the dungeon (CR 701.49c, 309.7) and enters it again.
    fn venture(&mut self, player: PlayerId) -> Result<(), Suspend>;
    /// Whether the resolving effect's controller could cast `card` transformed now, from where it is (CR 608.2g,
    /// 712.11a): a double-faced card that is not modal, judged as its back face alone.
    fn could_cast_transformed_during_resolution(&self, card: ObjectId) -> bool;
    /// The resolving effect's controller casts `card` transformed, for nothing (CR 712.11a, 118.9): put on the stack
    /// with its back face up, it resolves into a permanent back face up (CR 712.13). Whether it was cast.
    fn cast_transformed_during_resolution(&mut self, card: ObjectId) -> Result<bool, Suspend>;
    /// "Damage can't be prevented this turn" (CR 615.12): a rule of the game until the turn ends.
    fn damage_cant_be_prevented(&mut self);
    /// Makes the object about to enter a copy (CR 707.2).
    fn set_copy(&mut self, object: ObjectId, copy: Copiable);
    /// `player` gets energy counters (CR 107.14), or with a negative `delta` pays them (CR 122.1): never below none.
    fn change_energy(&mut self, player: PlayerId, delta: i32);
    /// "You may activate this permanent's loyalty abilities `times` more times this turn" (CR 606.3): recorded in the
    /// history, which is what the one-a-turn limit is judged against.
    fn allow_extra_loyalty_activations(&mut self, permanent: ObjectId, times: u8);
    /// `player` proliferates (CR 701.34a): chooses any number of permanents and players that have a counter, each of
    /// which gets another counter of each kind it already has. One question, asked of `player` about `source`'s effect;
    /// nothing is asked when no permanent or player has a counter.
    fn proliferate(&mut self, player: PlayerId, source: ObjectId) -> Result<(), Suspend>;
}
