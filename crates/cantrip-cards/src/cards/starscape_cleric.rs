use cantrip_api::*;
use cantrip_mechanics::offspring;
use cantrip_mechanics::triggers::YOU_GAIN_LIFE;
use cantrip_oracle as oracle;

/// "Each opponent loses 1 life."
fn each_opponent_loses_a_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.each_opponent_loses_life(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    keywords: Keywords::of(&[Keyword::Flying, Keyword::CantBlock]),
    kicker: Some(offspring::kicker(mana("{2}{B}"))),
    abilities: &[
        offspring::trigger("Offspring {2}{B} (You may pay an additional {2}{B} as you cast this spell. If you do, when this creature enters, create a 1/1 token copy of it.)"),
        triggered(YOU_GAIN_LIFE, each_opponent_loses_a_life, "Whenever you gain life, each opponent loses 1 life."),
    ],
    ..oracle::STARSCAPE_CLERIC
};
