use cantrip_api::*;
use cantrip_mechanics::{effects, hand, lands_you_control, targets, triggers::THIS_LEAVES, warp};
use cantrip_oracle as oracle;

/// "Draw a card, then you may put a permanent card with mana value less than or equal to the number of lands you control
/// from your hand onto the battlefield tapped." The card is drawn first, so it may be the one put; the lands are counted
/// after the draw, as the ability resolves (CR 608.2c); one permanent card (CR 110.4a), of the hand with all of it shown,
/// "up to one". Its mana value is `Query::mana_value` (X is 0 in a hand, CR 107.3b), and it enters tapped (CR 614.1d).
fn draw_then_put(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::draw(g, item, 1)?;
    let most = lands_you_control(&*g, item.controller) as u32;
    hand::may_put_onto_battlefield_tapped(g, item.controller, item.object, |g, c| {
        targets::is_permanent_card(&g.chars(c)) && g.mana_value(c) <= most
    })?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_LEAVES,
        draw_then_put,
        "When this creature leaves the battlefield, draw a card, then you may put a permanent card with mana value less than or equal to the number of lands you control from your hand onto the battlefield tapped.",
    )],
    ways: &[warp::way(mana("{4}"), "Warp {4}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::ANTICAUSAL_VESTIGE
};
