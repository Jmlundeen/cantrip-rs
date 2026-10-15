use cantrip_api::*;
use cantrip_mechanics::vote;
use cantrip_oracle as oracle;

/// Will of the council (CR 701.38a): starting with you, each player votes for a nonland permanent you don't control;
/// every permanent tied for the most votes is exiled together.
fn will_of_the_council(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let candidates: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.controller(id) != you && !g.chars(id).is(CardType::Land))
        .collect();
    if candidates.is_empty() {
        return Ok(());
    }
    let votes = vote::for_permanents(g, item.object, you, &candidates)?;
    g.exile_all(&vote::most_votes(&candidates, &votes));
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: Some(SpellDef { targets: &[], effect: will_of_the_council }), ..oracle::COUNCILS_JUDGMENT };
