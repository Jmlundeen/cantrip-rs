use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::lands::destroy_target_its_controller_may_fetch_basic;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// Destroy target land. Its controller may search their library for a basic land card, put it onto the battlefield, then
/// shuffle (see `destroy_target_its_controller_may_fetch_basic`).
fn destroy_and_search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target_its_controller_may_fetch_basic(g, item, 0, false)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana::tap_for(mana::add_c, Mana::C, "{T}: Add {C}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{T}, Sacrifice this land: Destroy target land. Its controller may search their library for a basic land card, put it onto the battlefield, then shuffle.",
            cost: Cost { tap: true, sacrifice_self: true, ..Cost::NONE },
            targets: &[targets::LAND],
            effect: destroy_and_search,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::GHOST_QUARTER
};
