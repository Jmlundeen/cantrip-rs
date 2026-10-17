//! The questions about the past that several printed cards ask in the same words, written once over the event
//! history (`cantrip_api::history`). A trigger matching an event and a replacement deciding about one both see the
//! history as it was just before it, so each of these reads the same from either.

use cantrip_api::*;

/// Whether a draw `player` is about to make (or is making, from a trigger on it) is the first card they draw in their
/// own draw step this turn: the draw that "except the first one they draw in each of their draw steps" exempts. Orcish
/// Bowmasters (a trigger) and Hullbreacher (a replacement) print it.
pub fn first_draw_in_own_draw_step(q: &dyn Query, player: PlayerId) -> bool {
    let turn = q.state().turn;
    turn.active == player
        && turn.step() == Step::Draw
        && !q.history().any(EventQuery::kind(EventMask::DREW).player(player).this_turn().step(Step::Draw))
}

/// Cards `player` has drawn this turn, before the draw being asked about (Faerie Mastermind's "second card", Leovold's
/// and Narset's "more than one card each turn").
pub fn drawn_this_turn(q: &dyn Query, player: PlayerId) -> usize {
    q.history().drawn(player)
}

/// Spells cast this turn by every player (storm, CR 702.40a, counts those cast before the storm spell: subtract it).
pub fn spells_cast_this_turn(q: &dyn Query) -> usize {
    q.history().casts().count()
}

/// Revolt (CR 207.2c): whether a permanent `player` controlled left the battlefield this turn. The leaving object's
/// row is as it was when it left, so its controller then is read from it.
pub fn permanent_left_battlefield_this_turn(q: &dyn Query, player: PlayerId) -> bool {
    q.history()
        .select(EventQuery::kind(EventMask::ZONE_CHANGE).from(ZoneKind::Battlefield).this_turn())
        .any(|r| matches!(r.event, Event::ZoneChange { old, .. } if q.obj(old).controller == player))
}

/// The void condition: "a nonland permanent left the battlefield this turn or a spell was warped this turn". Void is an
/// ability word (CR 207.2c) with no rules of its own; only its second clause has a rule, CR 702.185c: a spell was warped when
/// it was cast for its warp cost. Any player's permanent counts, a token too, judged as it was as it left the battlefield (CR 608.2h). Elegy Acolyte, Plasma Bolt and
/// every other card with a void ability print it.
pub fn void(q: &dyn Query) -> bool {
    q.history()
        .select(EventQuery::kind(EventMask::ZONE_CHANGE).from(ZoneKind::Battlefield).this_turn())
        .any(|r| matches!(r.event, Event::ZoneChange { old, .. } if q.state().last_known(old).is_some_and(|l| !l.types.has(CardType::Land))))
        || crate::warp::spell_was_warped_this_turn(q)
}

/// Cards (not tokens) that left `player`'s graveyard this turn (Gau, Feral Youth).
pub fn cards_left_graveyard_this_turn(q: &dyn Query, player: PlayerId) -> usize {
    q.history()
        .select(EventQuery::kind(EventMask::ZONE_CHANGE).from(ZoneKind::Graveyard).this_turn())
        .filter(|r| matches!(r.event, Event::ZoneChange { old, from, .. } if from == Zone::graveyard(player) && !q.obj(old).token))
        .count()
}

/// The cards put into `owner`'s graveyard from the battlefield this turn that are still there (Faith's Reward, Second
/// Sunrise, No Rest for the Wicked): the graveyard objects those moves made, in the order they arrived.
pub fn put_into_graveyard_from_battlefield_this_turn(q: &dyn Query, owner: PlayerId) -> Vec<ObjectId> {
    q.history()
        .select(EventQuery::kind(EventMask::ZONE_CHANGE).from(ZoneKind::Battlefield).to(ZoneKind::Graveyard).this_turn())
        .filter_map(|r| match r.event {
            // A token that left the battlefield is not a card in the graveyard (CR 111.7, 704.5d).
            Event::ZoneChange { old, new, to, .. }
                if to == Zone::graveyard(owner) && q.obj(new).became.is_none() && !q.obj(old).token =>
            {
                Some(new)
            }
            _ => None,
        })
        .collect()
}

/// Whether `permanent` entered the battlefield this turn (CR 400.7: it is the object that move made). Asked of a history
/// so that a static ability's condition reads it too (`LayerQuery::history`): Shardmage's Rescue's "as long as this Aura
/// entered this turn", The Wandering Emperor's, power-up's cost.
pub fn entered_this_turn(history: History<'_>, permanent: ObjectId) -> bool {
    history
        .select(EventQuery::kind(EventMask::ZONE_CHANGE).to(ZoneKind::Battlefield).this_turn())
        .any(|r| matches!(r.event, Event::ZoneChange { new, .. } if new == permanent))
}

