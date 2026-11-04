use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on up to one target creature. Draw a card." The ability may be activated with no target, and then it only
/// draws; with a target that is illegal as it resolves, the whole ability is removed from the stack (CR 608.2b) and nothing is
/// drawn, which the core does before this runs.
fn counter_then_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            cost("{W}, {T}").and(SACRIFICE_THIS),
            counter_then_draw,
            "Share \u{2014} {W}, {T}, Sacrifice this artifact: Put a +1/+1 counter on up to one target creature. Draw a card. Activate only as a sorcery.",
        )
        .sorcery_speed()
        .targets(&[targets::CREATURE.optional()]),
        ability(cost("{2}, {T}").and(SACRIFICE_THIS), effects::gain_life_then_draw::<3, 1>, "Nosh \u{2014} {2}, {T}, Sacrifice this artifact: You gain 3 life and draw a card."),
    ],
    ..oracle::BAGEL_AND_SCHMEAR
};
