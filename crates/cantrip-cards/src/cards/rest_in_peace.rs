use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "When this enchantment enters, exile all graveyards." Every card in every graveyard is exiled at once (CR 406.2, 101.4),
/// each to its owner's exile zone.
fn exile_graveyards(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let mut cards: Vec<ObjectId> = Vec::new();
    for owner in g.state().seats() {
        cards.extend(g.state().zone(Zone::graveyard(owner)).iter().copied());
    }
    g.exile_all(&cards);
    Ok(())
}

/// "If a card or token would be put into a graveyard from anywhere, exile it instead" (CR 614.1a). A copy of a spell is
/// neither a card nor a token (CR 108.2, 111.1), so it is not replaced.
fn applies(q: &dyn Query, _: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::ZoneChange(z) if z.to.is(ZoneKind::Graveyard) && (z.is_card || q.obj(z.object).token))
}

fn apply(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        z.to = Zone::exile(z.to.player);
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, exile_graveyards, "When this enchantment enters, exile all graveyards.")],
    replacements: &[replacement(ReplacementKind::ZoneChange, applies, apply, "exiles a card or token a graveyard would receive")],
    ..oracle::REST_IN_PEACE
};