/// How many of `player`'s opponents have lost life this turn (spectacle's "an opponent lost life this turn", Drill
/// Bit; Kaito, Bane of Nightmares counts them).
pub fn opponents_who_lost_life_this_turn(q: &dyn Query, player: PlayerId) -> usize {
    let h = q.history();
    q.state().opponents(player).filter(|&p| h.life_lost(p) > 0).count()
}

/// Whether an opponent of `player` has lost life this turn (CR 119.3: "lost life", not a lower life total - one who gained 2
/// and lost 1 still did). Spectacle, Fireglass Mentor and Flamecache Gecko ask it.
pub fn an_opponent_lost_life_this_turn(q: &dyn Query, player: PlayerId) -> bool {
    opponents_who_lost_life_this_turn(q, player) > 0
}

/// `an_opponent_lost_life_this_turn` as a condition of the ability's controller: a triggered ability's intervening "if an
/// opponent lost life this turn" (CR 603.4) - Fireglass Mentor, Flamecache Gecko - and spectacle's.
pub fn an_opponent_lost_life(q: &dyn Query, asker: &Asker) -> bool {
    an_opponent_lost_life_this_turn(q, asker.controller)
}

/// Whether `object` arrived where it is this turn (CR 400.7: it is the object a move this turn made), in whatever zone:
/// Night Nurse's "put there from anywhere this turn", and, on the battlefield, `entered_this_turn`.
pub fn arrived_this_turn(q: &dyn Query, object: ObjectId) -> bool {
    q.history()
        .select(EventQuery::kind(EventMask::ZONE_CHANGE).this_turn())
        .any(|r| matches!(r.event, Event::ZoneChange { new, .. } if new == object))
}

/// The creatures that died this turn (CR 700.4): every move from the battlefield to a graveyard this turn of an object
/// that was a creature as it left, by last known information, so a token that has since ceased to exist still counts.
/// With `under`, only those that player controlled as they died (Callous Sell-Sword's "under your control"; the leaving
/// object's row is as it was when it left). Spymaster's Vault counts them all.
pub fn creatures_died_this_turn(q: &dyn Query, under: Option<PlayerId>) -> usize {
    q.history()
        .select(EventQuery::kind(EventMask::ZONE_CHANGE).from(ZoneKind::Battlefield).to(ZoneKind::Graveyard).this_turn())
        .filter(|r| {
            matches!(r.event, Event::ZoneChange { old, .. }
                if q.state().last_known(old).is_some_and(|l| l.types.has(CardType::Creature))
                    && under.is_none_or(|p| q.obj(old).controller == p))
        })
        .count()
}

/// "If you gained life this turn" (CR 603.4, 119.3): whether `player` has gained any life this turn. Ocelot Pride's and
/// Enlightened Confidant's intervening ifs print it.
pub fn gained_life_this_turn(q: &dyn Query, player: PlayerId) -> bool {
    q.history().life_gained(player) > 0
}

/// `gained_life_this_turn` as a condition of the ability's controller: a triggered ability's intervening "if you gained life this
/// turn" (CR 603.4). Lunar Convocation and Moseo, Vein's New Dean print it.
pub fn you_gained_life_this_turn(q: &dyn Query, asker: &Asker) -> bool {
    gained_life_this_turn(q, asker.controller)
}

/// "If you lost life this turn" (CR 603.4, 119.3): whether `player` has lost any life this turn, however much they also gained
/// (damage and paying life count). Lunar Convocation and Thought-Stalker Warlock print it.
pub fn lost_life_this_turn(q: &dyn Query, player: PlayerId) -> bool {
    q.history().life_lost(player) > 0
}

/// Whether `player` has activated an equip ability this turn (CR 702.6a, 602.2): what "the first equip ability you activate
/// [during each of your turns / each turn]" asks of an equip ability being announced, which is not yet in the history.
/// Forge Anew and Kíli the Resourceful print it.
pub fn activated_an_equip_ability_this_turn(q: &dyn Query, player: PlayerId) -> bool {
    q.history().this_turn().iter().any(|r| {
        matches!(r.event, Event::AbilityActivated { ability, player: p, .. }
            if p == player && q.activated_ability(ability).is_some_and(|d| d.equip))
    })
}

/// "Choose one that hasn't been chosen" (CR 700.2): whether mode `mode` of a modal ability has not been chosen before by the
/// object that has it (`asker.source`), as its ability went on the stack. A mode chosen for an ability that was then countered
/// or fizzled stays chosen, whoever controlled the object when it was chosen, and an object that left and came back is a new one
/// (CR 400.7) with every mode to choose again. Gollum, Riddle Master and Demonic Pact print it (`ModalDef::allowed`).
pub fn mode_not_chosen_before(q: &dyn Query, asker: &Asker, mode: u8) -> bool {
    !q.history().all().iter().any(|r| {
        matches!(r.event, Event::ModesChosen { source, modes, .. } if source == asker.source && modes & (1 << mode) != 0)
    })
}
