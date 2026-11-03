use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "This creature deals 1 damage to you": dealt by the creature, as it last existed if it has left (CR 113.7a, 608.2h).
fn damage_you(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.damage(item.object, Target::Player(item.controller), 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::YOUR_UPKEEP,
            damage_you,
            "At the beginning of your upkeep, this creature deals 1 damage to you.",
        ),
    ],
    ..oracle::SERENDIB_EFREET
};
