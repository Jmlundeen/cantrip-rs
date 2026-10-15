use cantrip_api::*;
use cantrip_mechanics::storm;
use cantrip_mechanics::library::exile_top_may_play;
use cantrip_oracle as oracle;

/// Shuffle your library. Then exile the top card of your library. Until end of turn, you may play that card without
/// paying its mana cost (CR 118.9, 601.3), a land among it under the ordinary land-play rules (CR 305.2). Each storm
/// copy resolves the whole sequence again for its own controller.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    g.shuffle(player);
    let grant = PlayGrant { instead: Some(ManaCost::ZERO), lands: true, expiry: Expiry::EndOfTurn, ..PlayGrant::CAST };
    exile_top_may_play(g, player, 1, player, grant);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    abilities: &[storm::trigger("Storm (When you cast Mind's Desire, copy it for each spell cast before it this turn.)")],
    ..oracle::MINDS_DESIRE
};
