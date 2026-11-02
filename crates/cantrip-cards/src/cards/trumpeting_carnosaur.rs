use cantrip_api::*;
use cantrip_mechanics::{discover, effects, targets, triggers};
use cantrip_oracle as oracle;

/// "When this creature enters, discover 5." (CR 608.2).
fn discover_5(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    discover::discover(g, item.controller, 5)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            discover_5,
            "When this creature enters, discover 5.",
        ),
        ability(
                cost("{2}{R}").and(DISCARD_THIS),
                effects::damage_first_target::<3>,
                "{2}{R}, Discard this card: It deals 3 damage to target creature or planeswalker.",
            )
            .targets(&[targets::CREATURE_OR_PLANESWALKER])
            .from(ZoneKind::Hand),
    ],
    ..oracle::TRUMPETING_CARNOSAUR
};
