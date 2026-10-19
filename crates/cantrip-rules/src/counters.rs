//! Counters (CR 122): the one path by which counters come to be on an object or a player.
//!
//! Every counter that is put on anything is put by a player (CR 122.6a, 608.2, 120.3b-d, 714.3c), and that player is
//! named here, never left out: "if you would put one or more counters" (Innkeeper's Talent, Vorinclex) asks who it is.
//! The counters are put through the replacement effects of counters being put (CR 614.1a) - on a permanent, a permanent
//! entering with them (CR 122.6, 614.1c) or a player - and the `CountersPut` event names the player too. The storage
//! (`Obj::counters`, `Player::poison` and the rest) is private to `cantrip-state`, and its one adder, `put_path_add`, is
//! called from this module alone (`only_the_put_path_puts_counters`). Taking counters off (`remove`) is not putting them.

use cantrip_api::*;
use cantrip_state::{Obj, PendingTrigger};

use crate::layers::{self, live};
use crate::{Cx, ops, replace};

/// How counters are put, besides by whom: by an effect (CR 609.1), or as a cost (CR 118.3) or a turn-based action
/// (CR 714.3c), which "if an effect would put one or more counters" (Doubling Season, `CounterPlacement::effect`) does
/// not see. Every other replacement of counters being put applies to all of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum How {
    Effect,
    Cost,
    TurnBased,
}

/// `amount` counters of one kind put on `id` by `by` (CR 122.6). Nothing for an amount of none or fewer.
pub fn put(cx: &mut Cx, id: ObjectId, kind: CounterKind, amount: i16, by: PlayerId, how: How) {
    if amount <= 0 {
        return;
    }
    let mut counts = [0; CounterKind::COUNT];
    counts[kind as usize] = amount;
    put_all(cx, id, counts, by, how);
}

/// Counters of several kinds put on `id` together by `by` (CR 122.6): on a permanent, as many as the replacement effects
/// of counters being put leave (CR 614.1a); on a card in another zone, as many as it can have (CR 101.2), which no
/// replacement of counters put on a permanent sees. Each kind put is announced, with `by`.
pub fn put_all(cx: &mut Cx, id: ObjectId, counts: [i16; CounterKind::COUNT], by: PlayerId, how: How) {
    if !live(cx.s(), id) || counts.iter().all(|&n| n <= 0) {
        return;
    }
    let on_battlefield = cx.s().obj(id).zone.is(ZoneKind::Battlefield);
    let counts = if on_battlefield {
        let controller = cx.s().obj(id).controller;
        replace::counters(cx, id, controller, by, false, how == How::Effect, counts)
    } else {
        counts
    };
    if counts.iter().all(|&n| n <= 0) {
        return;
    }
    cx.st().update_obj(id, |o| o.put_path_add(&counts));
    // The layers look for hone counters from now on (CR 122.1j).
    if counts[CounterKind::Hone as usize] > 0 {
        cx.st().see_statics();
    }
    // CR 613.7c: a counter gets a timestamp as it is put on, and every counter of its kind already there gets the same one;
    // the ability a keyword counter gives applies in layer 6 at it (CR 613.1f, 122.1b).
    if on_battlefield {
        for k in CounterKind::KEYWORDS {
            if counts[k as usize] > 0 {
                cx.st().stamp_counters(id, k);
            }
        }
    }
    // The counters that were put are announced (CR 121.1, 122.6a), for a Saga's chapters (CR 714.2b) and for "whenever
    // counters are put on this", as many as the replacements left, with the player who put them.
    for kind in CounterKind::ALL {
        let amount = counts[kind as usize];
        if amount > 0 {
            let total = cx.s().obj(id).counters(kind);
            cx.emit(Event::CountersPut { object: id, kind, amount, total, by });
        }
    }
}

/// Takes up to `n` counters of one kind off `id` (CR 122.1): what a cost removes (CR 118.3), what an effect removes, and
/// the state-based action of CR 704.5q. When the last defense counter comes off a Siege, its defeat ability triggers
/// (CR 310.12b).
pub fn remove(cx: &mut Cx, id: ObjectId, kind: CounterKind, n: i16) {
    if n <= 0 || !live(cx.s(), id) {
        return;
    }
    let before = cx.s().obj(id).counters(kind);
    if before <= 0 {
        return;
    }
    cx.st().update_obj(id, |o| o.remove_counters(kind, n));
    ops::end_lapsed_durations(cx, id);
    if kind == CounterKind::Defense && cx.s().obj(id).counters(CounterKind::Defense) == 0 {
        siege_defeat_triggers(cx, id);
    }
}

