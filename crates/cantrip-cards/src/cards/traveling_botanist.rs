use cantrip_api::*;
use cantrip_mechanics::library;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

const REVEAL: Question = Question::new(
    "RevealLandToHand",
    "reveal it and put it into your hand?",
    "It is a land card. If you don't, you may put it into your graveyard instead",
    "Reveal it",
    "Don't",
);

/// "Look at the top card of your library. If it's a land card, you may reveal it and put it into your hand. If you don't put the
/// card into your hand, you may put it into your graveyard." (CR 701.20a.)
fn look_at_the_top_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::look_at_top_may_take_else_bin(g, item, &REVEAL, |g, top| g.chars(top).is(CardType::Land))
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_BECOMES_TAPPED,
        look_at_the_top_card,
        "Whenever this creature becomes tapped, look at the top card of your library. If it's a land card, you may reveal it and put it into your hand. If you don't put the card into your hand, you may put it into your graveyard.",
    )],
    ..oracle::TRAVELING_BOTANIST
};
