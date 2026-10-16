use cantrip_api::*;
use cantrip_mechanics::lands::{destroy_target_its_controller_may_fetch_basic, may_fetch_basic};
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Destroy target nonbasic land an opponent controls. That land's controller may search their library for a basic land
/// card, put it onto the battlefield, then shuffle. You may search your library for a basic land card, put it onto the
/// battlefield, then shuffle." (CR 701.8, 701.23.) With its only target illegal the engine does not resolve the ability
/// at all (CR 608.2b); otherwise both searches are offered even if the land was not destroyed.
fn destroy_and_search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target_its_controller_may_fetch_basic(g, item, 0, false)?;
    may_fetch_basic(g, item.controller, item.object, false)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(cost("{2}").and(TAP).and(SACRIFICE_THIS), destroy_and_search, "{2}, {T}, Sacrifice this land: Destroy target nonbasic land an opponent controls. That land's controller may search their library for a basic land card, put it onto the battlefield, then shuffle. You may search your library for a basic land card, put it onto the battlefield, then shuffle.")
            .targets(&[targets::NONBASIC_LAND_AN_OPPONENT_CONTROLS]),
    ],
    ..oracle::DEMOLITION_FIELD
};
