use cantrip_api::*;
use cantrip_mechanics::{effects, exile_target, targets};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

fn exile_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    Ok(())
}

fn exile_graveyards(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::exile_each_opponents_graveyard(g, item.controller);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(TAP.and(SACRIFICE_THIS), exile_graveyards, "{T}, Sacrifice Soul-Guide Lantern: Exile each opponent's graveyard."),
        ability(cost("{1}, {T}").and(SACRIFICE_THIS), |g, item| effects::draw(g, item, 1), "{1}, {T}, Sacrifice Soul-Guide Lantern: Draw a card."),
        triggered(THIS_ENTERS, exile_it, "When Soul-Guide Lantern enters, exile target card from a graveyard.")
            .targets(&[targets::CARD_IN_A_GRAVEYARD]),
    ],
    ..oracle::SOUL_GUIDE_LANTERN
};
