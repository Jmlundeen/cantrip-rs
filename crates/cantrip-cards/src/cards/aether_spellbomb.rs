use cantrip_api::*;
use cantrip_mechanics::{targets, effects};
use cantrip_oracle as oracle;

/// Return target creature to its owner's hand, if it is still a legal target.
fn bounce(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.return_to_hand(&[creature]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "{U}, Sacrifice Aether Spellbomb: Return target creature to its owner's hand.",
            cost: Cost { mana: mana(0, 0, 1, 0, 0, 0), sacrifice_self: true, ..Cost::NONE },
            targets: &[targets::CREATURE],
            effect: bounce,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{1}, Sacrifice Aether Spellbomb: Draw a card.",
            cost: Cost { mana: mana(1, 0, 0, 0, 0, 0), sacrifice_self: true, ..Cost::NONE },
            effect: effects::draw::<1>,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::AETHER_SPELLBOMB
};
