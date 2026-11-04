use cantrip_api::*;
use cantrip_mechanics::replacements::an_opponent_would_mill;
use cantrip_mechanics::cost_reduction;
use cantrip_oracle as oracle;

/// "... they mill that many cards plus four instead" (CR 614.1a). Each such effect applies once (CR 614.5), so a second
/// Crystal adds four more.
fn that_many_plus_four(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::Mill(m) = proposal {
        m.count += 4;
    }
}

/// "{4}{U}{U}, {T}: Each opponent mills cards equal to the number of cards in your hand." The count is read once, as the
/// ability resolves, and every opponent mills at once (CR 701.17a, 101.4).
fn opponents_mill_your_hand_size(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let count = g.state().zone(Zone::hand(item.controller)).len() as u32;
    let mills: Vec<(PlayerId, u32)> = g.state().opponents(item.controller).map(|o| (o, count)).collect();
    g.mill_together(&mills);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    cost_changes: &[cost_reduction::colored_spells_you_cast_cost_one_less::<{ cost_reduction::color(Color::Blue) }>],
    replacements: &[replacement(ReplacementKind::Mill, an_opponent_would_mill, that_many_plus_four, "mills that many cards plus four instead")],
    abilities: &[ability(
        cost("{4}{U}{U}, {T}"),
        opponents_mill_your_hand_size,
        "{4}{U}{U}, {T}: Each opponent mills cards equal to the number of cards in your hand.",
    )],
    ..oracle::THE_WATER_CRYSTAL
};