/// "Remove up to `maximum` counters from target creature" as an effect resolves (CR 608.2d, 122.1): `player` chooses
/// which of the counters on `id`, of any kinds and how many of each, from none to `maximum` in all, and they come off
/// together. Nothing is asked of an object that has changed zones (CR 400.7) or that has no counters. How many came off:
/// "for each counter removed this way". Heartless Act, Hex Parasite, Price of Betrayal (its permanents).
pub fn remove_up_to(cx: &mut Cx, player: PlayerId, source: ObjectId, id: ObjectId, maximum: u8) -> crate::R<u16> {
    if maximum == 0 || !live(cx.s(), id) {
        return Ok(0);
    }
    let obj = cx.s().obj(id);
    let pools: Vec<CounterPool> = CounterKind::ALL
        .into_iter()
        .filter_map(|kind| {
            let count = obj.counters(kind);
            (count > 0).then_some(CounterPool { object: id, kind, count: count as u16 })
        })
        .collect();
    if pools.is_empty() {
        return Ok(0);
    }
    let Answer::Counters(taken) = cx.ask_core(player, RequestKind::RemoveCounters { source, pools, total: maximum, up_to: true }, false)? else {
        return Ok(0);
    };
    let mut removed = 0u16;
    for (object, kind, n) in taken {
        let before = cx.s().obj(object).counters(kind);
        remove(cx, object, kind, n.min(i16::MAX as u16) as i16);
        removed += (before - cx.s().obj(object).counters(kind)) as u16;
    }
    Ok(removed)
}

/// A Siege that has lost its last defense counter triggers its defeat ability (CR 310.12b).
fn siege_defeat_triggers(cx: &mut Cx, id: ObjectId) {
    if !cx.s().obj(id).zone.is(ZoneKind::Battlefield) {
        return;
    }
    let chars = layers::chars(cx.cards(), cx.s(), &cx.g.log, id);
    if !chars.has_subtype(Subtype::Siege) {
        return;
    }
    let ability = AbilityRef { card: cx.cards().intrinsic, index: cantrip_api::intrinsic::SIEGE_DEFEAT };
    let controller = cx.s().obj(id).controller;
    cx.st().trigger_push(PendingTrigger {
        source: id,
        ability,
        controller,
        subject: Target::None,
        data: [0; 2],
        members: cantrip_state::Span::default(),
        count: None,
        since: None,
    });
}

/// What a permanent enters with because of what it is (CR 614.1c), counters it enters with like any other (CR 122.6),
/// put on by its controller (CR 122.6a): a planeswalker its printed loyalty (CR 306.5b), a battle its printed defense
/// (CR 310.4b), a Saga a lore counter (CR 714.3a). Added to the counters the replacements give it, before the replacements
/// of counters being put apply to them all (`entering`).
pub fn intrinsic(cards: &CardTable, obj: &Obj, counts: &mut [i16; CounterKind::COUNT]) {
    let card = obj.copy.map(|c| c.card).unwrap_or_else(|| cards.face(obj.card, obj.transformed));
    let def = cards.get(card);
    if let Some(loyalty) = def.loyalty {
        counts[CounterKind::Loyalty as usize] += loyalty;
    }
    if let Some(defense) = def.defense {
        counts[CounterKind::Defense as usize] += defense;
    }
    let saga = def.subtypes.has(Subtype::Saga) && def.abilities.iter().any(|a| matches!(a, AbilityDef::Triggered(t) if t.chapter > 0));
    if saga {
        counts[CounterKind::Lore as usize] += 1;
    }
}

