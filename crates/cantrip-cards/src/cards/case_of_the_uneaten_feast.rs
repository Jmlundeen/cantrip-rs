use cantrip_api::*;
use cantrip_mechanics::{effects, solved, triggers};
use cantrip_oracle as oracle;

/// "To solve — You've gained 5 or more life this turn" (CR 719.3a), an intervening "if" asked as the end step begins and as
/// the ability resolves: the life the Case's controller has gained this turn, from the history (CR 119.3).
fn gained_five_this_turn(q: &dyn Query, asker: &Asker) -> bool {
    q.history().life_gained(asker.controller) >= 5
}

/// "Creature cards in your graveyard gain 'You may cast this card from your graveyard' until end of turn": the creature cards
/// there as the ability resolves (CR 611.2c), each castable from there by its owner, you, for its mana cost and at the
/// time its type allows (CR 601.3). A card that leaves the graveyard is a new object without it (CR 400.7).
fn creatures_castable(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let creatures: Vec<ObjectId> =
        g.state().zone(Zone::graveyard(you)).iter().copied().filter(|&card| g.chars(card).is(CardType::Creature)).collect();
    for card in creatures {
        g.allow_play(card, you, PlayGrant { expiry: Expiry::EndOfTurn, ..PlayGrant::CAST });
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::CREATURE_YOU_CONTROL_ENTERS, |g, item| effects::gain_life(g, item, 1), "Whenever a creature you control enters, you gain 1 life."),
        solved::to_solve(gained_five_this_turn, "To solve \u{2014} You've gained 5 or more life this turn. (If unsolved, solve at the beginning of your end step.)"),
        solved::ability(SACRIFICE_THIS, creatures_castable, "Solved \u{2014} Sacrifice this Case: Creature cards in your graveyard gain \"You may cast this card from your graveyard\" until end of turn."),
    ],
    ..oracle::CASE_OF_THE_UNEATEN_FEAST
};
