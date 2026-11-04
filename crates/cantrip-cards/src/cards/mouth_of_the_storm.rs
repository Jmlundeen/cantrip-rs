use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Creatures your opponents control get -3/-0 until your next turn" (CR 611.2a, 611.2c): the creatures the opponents control as
/// the ability resolves, and none that enters or comes under their control later; it lasts through the rest of this turn and
/// the opponents' turns, ending as the ability's controller's next turn begins.
fn opponents_creatures_get_minus_three_zero(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<ObjectId> = g
        .state()
        .opponents(item.controller)
        .flat_map(|opponent| g.permanents_of(opponent))
        .filter(|&id| g.chars(id).is(CardType::Creature))
        .collect();
    instruction(g, |g| {
        for id in creatures {
            g.add_effect(item.object, id, EffectKind::Pump(-3, 0), Duration::UntilNextTurnOf(item.controller));
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggers::ward::<2>("Ward {2}"),
        triggered(
            triggers::THIS_ENTERS,
            opponents_creatures_get_minus_three_zero,
            "When this creature enters, creatures your opponents control get -3/-0 until your next turn.",
        ),
    ],
    ..oracle::MOUTH_OF_THE_STORM
};
