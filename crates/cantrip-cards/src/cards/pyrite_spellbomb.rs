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
        ability(cost("{R}").and(SACRIFICE_THIS), damage, "{R}, Sacrifice Pyrite Spellbomb: It deals 2 damage to any target.")
            .targets(&[targets::ANY]),
        ability(cost("{1}").and(SACRIFICE_THIS), |g, item| effects::draw(g, item, 1), "{1}, Sacrifice Pyrite Spellbomb: Draw a card."),
    ],
    ..oracle::PYRITE_SPELLBOMB
};
