use cantrip_api::*;
use cantrip_mechanics::history::gained_life_this_turn;
use cantrip_mechanics::triggers::YOUR_END_STEP;
use cantrip_mechanics::{effects, tokens};
use cantrip_oracle as oracle;

/// "If you lost life this turn" (CR 603.4, 119.3): any life lost by its controller, however much they also gained.
fn lost_life_this_turn(q: &dyn Query, player: PlayerId) -> bool {
    q.history().life_lost(player) > 0
}

/// "If you gained life this turn": the intervening "if" of the first end-step ability, checked as it triggers and as it resolves.
fn gained_life(q: &dyn Query, asker: &Asker) -> bool {
    gained_life_this_turn(q, asker.controller)
}

/// "If you gained and lost life this turn": the intervening "if" of the second, both halves of it.
fn gained_and_lost_life(q: &dyn Query, asker: &Asker) -> bool {
    gained_life_this_turn(q, asker.controller) && lost_life_this_turn(q, asker.controller)
}

/// "Each opponent loses 1 life."
fn each_opponent_loses_a_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.each_opponent_loses_life(item.controller, 1);
    Ok(())
}

/// "Create a 1/1 black Bat creature token with flying."
fn create_a_bat(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::BAT, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(YOUR_END_STEP, each_opponent_loses_a_life, "At the beginning of your end step, if you gained life this turn, each opponent loses 1 life.")
            .condition(gained_life),
        triggered(YOUR_END_STEP, create_a_bat, "At the beginning of your end step, if you gained and lost life this turn, create a 1/1 black Bat creature token with flying.")
            .condition(gained_and_lost_life),
        ability(cost("{1}{B}").and(life(2)), |g, item| effects::draw(g, item, 1), "{1}{B}, Pay 2 life: Draw a card."),
    ],
    ..oracle::LUNAR_CONVOCATION
};

pub static EXTRA: &[&CardDef] = &[&tokens::BAT];
