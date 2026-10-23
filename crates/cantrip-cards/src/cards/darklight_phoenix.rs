use cantrip_api::*;
use cantrip_mechanics::history;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "If two or more creatures died this turn" (CR 603.4): an intervening if, asked as it triggers and again as it resolves.
fn two_creatures_died_this_turn(q: &dyn Query, _: &Asker) -> bool {
    history::creatures_died_this_turn(q, None) >= 2
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::STEP_BEGAN, triggers::beginning_of_your_combat),
        cantrip_mechanics::effects::return_this_card_to_battlefield,
        "At the beginning of combat on your turn, if two or more creatures died this turn, return this card from your graveyard to the battlefield.",
    )
    .condition(two_creatures_died_this_turn)
    // It returns this card from the graveyard, so it functions only there (CR 113.6m).
    .functions_in(IN_A_GRAVEYARD)],
    ..oracle::DARKLIGHT_PHOENIX
};
