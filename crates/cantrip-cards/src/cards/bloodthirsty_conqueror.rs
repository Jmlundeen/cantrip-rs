use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Whenever an opponent loses life" (CR 119.3, 603.2): one event for each time life is lost, damage and paying life
/// included, and none when none is (a loss of 0, or damage that was prevented). "An opponent" is read against the
/// ability's controller (CR 603.7d-e), and a player who has left the game is no one's opponent (CR 800.4a). The trigger
/// carries how much was lost.
fn opponent_loses_life(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::LifeChange { player, delta } if *delta < 0 && q.state().opponents(me.controller).any(|p| p == *player) => {
            Some(Fired { subject: Target::Player(*player), data: [delta.unsigned_abs(), 0] })
        }
        _ => None,
    }
}

/// "You gain that much life."
fn gain_that_much(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, item.data[0] as i32);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when_asked(EventMask::LIFE_CHANGE, opponent_loses_life), gain_that_much, "Whenever an opponent loses life, you gain that much life.")],
    ..oracle::BLOODTHIRSTY_CONQUEROR
};
