//! Phasing (CR 702.26): a permanent's phased-in or phased-out status (CR 110.5). A phased-out permanent is treated as though
//! it does not exist (CR 702.26b), so it is kept off the battlefield's list - which everything that looks at the battlefield
//! reads - in `Zone::PHASED_OUT`, as the same object it was (CR 702.26d): its counters, status, damage, attachments and
//! the effects that apply to it stay with it, a token goes on existing, and neither phasing out nor phasing in is a zone
//! change, so no "leaves" or "enters" ability sees it, and no "becomes attached" or "unattached" one (CR 702.26j).

use cantrip_api::*;
use cantrip_state::Phased;

use crate::Cx;
use crate::layers::live;

/// Whether `id` is still on the battlefield as the object it was, phased in or out: phasing out is no leaving
/// (CR 702.26d), so "until [it] leaves the battlefield" (CR 610.3, 610.4) goes on while it is phased out.
pub fn still_on_battlefield(s: &GameState, id: ObjectId) -> bool {
    live(s, id) && (s.obj(id).zone.is(ZoneKind::Battlefield) || s.obj(id).zone.is(ZoneKind::PhasedOut))
}

/// The permanents attached to any of `hosts`, and those attached to them, found in `list`.
fn attached_to_any(s: &GameState, list: &[ObjectId], hosts: &[ObjectId], only: impl Fn(&Obj) -> bool) -> Vec<ObjectId> {
    let mut found: Vec<ObjectId> = Vec::new();
    let mut frontier: Vec<ObjectId> = hosts.to_vec();
    while let Some(host) = frontier.pop() {
        for &id in list {
            let obj = s.obj(id);
            if obj.attached_to == Some(host) && only(obj) && !found.contains(&id) && !hosts.contains(&id) {
                found.push(id);
                frontier.push(id);
            }
        }
    }
    found
}

/// Phases `permanents` out, all at once (CR 702.26b): each phased-in permanent among them that is still on the battlefield,
/// and with it, indirectly, every Aura, Equipment and Fortification attached to it, and those attached to them (CR
/// 702.26g); one that would phase out both ways phases out only indirectly (CR 702.26h). Each is removed from combat (CR
/// 506.4). `until_leaves` is "phases out until [that permanent] leaves the battlefield" (CR 610.4): nothing phases out if
/// it has already left (CR 610.4b; Oubliette's ruling), and what does phases in only once it has (`phase_in_returning`),
/// tapped if `tapped_as_it_returns`.
pub fn phase_out(cx: &mut Cx, permanents: &[ObjectId], until_leaves: Option<ObjectId>, tapped_as_it_returns: bool) {
    if until_leaves.is_some_and(|by| !still_on_battlefield(cx.s(), by)) {
        return;
    }
    let mut direct: Vec<ObjectId> = Vec::new();
    for &id in permanents {
        if live(cx.s(), id) && cx.s().obj(id).zone.is(ZoneKind::Battlefield) && !direct.contains(&id) {
            direct.push(id);
        }
    }
    if direct.is_empty() {
        return;
    }
    let indirect = attached_to_any(cx.s(), cx.s().battlefield(), &direct, |_| true);
    direct.retain(|id| !indirect.contains(id));
    let all: Vec<(ObjectId, bool)> = direct.iter().map(|&id| (id, false)).chain(indirect.iter().map(|&id| (id, true))).collect();
    let leaving = |id: ObjectId| all.iter().any(|&(a, _)| a == id);
    if cx.s().attackers.iter().any(|a| leaving(a.id)) {
        let attackers = cx.s().attackers.iter().copied().filter(|a| !leaving(a.id)).collect();
        cx.st().set_attackers(attackers);
    }
    if cx.s().blocks.iter().any(|b| leaving(b.blocker) || leaving(b.attacker)) {
        let blocks = cx.s().blocks.iter().copied().filter(|b| !leaving(b.blocker) && !leaving(b.attacker)).collect();
        cx.st().set_blocks(blocks);
    }
    // CR 702.26f: a "for as long as" duration that tracks one of them ends, as it can no longer see it (Teferi's Protection's
    // ruling on Mathas, Fiend Seeker): one about its counters or its being tapped. (One about an effect's source on the
    // battlefield ends with the state-based check, `WhileSourceOnBattlefield`.)
    let tracks = |e: &ContinuousEffect| leaving(e.affected) && matches!(e.duration, Duration::WhileCounter(_) | Duration::WhileTapped);
    if cx.s().effects.iter().any(tracks) {
        cx.st().effects_retain(|e| !tracks(e));
    }
    for &(id, indirect) in &all {
        let controller = cx.s().obj(id).controller;
        let phased = if indirect {
            Phased { under: controller, indirect, until_leaves: None, tapped_as_it_returns: false }
        } else {
            Phased { under: controller, indirect, until_leaves, tapped_as_it_returns }
        };
        cx.st().zone_remove(Zone::BATTLEFIELD, id);
        cx.st().zone_push(Zone::PHASED_OUT, id);
        cx.st().update_obj(id, |o| {
            o.zone = Zone::PHASED_OUT;
            o.marks.phased = Some(phased);
        });
        cx.record(Event::PhasedOut { permanent: id, controller, indirect });
    }
    // CR 702.26f: so does a "for as long as you control" permission about one of them, for good though it phase in.
    if cx.s().permissions.iter().any(|p| p.grant.expiry.controlled().is_some_and(leaving)) {
        cx.st().end_lapsed_permissions();
    }
}

