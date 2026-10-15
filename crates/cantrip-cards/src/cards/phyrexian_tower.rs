use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_mechanics::targets::filters;
use cantrip_oracle as oracle;

fn add_bb(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, Mana::B, 2);
    Ok(())
}

/// "{T}, Sacrifice a creature: Add {B}{B}." A mana ability whose sacrifice comes with the activation.
pub static CARD: CardDef = CardDef {
    abilities: &[
        mana::tap_for(mana::add_c, Mana::C, "{T}: Add {C}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{T}, Sacrifice a creature: Add {B}{B}.",
            cost: Cost {
                tap: true,
                permanents: Some(PermanentCost { count: 1, filter: filters::creature, to_hand: false }),
                ..Cost::NONE
            },
            mana: Some(add_bb),
            adds: ManaPool([0, 0, 2, 0, 0, 0]),
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::PHYREXIAN_TOWER
};
