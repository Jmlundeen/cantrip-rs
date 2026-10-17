use cantrip_api::*;
use cantrip_mechanics::tokens::{self, investigate};
use cantrip_mechanics::triggers::YOUR_UPKEEP;
use cantrip_oracle as oracle;

/// "Investigate once for each opponent who has more cards in hand than you" (CR 701.16a): the opponents still in the game
/// and the hands are counted as the ability resolves (CR 608.2c), one Clue each.
fn investigate_for_each_opponent_with_more_cards(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let yours = g.state().zone(Zone::hand(item.controller)).len();
    let opponents = g.state().opponents(item.controller).filter(|&p| g.state().zone(Zone::hand(p)).len() > yours).count();
    for _ in 0..opponents {
        investigate(g, item.controller)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOUR_UPKEEP,
        investigate_for_each_opponent_with_more_cards,
        "At the beginning of your upkeep, investigate once for each opponent who has more cards in hand than you. (To investigate, create a Clue token. It's an artifact with \"{2}, Sacrifice this token: Draw a card.\")",
    )],
    ..oracle::WOJEK_INVESTIGATOR
};

pub static EXTRA: &[&CardDef] = &[&tokens::CLUE];
