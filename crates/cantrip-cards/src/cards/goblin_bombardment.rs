use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

fn ping(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "Sacrifice a creature: Goblin Bombardment deals 1 damage to any target.",
        cost: Cost { permanents: Some(PermanentCost { count: 1, filter: targets::filters::creature, to_hand: false }), ..Cost::NONE },
        targets: &[targets::ANY],
        effect: ping,
        ..ActivatedDef::BLANK
    })],
    ..oracle::GOBLIN_BOMBARDMENT
};