/// The counters a permanent will enter with (CR 122.6, 614.1c), once the replacement effects of counters being put have
/// applied to them (CR 614.1a), the permanent judged as it would exist on the battlefield (the caller's `judged_entering`):
/// `by` puts them, its controller unless the effect giving them names another player (CR 122.6a). Placed by
/// `place_arrived` once it is there.
pub fn entering(cx: &mut Cx, id: ObjectId, controller: PlayerId, by: PlayerId, counts: [i16; CounterKind::COUNT]) -> [i16; CounterKind::COUNT] {
    replace::counters(cx, id, controller, by, true, true, counts)
}

/// Puts the counters an object arrives with on it as it arrives, before anything sees it (CR 614.1c: a permanent enters
/// with them), and announces them as counters `by` put on it (CR 122.6) into `events`, which the caller records after
/// the move's own event: a Saga's lore counter too, so its first chapter triggers (CR 714.3a, 714.2b), and all of them, so
/// "when the Nth ... counter is put" sees each it entered with. For a permanent, those `entering` worked out; for a card
/// arriving elsewhere, those a replacement effect gave it (Dauthi Voidwalker's void counter, CR 614.1a), which no
/// replacement of counters put on a permanent sees.
pub fn place_arrived(cx: &mut Cx, id: ObjectId, counts: [i16; CounterKind::COUNT], by: PlayerId, events: &mut Vec<Event>) {
    if counts.iter().all(|&n| n <= 0) {
        return;
    }
    cx.st().update_obj(id, |o| o.put_path_add(&counts));
    for kind in CounterKind::ALL {
        let amount = counts[kind as usize];
        if amount > 0 {
            let total = cx.s().obj(id).counters(kind);
            events.push(Event::CountersPut { object: id, kind, amount, total, by });
        }
    }
}

/// The permanent as it would exist on the battlefield with the counters it would enter with (CR 614.12): not counters
/// put, but a moment's hypothesis that `ops::as_it_would_exist` takes back (its journal mark) before anything happens.
pub(crate) fn hypothetical(o: &mut Obj, counts: &[i16; CounterKind::COUNT]) {
    o.clear_counters();
    o.put_path_add(counts);
}

/// Counters of a player's kinds (CR 122.1): poison, energy, experience.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlayerCounts {
    pub poison: u16,
    pub energy: u16,
    pub experience: u16,
}

/// Counters put on `player` by `by` (CR 122.6): an event replacement effects can change (CR 614.1a). A player who can't
/// get counters of a kind gets none (CR 101.2: Melira, Sylvok Outcast; Solemnity), and no replacement effect of the event
/// that doesn't happen applies.
pub fn put_on_player(cx: &mut Cx, player: PlayerId, counts: PlayerCounts, by: PlayerId) {
    let banned = |cx: &Cx, n: u16, kind| n > 0 && crate::static_rules::player_counters_banned(cx, cx.cards(), player, kind);
    let mut counts = counts;
    if banned(cx, counts.poison, PlayerCounter::Poison) {
        counts.poison = 0;
    }
    if banned(cx, counts.energy, PlayerCounter::Energy) {
        counts.energy = 0;
    }
    if banned(cx, counts.experience, PlayerCounter::Experience) {
        counts.experience = 0;
    }
    if counts == PlayerCounts::default() {
        return;
    }
    let counts = replace::player_counters(cx, player, by, counts);
    cx.st().update_player(player, |p| p.put_path_add(counts.poison, counts.energy, counts.experience));
}

#[cfg(test)]
mod tests {
    /// The counter storage's adder is called from the put path alone: any other caller would put counters with no
    /// player putting them and no replacement applied (CR 122.6a, 614.1a). The privacy of the storage stops every other
    /// write; this stops a second caller of the one adder, which the compiler cannot, as it is in another crate.
    #[test]
    fn only_the_put_path_puts_counters() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let mut offenders = Vec::new();
        let mut stack = vec![root.clone()];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    let text = std::fs::read_to_string(&path).unwrap();
                    let normal = path.to_string_lossy().replace('\\', "/");
                    let allowed = normal.ends_with("cantrip-rules/src/counters.rs") || normal.ends_with("cantrip-state/src/object.rs");
                    if !allowed && text.contains(concat!("put_path", "_add")) {
                        offenders.push(normal);
                    }
                }
            }
        }
        assert!(offenders.is_empty(), "counters put outside the put path (crates/cantrip-rules/src/counters.rs): {offenders:?}");
    }
}
