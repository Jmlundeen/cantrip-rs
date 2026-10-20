use cantrip_api::*;
use cantrip_mechanics::{
    effects::{may_draw, may_draw_then_discard},
    history::entered_this_turn,
    ninjutsu::ninjutsu,
    triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER,
};
use cantrip_oracle as oracle;

const MAY_DRAW: Question = Question::new("Draw", "draw a card?", "It entered this turn: you discard nothing", "Draw", "Don't draw");

/// "Whenever this creature deals combat damage to a player, you may draw a card. If you do, discard a card unless this
/// creature entered this turn." (CR 608.2c, 121.3). Whether it entered this turn is asked of the event history, which keeps
/// the answer for an object that has since left (CR 400.7, last-known information). Entered this turn, it is only "you may
/// draw a card"; otherwise it is the loot the shared recipe makes.
fn loot_unless_entered(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if entered_this_turn(g.history(), item.object) {
        may_draw(g, item.controller, item.object, 1, &MAY_DRAW)?;
        Ok(())
    } else {
        may_draw_then_discard(g, item)
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ninjutsu(
            mana("{U}"),
            "Ninjutsu {U} ({U}, Return an unblocked attacker you control to hand: Put this card onto the battlefield from your hand tapped and attacking.)",
        ),
        triggered(DEALS_COMBAT_DAMAGE_TO_PLAYER, loot_unless_entered, "Whenever this creature deals combat damage to a player, you may draw a card. If you do, discard a card unless this creature entered this turn."),
    ],
    ..oracle::MOON_CIRCUIT_HACKER
};
