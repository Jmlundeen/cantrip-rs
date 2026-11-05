use cantrip_api::*;
use cantrip_mechanics::{sacrifice_chosen, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of your end step, if this creature attacked this turn, sacrifice a nonland permanent." The intervening
/// "if" asks the history for its own attack this turn (CR 603.4, 508.1), and the permanent is chosen and sacrificed as the
/// ability resolves (CR 701.21a): its controller may pick the creature itself.
fn sacrifice_a_nonland_permanent(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    sacrifice_chosen(g, item.controller, item.object, 1, |c| !c.is(CardType::Land))?;
    Ok(())
}

fn attacked_this_turn(q: &dyn Query, asker: &Asker) -> bool {
    q.attacked_this_turn(asker.source)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOUR_END_STEP,
        sacrifice_a_nonland_permanent,
        "At the beginning of your end step, if this creature attacked this turn, sacrifice a nonland permanent.",
    )
    .condition(attacked_this_turn)],
    ..oracle::TARG_WRANGLER
};
