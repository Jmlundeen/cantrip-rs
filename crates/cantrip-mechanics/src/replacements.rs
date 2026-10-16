//! Replacement effects (CR 614) many cards print.

use cantrip_api::*;

/// "If one or more +1/+1 counters would be put on a [permanent of these types] you control, that many plus one +1/+1
/// counters are put on it instead" (CR 122.6, 614.1a): whether it replaces this event. Keyed to the recipient, never to
/// who puts the counters; a permanent entering with counters is judged as it would exist on the battlefield
/// (CR 614.12). Michelangelo, Weirdness to 11 (creatures) and Ozolith, the Shattered Spire (artifacts and creatures)
/// print it.
pub fn plus_one_counters_applies(q: &dyn Query, asker: &Asker, proposal: &Proposal, types: &[CardType]) -> bool {
    let Proposal::Counters(c) = proposal else { return false };
    c.controller == asker.controller
        && c.counts[CounterKind::PlusOne as usize] > 0
        && (c.entering || q.is_on_battlefield(c.object))
        && types.iter().any(|&t| q.chars(c.object).is(t))
}

/// "If you would put one or more counters on a permanent or player, put twice that many of each of those kinds of
/// counters on that permanent or player instead" (CR 122.6, 614.1a): whether it replaces this event. Keyed to who puts
/// the counters, never to the recipient. Innkeeper's Talent (level 3) and Vorinclex, Monstrous Raider print it.
pub fn double_counters_applies(proposal: &Proposal, asker: &Asker) -> bool {
    match proposal {
        Proposal::Counters(c) => c.putter == Some(asker.controller) && c.counts.iter().any(|&n| n > 0),
        Proposal::PlayerCounters(c) => c.putter == Some(asker.controller) && (c.poison > 0 || c.energy > 0),
        _ => false,
    }
}

/// ... twice that many of each kind.
pub fn double_counters_apply(proposal: &mut Proposal) {
    match proposal {
        Proposal::Counters(c) => {
            for n in c.counts.iter_mut().filter(|n| **n > 0) {
                *n *= 2;
            }
        }
        Proposal::PlayerCounters(c) => {
            c.poison = c.poison.saturating_mul(2);
            c.energy = c.energy.saturating_mul(2);
        }
        _ => {}
    }
}

/// "... that many plus one": one more +1/+1 counter. Each such effect applies once, so a second adds another.
pub fn plus_one_counters_apply(proposal: &mut Proposal) {
    if let Proposal::Counters(c) = proposal {
        c.counts[CounterKind::PlusOne as usize] += 1;
    }
}

fn buyback_applies(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::ZoneChange(z)
        if z.object == asker.source
            && z.reason == ZoneReason::Resolve
            && z.to.is(ZoneKind::Graveyard)
            && q.obj(asker.source).kicked > 0)
}

fn buyback_apply(q: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        z.to = Zone::hand(q.obj(z.object).owner);
    }
}

/// Buyback's second half (CR 702.27a): "If the buyback cost was paid, put this spell into its owner's hand instead of
/// into that player's graveyard as it resolves." The cost is the card's `kicker`; only the resolution move is
/// replaced, so a countered spell goes to the graveyard. Corpse Dance, Capsize and Lava Burst print buyback.
pub const BUYBACK: ReplacementDef = replacement(ReplacementKind::ZoneChange, buyback_applies, buyback_apply, "returned to its owner's hand by buyback")
    .own_everywhere();

fn exile_instead_of_dying_applies(_: &dyn Query, _: &Asker, affected: ObjectId, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::ZoneChange(z)
        if z.object == affected && z.from.is(ZoneKind::Battlefield) && z.to.is(ZoneKind::Graveyard))
}

fn exile_instead_of_dying_apply(_: &dyn Query, _: &Asker, _: ObjectId, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        z.to = Zone::exile(z.to.player);
    }
}

/// "If that creature would die this turn, exile it instead" (CR 614.1a, 700.4, 611.2c): a replacement effect the spell or
/// ability makes about one object for the turn, not an ability the creature gains. Only a move from the battlefield to a
/// graveyard is replaced, so a creature returned to its hand goes there. List it among a card's `granted_replacements`
/// and make the effect with `exile_if_it_would_die` before anything can kill the creature. Suplex and Mawloc print it.
pub const EXILE_INSTEAD_OF_DYING: GrantedReplacementDef =
    granted_replacement(ReplacementKind::ZoneChange, exile_instead_of_dying_applies, exile_instead_of_dying_apply, "exiled instead of dying");

/// Makes `EXILE_INSTEAD_OF_DYING` (the card's replacement number `index`) about `creature` until end of turn. It names the
/// object, so it does not follow the creature if it leaves and returns as a new object (CR 400.7).
pub fn exile_if_it_would_die(g: &mut dyn Game, source: ObjectId, index: u8, creature: ObjectId) {
    g.add_effect(source, creature, EffectKind::Replacement(index), Duration::EndOfTurn);
}

/// Which permanents `opponents_enter_tapped` taps: a bit set of these.
pub mod enter_tapped {
    pub const CREATURES: u8 = 1;
    pub const ARTIFACTS: u8 = 2;
    pub const NONBASIC_LANDS: u8 = 4;
}

fn opponents_enter_tapped_applies<const WHICH: u8>(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    let Proposal::ZoneChange(z) = proposal else { return false };
    z.to.is(ZoneKind::Battlefield) && !z.tapped && z.controller != asker.controller && {
        let c = q.chars(z.object);
        (WHICH & enter_tapped::CREATURES != 0 && c.is(CardType::Creature))
            || (WHICH & enter_tapped::ARTIFACTS != 0 && c.is(CardType::Artifact))
            || (WHICH & enter_tapped::NONBASIC_LANDS != 0 && c.is(CardType::Land) && !c.supertypes.basic)
    }
}

fn opponents_enter_tapped_apply(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        z.tapped = true;
    }
}

/// "[Creatures / artifacts and creatures / nonbasic lands] your opponents control enter tapped" (CR 614.1c): the
/// object is judged as it would exist on the battlefield (CR 614.12), under the player it enters under (CR 110.2a).
/// `WHICH` is a set of `enter_tapped` bits. Authority of the Consuls (creatures), Spider-Woman, Stunning Savior
/// (artifacts and creatures) and Archon of Emeria (nonbasic lands) print it.
pub const fn opponents_enter_tapped<const WHICH: u8>(hint: &'static str) -> ReplacementDef {
    replacement(ReplacementKind::ZoneChange, opponents_enter_tapped_applies::<WHICH>, opponents_enter_tapped_apply, hint)
}
