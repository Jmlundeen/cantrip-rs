use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Draw a card" (CR 121.1).
fn draw_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::draw(g, item, 1)
}

/// "Another artifact or land" (CR 109.1, 701.21a): an artifact or a land (an artifact land counts once) other than the Scrapper.
fn another_artifact_or_land(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    let c = q.chars(id);
    id != asker.source && (c.is(CardType::Artifact) || c.is(CardType::Land))
}

/// "{2}, {T}, Sacrifice another artifact or land: Draw a card." The sacrifice is part of the cost (CR 602.2, 118.3), and "another"
/// keeps the Scrapper itself, an artifact, from paying it.
pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{2}").and(TAP).and(Cost::sacrifice(1, another_artifact_or_land)),
        draw_a_card,
        "{2}, {T}, Sacrifice another artifact or land: Draw a card.",
    )],
    ..oracle::SLAGDRILL_SCRAPPER
};
