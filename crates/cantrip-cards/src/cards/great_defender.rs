use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// The target's mana value is fixed as the spell resolves (CR 608.2h, 611.2a).
fn great_defender(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    // Toughness modifiers use i32; values beyond that limit are capped (RX-5).
    let value = g.mana_value(creature) as i32;
    g.add_effect(item.object, creature, EffectKind::Pump(0, value), Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], great_defender),
    ..oracle::GREAT_DEFENDER
};
