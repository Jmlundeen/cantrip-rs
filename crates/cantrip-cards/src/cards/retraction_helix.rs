use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// The granted ability (the card's delayed ability 0): "{T}: Return target nonland permanent to its owner's hand."
fn return_to_hand(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        g.return_to_hand(&[permanent]);
    }
    Ok(())
}

/// Until end of turn, target creature gains the ability. The creature is the ability's source, so its {T} is subject
/// to summoning sickness (CR 302.6).
fn grant(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let this = g.obj(item.object).card;
    g.add_effect(item.object, creature, EffectKind::AddAbility(AbilityRef::delayed(this, 0)), Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], grant),
    delayed: &[ability(TAP, return_to_hand, "{T}: Return target nonland permanent to its owner's hand.")
        .targets(&[targets::NONLAND_PERMANENT])],
    ..oracle::RETRACTION_HELIX
};
