use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Whenever an opponent draws a card": that player is the ability's subject, who loses the life.
fn opponent_draws(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Drew { player, .. } if *player != q.controller(source) => Fired::about(Target::Player(*player)),
        _ => None,
    }
}

fn lose_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Player(player) = item.subject {
        g.lose_life(player, 2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(cantrip_mechanics::triggers::YOU_DRAW_A_CARD, |g, item| effects::gain_life(g, item, 2), "Whenever you draw a card, you gain 2 life."),
        triggered(when(EventMask::DREW, opponent_draws), lose_two, "Whenever an opponent draws a card, they lose 2 life."),
    ],
    ..oracle::SHEOLDRED_THE_APOCALYPSE
};
