use cantrip_api::*;
use cantrip_mechanics::library::{look_at_top, Rest};
use cantrip_mechanics::artifacts_you_control;
use cantrip_oracle as oracle;

/// Look at the top X cards of your library, where X is the number of artifacts you control as this resolves. Put one of
/// those cards into your hand and the rest on the bottom of your library in a random order. No artifacts is looking at
/// nothing, which does nothing.
fn look(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let artifacts = artifacts_you_control(g, player);
    look_at_top(g, player, item.object, artifacts, (1, 1), false, |_| true, Rest::BottomRandom)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(cost("{3}, {T}").and(discard_cards(1)), look, "{3}, {T}, Discard a card: Look at the top X cards of your library, where X is the number of artifacts you control. Put one of those cards into your hand and the rest on the bottom of your library in a random order."),
    ],
    ..oracle::FOMORI_VAULT
};