/// Phases in, all at once (CR 702.26c), each of `permanents` that phased out directly, and with each everything that phased
/// out indirectly with it (CR 702.26g), attached as it was. An Aura, Equipment or Fortification that phased out directly
/// phases in attached to what it was attached to only if that is still on the battlefield, or that player still in the
/// game; otherwise it phases in unattached, for the state-based actions to deal with (CR 702.26i, 704.5m, 704.5n). A
/// permanent goes back to whoever its control-changing effects now say controls it (CR 613.1b: the rulings on one that
/// expired while it was phased out).
fn phase_in(cx: &mut Cx, permanents: &[ObjectId]) {
    let direct: Vec<ObjectId> = permanents
        .iter()
        .copied()
        .filter(|&id| cx.s().obj(id).marks.phased.is_some_and(|p| !p.indirect) && cx.s().obj(id).zone.is(ZoneKind::PhasedOut))
        .collect();
    if direct.is_empty() {
        return;
    }
    let list = cx.s().zone(Zone::PHASED_OUT).to_vec();
    let indirect = attached_to_any(cx.s(), &list, &direct, |o| o.marks.phased.is_some_and(|p| p.indirect));
    let all: Vec<ObjectId> = direct.iter().chain(indirect.iter()).copied().collect();
    for &id in &all {
        let tapped = cx.s().obj(id).marks.phased.is_some_and(|p| p.tapped_as_it_returns);
        cx.st().zone_remove(Zone::PHASED_OUT, id);
        cx.st().zone_push(Zone::BATTLEFIELD, id);
        cx.st().update_obj(id, |o| {
            o.zone = Zone::BATTLEFIELD;
            o.marks.phased = None;
            // Oubliette's "tap that creature as it phases in this way": it is tapped as it phases in, which is no
            // becoming tapped that an ability sees (the ruling).
            o.tapped |= tapped;
        });
    }
    for &id in &direct {
        let obj = *cx.s().obj(id);
        if let Some(host) = obj.attached_to
            && !(live(cx.s(), host) && cx.s().obj(host).zone.is(ZoneKind::Battlefield))
        {
            cx.st().update_obj(id, |o| o.attached_to = None);
        }
        if let Some(p) = obj.attached_player
            && !cx.s().in_game(p)
        {
            cx.st().update_obj(id, |o| o.attached_player = None);
        }
    }
    for &id in &all {
        crate::ops::refresh_control(cx, id);
        cx.record(Event::PhasedIn { permanent: id });
    }
}

