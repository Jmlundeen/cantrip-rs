//! Attachment (CR 301.5 Equipment, 303.4 Auras, 701.3): what a legal host is, shared by the effect that attaches
//! (CR 701.3b: an illegal attachment does not move), the state-based actions that shake an attachment off
//! (CR 704.5m-n) and an Aura choosing its host as it enters (CR 303.4f), so they can never disagree.

use cantrip_api::*;

use crate::layers::live;
use crate::{Cx, R, ops};

/// Whether `attachment` could be attached to `host` right now (CR 701.3a-b): the host is still the object it was,
/// is not the attachment itself and does not have protection from it (CR 702.16c-d), and is one the attachment's
/// enchant ability, or Equipment's creature on the battlefield (CR 301.5c), allows. Judged without targeting, so
/// hexproof does not matter.
pub fn host_allows(q: &dyn Query, attachment: ObjectId, host: ObjectId, attaching: bool) -> bool {
    if attachment == host || !live(q.state(), host) {
        return false;
    }
    let ac = q.chars(attachment);
    let hc = q.chars(host);
    if hc.protected_from_by(&ac, q.controller(attachment)) {
        return false;
    }
    if ac.has_subtype(Subtype::Equipment) {
        // An Equipment that's a creature can't equip unless it has reconfigure (CR 301.5c), and then only as it attaches.
        let creature_ok = attaching && q.card(ac.card).reconfigure;
        return (!ac.is(CardType::Creature) || creature_ok) && q.is_on_battlefield(host) && hc.is(CardType::Creature);
    }
    if ac.has_subtype(Subtype::Aura) {
        let asker = Asker { source: attachment, controller: q.controller(attachment) };
        return enchants_allow(q, attachment, ac.card, &asker, Target::Object(host));
    }
    false
}

/// Whether the Aura `aura` may enchant `what` by its enchant abilities (CR 303.4c, 702.5a, 704.5m): its card's printed one
/// (`card`, the card its copiable values are) and each one an effect gave it (`EffectKind::Enchant`: Necromancy,
/// `EntersAs::aura`). It has one, and each allows it, as each restricts what it can enchant.
fn enchants_allow(q: &dyn Query, aura: ObjectId, card: CardId, asker: &Asker, what: Target) -> bool {
    let mut any = false;
    if let Some(spec) = q.card(card).enchant {
        if !(spec.filter)(q, asker, what) {
            return false;
        }
        any = true;
    }
    for e in &q.state().effects {
        if e.affected == aura
            && let EffectKind::Enchant(by) = e.kind
        {
            match q.card(by).becomes_aura_enchant {
                Some(spec) if (spec.filter)(q, asker, what) => any = true,
                _ => return false,
            }
        }
    }
    any
}

/// Whether an Aura with these enchant abilities may enchant `what` (CR 303.4a, 303.4f, 704.5m): it has one, and each allows
/// it, as each restricts what it can enchant (CR 702.5a).
fn enchant_allows(specs: &[TargetSpec], q: &dyn Query, asker: &Asker, what: Target) -> bool {
    !specs.is_empty() && specs.iter().all(|spec| (spec.filter)(q, asker, what))
}

/// Whether an Aura with "enchant player" can enchant `player` right now (CR 303.4, 704.5m): a player still in the game,
/// without protection from it (CR 702.16c: from everything, CR 702.16j, or from the Aura's controller, 702.16k), whom its
/// enchant ability allows.
pub fn player_allows(cx: &Cx, aura: ObjectId, player: PlayerId) -> bool {
    let q: &dyn Query = cx;
    if q.state().player(player).lost
        || crate::layers::player_protection(cx.cards(), cx.s(), &cx.g.log, player, q.controller(aura)).is_some()
    {
        return false;
    }
    let ac = q.chars(aura);
    let asker = Asker { source: aura, controller: q.controller(aura) };
    enchants_allow(q, aura, ac.card, &asker, Target::Player(player))
}

