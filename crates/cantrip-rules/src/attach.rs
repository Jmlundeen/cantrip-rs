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
        let def = q.card(ac.card);
        return def.enchant.or(def.becomes_aura_enchant).is_some_and(|spec| (spec.filter)(q, &asker, Target::Object(host)));
    }
    false
}

/// Whether an Aura with "enchant player" can enchant `player` right now (CR 303.4, 704.5m): a player still in the game
/// whom its enchant ability allows.
pub fn player_allows(q: &dyn Query, aura: ObjectId, player: PlayerId) -> bool {
    if q.state().player(player).lost {
        return false;
    }
    let ac = q.chars(aura);
    let asker = Asker { source: aura, controller: q.controller(aura) };
    q.card(ac.card).enchant.is_some_and(|spec| (spec.filter)(q, &asker, Target::Player(player)))
}

/// What an Aura card about to enter would enchant (CR 303.4f): the battlefield, then the graveyards, as the C#
/// engine lists them, leaving out itself and everything entering with it.
fn aura_hosts(cx: &Cx, aura: ObjectId, spec: &TargetSpec, controller: PlayerId, entering: &[ObjectId]) -> Vec<Target> {
    let asker = Asker { source: aura, controller };
    let aura_chars = cx.chars(aura);
    let state = cx.s();
    let mut zones = vec![Zone::BATTLEFIELD];
    zones.extend(state.seats().map(Zone::graveyard));
    let mut hosts = Vec::new();
    for zone in zones {
        for &id in state.zone(zone) {
            let target = Target::Object(id);
            if id == aura || entering.contains(&id) || !(spec.filter)(cx, &asker, target) {
                continue;
            }
            if cx.chars(id).protected_from_by(&aura_chars, cx.s().obj(aura).controller) {
                continue;
            }
            hosts.push(target);
        }
    }
    hosts
}

/// Auras entering without being cast choose what they enchant, in turn order (CR 303.4f, 101.4); a choice made for
/// one is no target (hexproof does not matter), and an Aura with nothing to enchant does not enter (CR 303.4g).
pub fn choose_aura_hosts(cx: &mut Cx, moves: &mut Vec<ops::Moving>) -> R<()> {
    let mut auras: Vec<usize> = (0..moves.len())
        .filter(|&i| {
            let card = moves[i].entering.and_then(|e| e.copy).map(|c| c.card).unwrap_or(cx.s().obj(moves[i].id).card);
            cx.card(card).enchant.is_some()
        })
        .collect();
    if auras.is_empty() {
        return Ok(());
    }
    let entering: Vec<ObjectId> = moves.iter().map(|m| m.id).collect();
    let apnap: Vec<PlayerId> = cx.s().apnap().collect();
    auras.sort_by_key(|&i| {
        let controller = moves[i].entering.map(|e| e.controller);
        apnap.iter().position(|&p| Some(p) == controller)
    });
    let mut dropped = Vec::new();
    for i in auras {
        let Some(mut e) = moves[i].entering else { continue };
        let card = e.copy.map(|c| c.card).unwrap_or(cx.s().obj(moves[i].id).card);
        let spec = cx.card(card).enchant.expect("an Aura");
        let hosts = aura_hosts(cx, moves[i].id, &spec, e.controller, &entering);
        let chosen = match hosts.len() {
            0 => {
                dropped.push(i);
                continue;
            }
            1 => hosts[0],
            _ => {
                let ask = RequestKind::ChooseTarget {
                    source: moves[i].id,
                    purpose: Purpose::Attach,
                    candidates: hosts,
                    optional: false,
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
