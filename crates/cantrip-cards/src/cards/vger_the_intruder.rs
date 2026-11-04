use cantrip_api::*;
use cantrip_mechanics::{hand, targets, triggers};
use cantrip_oracle as oracle;

/// Mode 0: "look at target opponent's hand, then draw a card" (CR 701.20b): the look, then the card.
fn look_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    hand::look_at_target_players_hand(g, item)?;
    g.draw(item.controller, 1);
    Ok(())
}

/// Mode 1: "creatures your opponents control get -1/-0 until your next turn" (CR 611.2, 611.2c): each creature an opponent
/// controls as it resolves, as this engine's set-wide pumps do (`cantrip_mechanics::effects::pump_creatures_you_control` and
/// its siblings, which no creature that enters later gets), each with the mode's own duration rather than end of turn.
fn weaken_their_creatures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let theirs: Vec<ObjectId> = g
        .state()
        .zone(Zone::BATTLEFIELD)
        .iter()
        .copied()
        .filter(|&id| g.state().obj(id).controller != item.controller && g.chars(id).is(CardType::Creature))
        .collect();
    instruction(g, |g| {
        for id in theirs {
            g.add_effect(item.object, id, EffectKind::Pump(-1, 0), Duration::UntilNextTurnOf(item.controller));
        }
    });
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(look_and_draw, "Look at target opponent's hand, then draw a card.").targets(&[targets::OPPONENT]),
    mode(weaken_their_creatures, "Creatures your opponents control get -1/-0 until your next turn."),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(
        triggers::THIS_ENTERS_OR_ATTACKS,
        &MODAL,
        "Whenever V'Ger enters or attacks, choose one — Look at target opponent's hand, then draw a card; or creatures your opponents control get -1/-0 until your next turn.",
    )],
    ..oracle::VGER_THE_INTRUDER
};