/// What an Aura card about to enter would enchant (CR 303.4f): the battlefield, then the graveyards, as the C#
/// engine lists them, leaving out itself and everything entering with it. Each of its enchant abilities allows it
/// (`specs`), and it has no protection from the Aura as it would exist (`aura_chars`) under `controller`.
fn aura_hosts(cx: &Cx, aura: ObjectId, specs: &[TargetSpec], aura_chars: &Chars, controller: PlayerId, entering: &[ObjectId]) -> Vec<Target> {
    let asker = Asker { source: aura, controller };
    let state = cx.s();
    let mut zones = vec![Zone::BATTLEFIELD];
    zones.extend(state.seats().map(Zone::graveyard));
    let mut hosts = Vec::new();
    for zone in zones {
        for &id in state.zone(zone) {
            let target = Target::Object(id);
            if id == aura || entering.contains(&id) || !enchant_allows(specs, cx, &asker, target) {
                continue;
            }
            if cx.chars(id).protected_from_by(aura_chars, controller) {
                continue;
            }
            hosts.push(target);
        }
    }
    hosts
}

/// The enchant abilities a card about to enter has as an Aura (CR 303.4f), if it enters as one: its printed one, or the one
/// the effect putting it there gives it (`EntersAs::aura`, `CardDef::becomes_aura_enchant`). None if it enters as no Aura.
fn entering_enchants(cx: &Cx, m: &ops::Moving) -> Option<Vec<TargetSpec>> {
    let e = m.entering?;
    let card = e.copy.map(|c| c.card).unwrap_or(cx.s().obj(m.id).card);
    let printed = cx.card(card).enchant;
    let given = e.becomes.filter(|b| b.what.aura);
    if printed.is_none() && given.is_none() {
        return None;
    }
    let mut specs: Vec<TargetSpec> = printed.into_iter().collect();
    // An effect's Aura with no enchant ability (`becomes_aura_enchant` unset) could enchant nothing (CR 303.4g).
    specs.extend(given.and_then(|b| cx.card(b.card).becomes_aura_enchant));
    Some(specs)
}

/// Auras entering without being cast choose what they enchant, in turn order (CR 303.4f, 101.4); a choice made for
/// one is no target (hexproof does not matter), and an Aura with nothing to enchant does not enter (CR 303.4g). A card an
/// effect puts there as an Aura (`EntersAs::aura`) is one as it enters.
pub fn choose_aura_hosts(cx: &mut Cx, moves: &mut Vec<ops::Moving>) -> R<()> {
    let mut auras: Vec<(usize, Vec<TargetSpec>)> =
        (0..moves.len()).filter_map(|i| entering_enchants(cx, &moves[i]).map(|specs| (i, specs))).collect();
    if auras.is_empty() {
        return Ok(());
    }
    let entering: Vec<ObjectId> = moves.iter().map(|m| m.id).collect();
    let apnap: Vec<PlayerId> = cx.s().apnap().collect();
    auras.sort_by_key(|(i, _)| {
        let controller = moves[*i].entering.map(|e| e.controller);
        apnap.iter().position(|&p| Some(p) == controller)
    });
    let mut dropped = Vec::new();
    for (i, specs) in auras {
        let Some(mut e) = moves[i].entering else { continue };
        // Protection is judged against the Aura as it would be on the battlefield (CR 702.16c, 614.12): a creature card
        // an effect returns as an Aura enchantment is no creature then.
        let aura_chars = if e.becomes.is_some() { ops::as_it_would_exist(cx, &e).chars } else { cx.chars(moves[i].id) };
        let hosts = aura_hosts(cx, moves[i].id, &specs, &aura_chars, e.controller, &entering);
        let chosen = match hosts.len() {
            0 => {
                dropped.push(i);
                continue;
            }
            1 => hosts[0],
            _ => {
                let ask = RequestKind::ChooseTarget { label: "", slot_player: None, total: None,
                    source: moves[i].id,
                    purpose: Purpose::Attach,
                    candidates: hosts,
                    optional: false,
                    least: 1,
                    count: 1,
                    any_number: false,
                };
                match cx.ask_core(e.controller, ask, false)? {
                    Answer::Targets(t) if t.len() == 1 => t[0],
                    _ => continue,
                }
            }
        };
        e.attach = chosen.object();
        moves[i].entering = Some(e);
    }
    dropped.sort_unstable();
    for i in dropped.into_iter().rev() {
        moves.remove(i);
    }
    Ok(())
}
