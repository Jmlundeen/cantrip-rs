use cantrip_api::*;
use cantrip_mechanics::targets::filters;
use cantrip_oracle as oracle;

const ASK: Question = Question::new("Discard", "discard a land card?", "If you do, Mox Diamond enters the battlefield. If you don't, it goes to your graveyard", "Discard a land", "Mox to graveyard");

/// "If this artifact would enter, you may discard a land card instead. If you do, put this artifact onto the battlefield. If
/// you don't, put it into its owner's graveyard." A replacement with a choice inside it (CR 614.12a), settled before the card
/// moves, wherever it is. The discard is a cost paid as it applies (CR 118.12), and a cost that names a land card: the land
/// can't be put into a hidden zone unrevealed (CR 701.9c), which the payment sees to. A player with no land card in hand is
/// never asked: the payment is impossible, and it goes to the graveyard.
fn enters(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let player = entering.controller;
    if !g.may_pay(player, entering.object, entering.object, discard(1, filters::land), &ASK)? {
        entering.instead = Some(Zone::graveyard(g.obj(entering.object).owner));
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters),
    abilities: &[mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color.")],
    ..oracle::MOX_DIAMOND
};
