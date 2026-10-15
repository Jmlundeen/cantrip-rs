use cantrip_api::*;
use cantrip_mechanics::{triggers::this_enters, triggers, effects};
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
        AbilityDef::Triggered(TriggeredDef {
            text: "When this creature enters, you take the initiative.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: effects::take_the_initiative,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever this creature attacks, exile the top card of your library. If you've completed a dungeon, you may play that card this turn without paying its mana cost. Otherwise, you may play that card this turn.",
            listens: EventMask::ATTACKS,
            matches: triggers::this_attacks,
            effect: exile_top_and_play,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::CAVES_OF_CHAOS_ADVENTURER
};
