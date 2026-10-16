use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

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
    // The granted ability (the card's delayed ability 0).
    delayed: &[ability(TAP, cantrip_mechanics::effects::return_first_target_to_hand, "{T}: Return target nonland permanent to its owner's hand.")
        .targets(&[cantrip_mechanics::targets::NONLAND_PERMANENT])],
    ..oracle::BANISHING_KNACK
};
