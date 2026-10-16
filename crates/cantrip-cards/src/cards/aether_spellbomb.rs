use cantrip_api::*;
use cantrip_mechanics::{targets, effects};
use cantrip_oracle as oracle;

/// Return target creature to its owner's hand, if it is still a legal target.
fn bounce(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.return_to_hand(&[creature]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{U}").and(SACRIFICE_THIS), bounce, "{U}, Sacrifice Aether Spellbomb: Return target creature to its owner's hand.")
            .targets(&[targets::CREATURE]),
        ability(cost("{1}").and(SACRIFICE_THIS), |g, item| effects::draw(g, item, 1), "{1}, Sacrifice Aether Spellbomb: Draw a card."),
    ],
    ..oracle::AETHER_SPELLBOMB
};
