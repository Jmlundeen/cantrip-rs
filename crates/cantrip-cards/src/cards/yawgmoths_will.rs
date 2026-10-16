use cantrip_api::*;
use cantrip_oracle as oracle;

/// "If a card would be put into your graveyard from anywhere this turn, exile that card instead" (CR 614.1a): a card, so
/// not a token, headed for the graveyard of the player who cast the spell.
fn exile_instead_applies(_: &dyn Query, asker: &Asker, _: ObjectId, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::ZoneChange(z) if z.is_card && z.to == Zone::graveyard(asker.controller))
}

fn exile_instead_apply(_: &dyn Query, _: &Asker, _: ObjectId, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        z.to = Zone::exile(z.to.player);
    }
}

/// Until end of turn, you may play lands and cast spells from your graveyard. If a card would be put into your graveyard from
/// anywhere this turn, exile that card instead. Both are effects of the spell lasting the turn (CR 611.2): about the player,
/// so they outlive the spell's own object. The spell is itself put into the graveyard once it is done, so it is exiled.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_effect(item.object, item.object, EffectKind::PlayFromGraveyard, Duration::EndOfTurn);
    g.add_effect(item.object, item.object, EffectKind::ReplacementRule(0), Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    granted_replacements: &[granted_replacement(ReplacementKind::ZoneChange, exile_instead_applies, exile_instead_apply, "exile that card instead of putting it into your graveyard")],
    ..oracle::YAWGMOTHS_WILL
};
