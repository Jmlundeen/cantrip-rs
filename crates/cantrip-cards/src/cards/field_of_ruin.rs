use cantrip_api::*;
use cantrip_mechanics::{lands, targets};
use cantrip_oracle as oracle;

/// "Destroy target nonbasic land an opponent controls. Each player searches their library for a basic land card, puts it
/// onto the battlefield, then shuffles." (CR 701.8, 701.23.) With its only target illegal the engine does not resolve the
/// ability at all (CR 608.2b); otherwise every player searches even if the land was not destroyed (ruling 2021-03-19). The
/// players search together: they choose in turn order, the lands enter at once, then each shuffles (CR 101.4, 701.23i).
fn destroy_and_search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(land) = targets::object(g, item, 0) else { return Ok(()) };
    g.destroy(land);
    g.end_sentence();
    let searchers: Vec<(PlayerId, u32)> = g.state().apnap().map(|p| (p, 1)).collect();
    lands::each_searches_basic_lands_together(g, &searchers, item.object, false)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(cost("{2}").and(TAP).and(SACRIFICE_THIS), destroy_and_search, "{2}, {T}, Sacrifice this land: Destroy target nonbasic land an opponent controls. Each player searches their library for a basic land card, puts it onto the battlefield, then shuffles.")
            .targets(&[targets::NONBASIC_LAND_AN_OPPONENT_CONTROLS]),
    ],
    ..oracle::FIELD_OF_RUIN
};
