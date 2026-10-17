use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, library};
use cantrip_oracle as oracle;

/// "Mill four cards. You may put a creature card from among them into your hand."
fn mill_four_take_a_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::mill_then_may_take_into_hand(g, item.controller, item.object, 4, "creature", |c| c.is(CardType::Creature))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, |g, item| effects::draw(g, item, 1), "When this artifact enters, draw a card."),
        ability(
            cost("{1}{B}").and(SACRIFICE_THIS),
            mill_four_take_a_creature,
            "{1}{B}, Sacrifice this artifact: Mill four cards. You may put a creature card from among them into your hand.",
        ),
    ],
    ..oracle::EERIE_GRAVESTONE
};
