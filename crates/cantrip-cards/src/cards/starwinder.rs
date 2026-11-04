use cantrip_api::*;
use cantrip_mechanics::triggers::creature_you_control_deals_combat_damage_to_player_for_amount;
use cantrip_mechanics::{effects, warp};
use cantrip_oracle as oracle;

const MAY_DRAW_THAT_MANY: Question = Question::new("Draw", "draw that many cards?", "One for each point of damage", "Draw", "Don't draw");

/// "You may draw that many cards" (CR 121.3, 608.2d): the trigger's `data[0]` is the number, the damage dealt (the trigger
/// fires only for damage above 0); asked of the ability's controller, not offered a player who can't draw that many
/// (`may_draw`).
fn may_draw_that_many(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::may_draw(g, item.controller, item.object, item.data[0], &MAY_DRAW_THAT_MANY)?;
    Ok(())
}

/// "Whenever a creature you control deals combat damage to a player, you may draw that many cards" (CR 510.2, 603.2): one
/// trigger for each creature, drawing as many as that creature dealt. Warp {2}{U}{U} (CR 702.185a).
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::DAMAGE, creature_you_control_deals_combat_damage_to_player_for_amount),
        may_draw_that_many,
        "Whenever a creature you control deals combat damage to a player, you may draw that many cards.",
    )],
    ways: &[warp::way(mana("{2}{U}{U}"), "Warp {2}{U}{U}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::STARWINDER
};
