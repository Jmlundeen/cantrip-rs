use cantrip_api::*;
use cantrip_mechanics::lands::destroy_target_its_controller_may_fetch_basic;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Destroy target creature or planeswalker. Its controller may search their library for a basic land card, put it onto
/// the battlefield tapped, then shuffle (see `destroy_target_its_controller_may_fetch_basic`).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target_its_controller_may_fetch_basic(g, item, 0, true)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_OR_PLANESWALKER], effect),
    ..oracle::ERODE
};
