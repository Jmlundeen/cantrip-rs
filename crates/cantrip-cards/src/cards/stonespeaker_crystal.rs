use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Exile any number of target players' graveyards." (CR 406.2, 608.2b): every card in the graveyard of each target
/// player still legal, in one move. With no targets nothing happens.
fn exile_target_players_graveyards(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cards: Vec<ObjectId> =
        targets::legal_players(g, item).into_iter().flat_map(|player| g.state().zone(Zone::graveyard(player)).to_vec()).collect();
    g.exile_all(&cards);
    Ok(())
}

/// "{2}, {T}, Sacrifice this artifact: Exile any number of target players' graveyards. Draw a card." (CR 608.2c, 121.1).
fn exile_then_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target_players_graveyards(g, item)?;
    g.end_sentence();
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_n(Mana::C, 2), "{T}: Add {C}{C}."),
        ability(cost("{2}, {T}").and(SACRIFICE_THIS), exile_then_draw, "{2}, {T}, Sacrifice this artifact: Exile any number of target players' graveyards. Draw a card.")
            .targets(&[targets::PLAYER.between(0, TargetSpec::ANY_NUMBER)]),
    ],
    ..oracle::STONESPEAKER_CRYSTAL
};
