use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Creatures you control have base power and toughness X/X until end of turn" (CR 613.4b, 611.2c): the creatures
/// that are there as it resolves, and no others.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = item.x as i16;
    for id in g.permanents_of(item.controller) {
        if g.chars(id).is(CardType::Creature) {
            g.add_effect(item.object, id, EffectKind::SetPt(x, x), Duration::EndOfTurn);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    ..oracle::BIOMASS_MUTATION
};
