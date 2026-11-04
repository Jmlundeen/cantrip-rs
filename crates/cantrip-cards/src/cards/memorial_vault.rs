use cantrip_api::*;
use cantrip_mechanics::library::exile_top_play_this_turn;
use cantrip_mechanics::targets::filters;
use cantrip_oracle as oracle;

/// "Exile the top X cards of your library, where X is one plus the mana value of the sacrificed artifact. You may play those
/// cards this turn." (CR 406.3, 305.1, 601.2, 514.2): the artifact is the one this activation's cost sacrificed
/// (`Query::cost_objects`, CR 602.2b), its mana value as it last existed on the battlefield (CR 608.2h) - an {X} in its cost
/// counting 0 (CR 202.3e), a token's 0 - so X is never below one. The permission is to play, so a land among the cards may be
/// the turn's land drop. A library with fewer cards than X is exiled whole.
fn exile_top_cards(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let sacrificed: u32 = g.cost_objects(item).iter().map(|&artifact| g.mana_value(artifact)).sum();
    exile_top_play_this_turn(g, item.controller, sacrificed as usize + 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{T}").and(sacrifice(1, filters::another_artifact)),
        exile_top_cards,
        "{T}, Sacrifice another artifact: Exile the top X cards of your library, where X is one plus the mana value of the sacrificed artifact. You may play those cards this turn.",
    )],
    ..oracle::MEMORIAL_VAULT
};
