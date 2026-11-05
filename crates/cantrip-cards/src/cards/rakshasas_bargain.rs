use cantrip_api::*;
use cantrip_mechanics::library::look_at_top_take_some_rest_to_graveyard;
use cantrip_oracle as oracle;

/// "Look at the top four cards of your library. Put two of them into your hand and the rest into your graveyard" (CR 701.20e,
/// 400.3): `look_at_top_take_some_rest_to_graveyard`.
fn look_at_four(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    look_at_top_take_some_rest_to_graveyard(g, item.controller, item.object, 4, 2)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], look_at_four),
    ..oracle::RAKSHASAS_BARGAIN
};
