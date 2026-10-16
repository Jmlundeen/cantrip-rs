use cantrip_api::*;
use cantrip_mechanics::{triggers::THIS_ENTERS, triggers, effects};
use cantrip_oracle as oracle;

/// Exile the top card of your library. If you've completed a dungeon, you may play that card this turn without paying
/// its mana cost. Otherwise, you may play that card this turn.
fn exile_top_and_play(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let Some(&top) = g.state().zone(Zone::library(player)).first() else { return Ok(()) };
    let exiled = g.exile(top);
    if g.obj(exiled).zone.is(ZoneKind::Exile) {
        let free = g.state().player(player).dungeons_completed > 0;
        g.allow_play(
            exiled,
            player,
            PlayGrant { instead: if free { Some(ManaCost::ZERO) } else { None }, lands: true, expiry: Expiry::EndOfTurn, ..PlayGrant::CAST },
        );
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, effects::take_the_initiative, "When this creature enters, you take the initiative."),
        triggered(triggers::THIS_ATTACKS, exile_top_and_play, "Whenever this creature attacks, exile the top card of your library. If you've completed a dungeon, you may play that card this turn without paying its mana cost. Otherwise, you may play that card this turn."),
    ],
    ..oracle::CAVES_OF_CHAOS_ADVENTURER
};
