use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_oracle as oracle;

/// "If a card would be put into an opponent's graveyard from anywhere, instead exile it with a void counter on it."
/// (CR 614.1a.) It belongs to Dauthi Voidwalker, so it functions only while it is on the battlefield (CR 604.1). "An
/// opponent's graveyard" is read off the destination zone's owner rather than the moving card's, since a card someone
/// else owns can be discarded, milled or destroyed from anywhere; "a card" (CR 108.2) is not a token or a copy of a
/// spell. The void counter is given as the card arrives, because counters do not survive a zone change (CR 122.2).
fn applies(_: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::ZoneChange(z) if z.to.is(ZoneKind::Graveyard) && z.to.player != asker.controller && z.is_card)
}

fn apply(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        z.to = Zone::exile(z.to.player);
        z.counters[CounterKind::Void as usize] += 1;
    }
}

/// "{T}, Sacrifice this creature: Choose an exiled card an opponent owns with a void counter on it. You may play it this
/// turn without paying its mana cost." Every card in exile that carries a void counter (CR 122.1) and is owned by an
/// opponent of the ability's controller - not only those this creature exiled, since the counter belongs to the card.
/// One such card is no choice at all. The permission is for this turn (CR 118.9, 305.1).
fn play_exiled(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let mut eligible: Vec<ObjectId> = Vec::new();
    for owner in g.state().seats() {
        if owner == item.controller {
            continue;
        }
        for &card in g.state().zone(Zone::exile(owner)) {
            if g.obj(card).counters(CounterKind::Void) > 0 && g.obj(card).became.is_none() {
                eligible.push(card);
            }
        }
    }
    eligible.sort();
    let chosen = if eligible.len() == 1 { eligible.first().copied() } else { choose::card(g, item.controller, item.object, Purpose::Play, eligible)? };
    let Some(chosen) = chosen else { return Ok(()) };
    g.allow_play(chosen, item.controller, PlayGrant { instead: Some(ManaCost::ZERO), lands: true, expiry: Expiry::EndOfTurn, ..PlayGrant::CAST });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP.and(SACRIFICE_THIS), play_exiled, "{T}, Sacrifice Dauthi Voidwalker: Choose an exiled card an opponent owns with a void counter on it. You may play it this turn without paying its mana cost.")],
    replacements: &[ReplacementDef {
        hint: "exiles a card an opponent's graveyard would receive",
        kind: ReplacementKind::ZoneChange,
        applies,
        apply,
        ..ReplacementDef::BLANK
    }],
    ..oracle::DAUTHI_VOIDWALKER
};
