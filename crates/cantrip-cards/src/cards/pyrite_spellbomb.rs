use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;
use cantrip_mechanics::targets;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{R}").and(SACRIFICE_THIS), effects::damage_first_target::<2>, "{R}, Sacrifice Pyrite Spellbomb: It deals 2 damage to any target.")
            .targets(&[targets::ANY]),
        ability(cost("{1}").and(SACRIFICE_THIS), |g, item| effects::draw(g, item, 1), "{1}, Sacrifice Pyrite Spellbomb: Draw a card."),
    ],
    ..oracle::PYRITE_SPELLBOMB
};
