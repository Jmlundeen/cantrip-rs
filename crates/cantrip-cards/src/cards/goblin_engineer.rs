use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::library::{search_to_graveyard, Find};
use cantrip_mechanics::targets::card_in_graveyard;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

const ASK: Question = Question::new("SearchLibrary", "search your library for an artifact card?", "It goes into your graveyard, then you shuffle", "Search", "Don't search");

/// When Goblin Engineer enters, you may search your library for an artifact card, put it into your graveyard, then
/// shuffle. Declining skips the search and the shuffle.
fn search_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    if !cantrip_mechanics::effects::ask_may(g, player, item.object, Purpose::Ask(&ASK))? {
        return Ok(());
    }
    search_to_graveyard(g, player, item.object, Find::ONE, |q, c| q.chars(c).is(CardType::Artifact))?;
    Ok(())
}

/// "Target artifact card with mana value 3 or less from your graveyard".
fn cheap_artifact_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    card_in_graveyard(q, t, &[CardType::Artifact])
        && matches!(t, Target::Object(id) if q.obj(id).owner == asker.controller && q.mana_value(id) <= 3)
}

/// Return target artifact card from your graveyard to the battlefield.
fn reanimate_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.put_onto_battlefield(card, item.controller, false)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, search_artifact, "When this creature enters, you may search your library for an artifact card, put it into your graveyard, then shuffle."),
        ability(cost("{R}, {T}").and(sacrifice(1, targets::filters::artifact)), reanimate_artifact, "{R}, {T}, Sacrifice an artifact: Return target artifact card with mana value 3 or less from your graveyard to the battlefield.")
            .targets(&[target(cheap_artifact_in_your_graveyard)]),
    ],
    ..oracle::GOBLIN_ENGINEER
};
