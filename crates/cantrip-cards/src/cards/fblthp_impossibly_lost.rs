use cantrip_api::*;
use cantrip_mechanics::your_turn;
use cantrip_oracle as oracle;

/// Combat damage to opponents during your turn; `.once_per_batch()` groups simultaneous hits (CR 603.2c).
fn opponents_dealt_combat_damage(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { target: Target::Player(p), amount, kind, .. }
            if kind.is_combat() && *amount > 0 && your_turn(q, me) && q.state().opponents(me.controller).any(|o| o == *p) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Draw two cards. If your library has no cards in it, you win the game. Fblthp's owner shuffles him into their library."
/// The win is checked as the ability resolves, before the state-based actions could make an empty-library draw lose
/// (CR 704.5b); if the game is over the ability stops there. Otherwise Fblthp is shuffled into his owner's library: the
/// move needs the original object (CR 400.7), but the library is shuffled either way (CR 701.24c).
fn draw_win_and_shuffle(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.draw(you, 2);
    if g.state().zone(Zone::library(you)).is_empty() {
        g.win_game(you);
        if g.state().is_over() {
            return Ok(());
        }
    }
    g.put_into_library(&[item.object], LibraryPlace::ShuffledIn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::DAMAGE, opponents_dealt_combat_damage),
        draw_win_and_shuffle,
        "When one or more of your opponents are dealt combat damage during your turn, draw two cards. If your library has no cards in it, you win the game. Fblthp's owner shuffles him into their library. (If you draw from an empty library this way, you still win the game.)",
    )
    .once_per_batch()],
    ..oracle::FBLTHP_IMPOSSIBLY_LOST
};
