use cantrip_api::*;
use cantrip_oracle as oracle;

/// Creatures you control get +2/+0 until end of turn: the creatures there are as it resolves (CR 611.2c). If this
/// spell was kicked, whenever a creature you control dies this turn, draw a card.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::effects::pump_creatures_you_control(g, item, 2, 0);
    if g.obj(item.object).kicked > 0 {
        cantrip_mechanics::delayed::this_turn(g, item, item.object);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    kicker: kicker(mana("{B}")),
    delayed: &[triggered(cantrip_mechanics::triggers::CREATURE_YOU_CONTROL_DIES, |g, item| cantrip_mechanics::effects::draw(g, item, 1), "Whenever a creature you control dies this turn, draw a card.")],
    ..oracle::WARHOSTS_FRENZY
};
