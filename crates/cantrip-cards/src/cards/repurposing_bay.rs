use cantrip_api::*;
use cantrip_mechanics::library::{search_to_battlefield, Find};
use cantrip_oracle as oracle;

/// "Another artifact" (CR 109.1) as a sacrifice cost: an artifact the player controls other than the Bay itself.
fn another_artifact(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    id != asker.source && q.chars(id).is(CardType::Artifact)
}

/// "Search your library for an artifact card with mana value equal to 1 plus the sacrificed artifact's mana value, put that card
/// onto the battlefield, then shuffle" (CR 701.23, 202.3): the sacrificed artifact is read as it last was on the battlefield
/// (CR 608.2h), so a token (mana value 0) or a copy counts as the rules say (`Query::mana_value`). The search may find nothing
/// (CR 701.23b); the library is shuffled either way.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(&sacrificed) = g.cost_objects(item).first() else { return Ok(()) };
    let wanted = g.mana_value(sacrificed) + 1;
    search_to_battlefield(g, item.controller, item.object, Find::ONE, false, |q, card| {
        q.chars(card).is(CardType::Artifact) && q.mana_value(card) == wanted
    })?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{2}, {T}").and(sacrifice(1, another_artifact)), search, "{2}, {T}, Sacrifice another artifact: Search your library for an artifact card with mana value equal to 1 plus the sacrificed artifact's mana value, put that card onto the battlefield, then shuffle. Activate only as a sorcery.")
        .sorcery_speed()],
    ..oracle::REPURPOSING_BAY
};
