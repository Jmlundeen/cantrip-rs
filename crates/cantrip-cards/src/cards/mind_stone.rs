use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana::tap_for(mana::add_c, Mana::C, "{T}: Add {C}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{1}, {T}, Sacrifice this artifact: Draw a card.",
            cost: Cost { mana: mana(1, 0, 0, 0, 0, 0), tap: true, sacrifice_self: true, ..Cost::NONE },
            effect: cantrip_mechanics::effects::draw::<1>,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::MIND_STONE
};