/// The untap step's first turn-based action (CR 502.1, 703.4a): every permanent that phased out directly under the active
/// player's control phases in, but not one phased out "until" an event (CR 610.4a). (Permanents with the phasing keyword
/// would phase out at the same time; no card here has it.)
pub fn untap_step(cx: &mut Cx, active: PlayerId) {
    let s = cx.s();
    if s.zone(Zone::PHASED_OUT).is_empty() {
        return;
    }
    let due: Vec<ObjectId> = s
        .zone(Zone::PHASED_OUT)
        .iter()
        .copied()
        .filter(|&id| s.obj(id).marks.phased.is_some_and(|p| !p.indirect && p.under == active && p.until_leaves.is_none()))
        .collect();
    phase_in(cx, &due);
}

/// CR 610.4: a permanent phased out "until [a permanent] leaves the battlefield" phases in, as a one-shot effect that
/// follows the event, once that permanent has left; Oubliette's ruling: "immediately after Oubliette leaves the
/// battlefield". Whether any did.
pub fn phase_in_returning(cx: &mut Cx) -> bool {
    let s = cx.s();
    if s.zone(Zone::PHASED_OUT).is_empty() {
        return false;
    }
    let due: Vec<ObjectId> = s
        .zone(Zone::PHASED_OUT)
        .iter()
        .copied()
        .filter(|&id| s.obj(id).marks.phased.and_then(|p| p.until_leaves).is_some_and(|by| !still_on_battlefield(s, by)))
        .collect();
    if due.is_empty() {
        return false;
    }
    phase_in(cx, &due);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;

    static BEAR: CardDef = CardDef {
        name: "Phasing Test Bear",
        types: TypeSet::of(&[CardType::Creature]),
        power: Some(2),
        toughness: Some(2),
        ..CardDef::BLANK
    };
    static AURA: CardDef = CardDef { name: "Phasing Test Aura", types: TypeSet::of(&[CardType::Enchantment]), ..CardDef::BLANK };
    static WARDEN: CardDef = CardDef { name: "Phasing Test Warden", types: TypeSet::of(&[CardType::Enchantment]), ..CardDef::BLANK };

    /// A game with player 0's Bear, an Aura of player 1's attached to it, and player 0's Warden, all on the battlefield.
    fn game() -> (Game, ObjectId, ObjectId, ObjectId) {
        let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&BEAR, &AURA, &WARDEN])));
        let mut game = Game::new(table, &crate::Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() });
        let put = |game: &mut Game, def: &'static CardDef, p: u8| {
            let id = game.state.push_obj(Obj::new(table.id_of(def), PlayerId(p), Zone::BATTLEFIELD));
            game.state.zone_push(Zone::BATTLEFIELD, id);
            id
        };
        let bear = put(&mut game, &BEAR, 0);
        let aura = put(&mut game, &AURA, 1);
        let warden = put(&mut game, &WARDEN, 0);
        game.state.update_obj(aura, |o| o.attached_to = Some(bear));
        (game, bear, aura, warden)
    }

    fn cx(game: &mut Game) -> Cx<'_> {
        Cx { g: game, frame: usize::MAX, cursor: 0, mana_source: None, entering: None, lineage: Vec::new(), asked_at: None }
    }

    #[test]
    fn a_permanent_phases_out_with_its_aura_and_both_phase_in_in_its_controllers_untap_step() {
        let (mut game, bear, aura, warden) = game();
        let mut cx = cx(&mut game);
        phase_out(&mut cx, &[bear], None, false);
        let s = cx.s();
        assert_eq!(s.battlefield(), &[warden]);
        assert_eq!(s.zone(Zone::PHASED_OUT), &[bear, aura]);
        // The same objects, the Aura still attached (CR 702.26d, 702.26g); it phased out indirectly, under its own controller.
        assert!(live(s, bear) && live(s, aura));
        assert_eq!(s.obj(aura).attached_to, Some(bear));
        assert_eq!(s.obj(aura).marks.phased.map(|p| (p.indirect, p.under)), Some((true, PlayerId(1))));
        assert_eq!(s.obj(bear).marks.phased.map(|p| (p.indirect, p.under)), Some((false, PlayerId(0))));
        // Not the Bear's controller's untap step: nothing phases in, the indirect Aura's controller's no more than another's.
        untap_step(&mut cx, PlayerId(1));
        assert_eq!(cx.s().zone(Zone::PHASED_OUT).len(), 2);
        untap_step(&mut cx, PlayerId(0));
        let s = cx.s();
        assert!(s.zone(Zone::PHASED_OUT).is_empty());
        assert_eq!(s.battlefield(), &[warden, bear, aura]);
        assert_eq!(s.obj(aura).attached_to, Some(bear));
        assert!(s.obj(bear).marks.phased.is_none() && s.obj(aura).marks.phased.is_none());
        assert!(s.obj(bear).zone.is(ZoneKind::Battlefield));
    }

    #[test]
    fn a_permanent_phased_out_until_another_leaves_phases_in_tapped_only_once_that_one_has_left() {
        let (mut game, bear, aura, warden) = game();
        let mut cx = cx(&mut game);
        phase_out(&mut cx, &[bear], Some(warden), true);
        // CR 610.4a: not in an untap step.
        untap_step(&mut cx, PlayerId(0));
        assert!(!phase_in_returning(&mut cx));
        assert_eq!(cx.s().zone(Zone::PHASED_OUT), &[bear, aura]);
        // The Warden phasing out is no leaving (CR 702.26d).
        phase_out(&mut cx, &[warden], None, false);
        assert!(!phase_in_returning(&mut cx));
        untap_step(&mut cx, PlayerId(0));
        crate::ops::move_to(&mut cx, warden, Zone::graveyard(PlayerId(0)));
        assert!(phase_in_returning(&mut cx));
        let s = cx.s();
        assert_eq!(s.battlefield(), &[bear, aura]);
        assert!(s.obj(bear).tapped && !s.obj(aura).tapped);
    }

    #[test]
    fn a_phased_out_permanent_is_moved_by_nothing() {
        let (mut game, bear, aura, _) = game();
        let mut cx = cx(&mut game);
        phase_out(&mut cx, &[bear], None, false);
        // CR 702.26b: "Return it to your hand" from an ability that remembered it does nothing.
        assert!(crate::ops::return_to_hand(&mut cx, &[bear]).is_empty());
        assert!(crate::ops::exile_all(&mut cx, &[aura]).is_empty());
        assert_eq!(cx.s().zone(Zone::PHASED_OUT), &[bear, aura]);
        assert!(live(cx.s(), bear) && live(cx.s(), aura));
    }

    #[test]
    fn nothing_phases_out_until_a_permanent_that_has_already_left() {
        let (mut game, bear, _, warden) = game();
        let mut cx = cx(&mut game);
        crate::ops::move_to(&mut cx, warden, Zone::graveyard(PlayerId(0)));
        phase_out(&mut cx, &[bear], Some(warden), true);
        assert!(cx.s().zone(Zone::PHASED_OUT).is_empty());
        assert!(!cx.s().obj(bear).tapped);
    }

    #[test]
    fn an_aura_that_phased_out_alone_phases_in_unattached_once_its_creature_is_gone() {
        let (mut game, bear, aura, _) = game();
        let mut cx = cx(&mut game);
        phase_out(&mut cx, &[aura], None, false);
        assert_eq!(cx.s().obj(aura).marks.phased.map(|p| p.indirect), Some(false));
        crate::ops::move_to(&mut cx, bear, Zone::graveyard(PlayerId(0)));
        untap_step(&mut cx, PlayerId(1));
        let s = cx.s();
        assert!(s.obj(aura).zone.is(ZoneKind::Battlefield));
        assert_eq!(s.obj(aura).attached_to, None);
    }
}
