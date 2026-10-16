use cantrip_api::*;
use cantrip_mechanics::pump;
use cantrip_oracle as oracle;

/// All creatures get -X/-X until end of turn, for the X paid in life as an additional cost.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = item.x as i16;
    let creatures: Vec<ObjectId> =
        g.state().battlefield().iter().copied().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    for creature in creatures {
        pump(g, item.object, creature, -x, -x);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    // As an additional cost to cast this spell, pay X life.
    additional_cost: Some(Cost { pay_x_life: true, ..Cost::NONE }),
    ..oracle::TOXIC_DELUGE
};
