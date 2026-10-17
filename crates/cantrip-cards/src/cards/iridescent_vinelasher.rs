use cantrip_api::*;
use cantrip_mechanics::triggers::LANDFALL;
use cantrip_mechanics::{damage_target, offspring, targets};
use cantrip_oracle as oracle;

/// "Landfall -- Whenever a land you control enters, this creature deals 1 damage to target opponent" (CR 207.2c, 603.2): the
/// target is chosen as the ability goes on the stack, an opponent being the only legal kind; with it gone as the ability
/// resolves nothing is dealt (CR 608.2b).
fn ping(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    kicker: Some(offspring::kicker(mana("{2}"))),
    abilities: &[
        offspring::trigger("Offspring {2} (You may pay an additional {2} as you cast this spell. If you do, when this creature enters, create a 1/1 token copy of it.)"),
        triggered(LANDFALL, ping, "Landfall \u{2014} Whenever a land you control enters, this creature deals 1 damage to target opponent.")
            .targets(&[targets::OPPONENT]),
    ],
    ..oracle::IRIDESCENT_VINELASHER
};
