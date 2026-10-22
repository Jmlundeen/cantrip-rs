use cantrip_api::*;
use cantrip_mechanics::{library, triggers};
use cantrip_oracle as oracle;

const NAME: &str = "Faerie Miscreant";

/// "If you control another creature named Faerie Miscreant" (CR 603.4): checked as the ability would trigger and again as
/// it resolves, off the board as it stands. "Another" is any creature but this object; a name is the one it has now
/// (CR 201.2a), so a copy's or an effect's counts (CR 612.8).
fn control_another_named(q: &dyn Query, asker: &Asker) -> bool {
    q.permanents_of(asker.controller)
        .into_iter()
        .any(|id| id != asker.source && q.chars(id).is(CardType::Creature) && library::object_names(q, id).contains(&NAME))
}

/// "Draw a card."
fn draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        draw,
        "When this creature enters, if you control another creature named Faerie Miscreant, draw a card.",
    )
    .condition(control_another_named)],
    ..oracle::FAERIE_MISCREANT
};
