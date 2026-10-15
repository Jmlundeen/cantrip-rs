use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;
use cantrip_mechanics::{damage_target, targets};

fn damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "{R}, Sacrifice Pyrite Spellbomb: It deals 2 damage to any target.",
            cost: Cost { mana: mana(0, 0, 0, 0, 1, 0), sacrifice_self: true, ..Cost::NONE },
            targets: &[targets::ANY],
            effect: damage,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{1}, Sacrifice Pyrite Spellbomb: Draw a card.",
            cost: Cost { mana: mana(1, 0, 0, 0, 0, 0), sacrifice_self: true, ..Cost::NONE },
            effect: effects::draw::<1>,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::PYRITE_SPELLBOMB
};
