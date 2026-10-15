use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Whenever you draw a card" (CR 120.3, 603.2).
fn you_draw(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Drew { player, .. } if *player == q.controller(source) => Fired::PLAIN,
        _ => None,
    }
}

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
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever you draw a card, you gain 2 life.",
            listens: EventMask::DREW,
            matches: you_draw,
            effect: effects::gain_life::<2>,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever an opponent draws a card, they lose 2 life.",
            listens: EventMask::DREW,
            matches: opponent_draws,
            effect: lose_two,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::SHEOLDRED_THE_APOCALYPSE
};
