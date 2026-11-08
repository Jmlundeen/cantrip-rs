use cantrip_api::*;
use cantrip_oracle as oracle;

/// "At the beginning of each player's end step" (CR 513.1, 603.2): any player's end step, about the player whose end step
/// it is.
fn each_players_end_step(q: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::StepBegan { step: Step::End } => Fired::about(Target::Player(q.state().turn.active)),
        _ => None,
    }
}

/// "that player draws a card. If the player is your opponent and has four or more cards in hand, this enchantment deals 2
/// damage to that player." (CR 121.1, 603.2): the draw happens first and the hand size is read after it, with no action
/// between the two steps.
fn draw_and_punish(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(player) = item.subject else { return Ok(()) };
    g.draw(player, 1);
    g.end_sentence();
    if player != item.controller && g.state().zone(Zone::hand(player)).len() >= 4 {
        g.damage(item.object, Target::Player(player), 2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when(EventMask::STEP_BEGAN, each_players_end_step),
        draw_and_punish,
        "At the beginning of each player's end step, that player draws a card. If the player is your opponent and has four or more cards in hand, this enchantment deals 2 damage to that player.",
    )],
    ..oracle::FEVERED_VISIONS
};
