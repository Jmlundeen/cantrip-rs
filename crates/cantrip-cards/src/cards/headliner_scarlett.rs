use cantrip_api::*;
use cantrip_mechanics::{targets, triggers::this_enters};
use cantrip_oracle as oracle;

/// Creatures target player controls can't block this turn (CR 509.1b): a rule about the player until the turn ends, so it
/// reaches creatures that arrive under their control later (CR 611.2c) and grants none of them an ability (CR 113.12).
fn cant_block(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.target_legal(item, 0)
        && let Target::Player(p) = item.targets[0]
    {
        g.add_effect(item.object, item.object, EffectKind::CreaturesCantBlock(p), Duration::EndOfTurn);
    }
    Ok(())
}

/// Exile the top card of your library face down. You may look at and play that card this turn.
fn exile_top_and_play(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let Some(&top) = g.state().zone(Zone::library(player)).first() else { return Ok(()) };
    let exiled = g.exile_face_down(&[top], None)[0];
    g.look_at(player, &[exiled]);
    if g.obj(exiled).zone.is(ZoneKind::Exile) {
        g.allow_play(exiled, player, PlayGrant { lands: true, expiry: Expiry::EndOfTurn, ..PlayGrant::CAST });
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When Headliner Scarlett enters, creatures target player controls can't block this turn.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            targets: &[targets::PLAYER],
            effect: cant_block,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "At the beginning of your upkeep, exile the top card of your library face down. You may look at and play that card this turn.",
            listens: EventMask::STEP_BEGAN,
            matches: cantrip_mechanics::triggers::your_upkeep,
            effect: exile_top_and_play,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::HEADLINER_SCARLETT
};
