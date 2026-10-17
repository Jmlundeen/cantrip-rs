use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Any number of target players" (CR 115.1d, 601.2c): none, or each player once (CR 115.3), with what the slot is for.
const fn any_number_of_players(label: &'static str) -> TargetSpec {
    targets::PLAYER.between(0, TargetSpec::ANY_NUMBER).labelled(label)
}

/// The targeted players still legal as the ability resolves (CR 608.2b), in the order they were chosen.
fn legal_players(g: &dyn Game, item: &StackItem) -> Vec<PlayerId> {
    (0..item.target_count as usize).filter_map(|slot| targets::player(g, item, slot)).collect()
}

/// "{1}, {T}: Any number of target players each mill two cards." One simultaneous mill (CR 701.17, 101.4).
fn mill_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let mills: Vec<(PlayerId, u32)> = legal_players(g, item).into_iter().map(|p| (p, 2)).collect();
    g.mill_together(&mills);
    Ok(())
}

/// "Exhaust — {2}{U}{U}, {T}: Any number of target players each mill cards equal to the number of cards in their
/// graveyard." Each count is that player's graveyard as the ability resolves, read before anyone mills, and they all mill
/// at once (CR 101.4, 701.17).
fn mill_graveyard_size(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let mills: Vec<(PlayerId, u32)> =
        legal_players(g, item).into_iter().map(|p| (p, g.state().zone(Zone::graveyard(p)).len() as u32)).collect();
    g.mill_together(&mills);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{1}, {T}"), mill_two, "{1}, {T}: Any number of target players each mill two cards.")
            .targets(&[any_number_of_players("Players who each mill two cards")]),
        ability(
            cost("{2}{U}{U}, {T}"),
            mill_graveyard_size,
            "Exhaust \u{2014} {2}{U}{U}, {T}: Any number of target players each mill cards equal to the number of cards in their graveyard. (Activate each exhaust ability only once.)",
        )
        .targets(&[any_number_of_players("Players who each mill as many cards as their graveyard holds")])
        .exhaust(),
    ],
    ..oracle::RIVERCHURN_MONUMENT
};
