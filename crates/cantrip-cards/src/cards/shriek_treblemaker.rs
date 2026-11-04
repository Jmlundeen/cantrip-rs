use cantrip_api::*;
use cantrip_mechanics::triggers::{self, YOUR_FIRST_MAIN_PHASE};
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "At the beginning of your first main phase, you may discard a card. When you do, target creature can't block this turn."
/// (CR 701.9, 603.12): the optional discard happens on resolution; only then is the reflexive ability created and its target
/// chosen.
fn discard_for_cant_block(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if effects::may_discard_a_card(g, item)? {
        let this = g.card_id(&CARD);
        g.reflexive_trigger(item.object, item.controller, AbilityRef::delayed(this, 0), Target::None);
    }
    Ok(())
}

/// "Shriek deals 1 damage to that player" (CR 120.3a).
fn damage_that_player(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.damage(item.object, item.subject, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(YOUR_FIRST_MAIN_PHASE, discard_for_cant_block, "At the beginning of your first main phase, you may discard a card. When you do, target creature can't block this turn."),
        triggered(triggers::CREATURE_AN_OPPONENT_CONTROLS_DIES, damage_that_player, "Sonic Blast — Whenever a creature an opponent controls dies, Shriek deals 1 damage to that player."),
    ],
    delayed: &[delayed_trigger(effects::first_target_cant_block_this_turn, "When you do, target creature can't block this turn.")
        .targets(&[targets::CREATURE])],
    ..oracle::SHRIEK_TREBLEMAKER
};
