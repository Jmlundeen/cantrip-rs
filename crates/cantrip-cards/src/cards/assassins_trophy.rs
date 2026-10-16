use cantrip_api::*;
use cantrip_mechanics::lands::destroy_target_its_controller_may_fetch_basic;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Destroy target permanent an opponent controls. Its controller may search their library for a basic land card, put it
/// onto the battlefield, then shuffle." (see `destroy_target_its_controller_may_fetch_basic`): the controller is asked after
/// the destruction even when the permanent survived (indestructible, regenerated), the basic land enters untapped, and with
/// the target illegal the spell does not resolve and nobody searches (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target_its_controller_may_fetch_basic(g, item, 0, false)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PERMANENT_AN_OPPONENT_CONTROLS], effect),
    ..oracle::ASSASSINS_TROPHY
};
