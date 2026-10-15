use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

fn creature_card(q: &dyn Query, _: &Asker, t: Target) -> bool {
    targets::card_in_graveyard(q, t, &[CardType::Creature])
}

/// Put target creature card from a graveyard onto the battlefield under your control; you lose life equal to its
/// mana value.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    let value = g.mana_value(card) as i32;
    g.put_onto_battlefield(card, item.controller, false)?;
    g.lose_life(item.controller, value);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[TargetSpec { min: 1, max: 1, filter: creature_card }], effect }),
    ..oracle::REANIMATE
};
