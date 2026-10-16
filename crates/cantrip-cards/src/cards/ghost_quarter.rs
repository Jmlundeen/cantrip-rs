use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::lands::destroy_target_its_controller_may_fetch_basic;
use cantrip_oracle as oracle;

/// Destroy target land. Its controller may search their library for a basic land card, put it onto the battlefield, then
/// shuffle (see `destroy_target_its_controller_may_fetch_basic`).
fn destroy_and_search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target_its_controller_may_fetch_basic(g, item, 0, false)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(TAP.and(SACRIFICE_THIS), destroy_and_search, "{T}, Sacrifice this land: Destroy target land. Its controller may search their library for a basic land card, put it onto the battlefield, then shuffle.")
            .targets(&[targets::LAND]),
    ],
    ..oracle::GHOST_QUARTER
};
