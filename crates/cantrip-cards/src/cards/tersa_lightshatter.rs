use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

/// When this enters, discard up to two cards, then draw that many cards. The discards happen together (CR 701.9),
/// before the draw.
fn discard_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let hand = g.state().zone(Zone::hand(player)).to_vec();
    let chosen = choose::cards(g, player, item.object, Purpose::Discard, hand, 2, true)?;
    if chosen.is_empty() {
        return Ok(());
    }
    g.discard_together(&chosen);
    g.draw(player, chosen.len() as u32);
    Ok(())
}

/// "If there are seven or more cards in your graveyard" (CR 603.4): asked as it triggers and again as it resolves.
fn seven_in_graveyard(q: &dyn Query, asker: &Asker) -> bool {
    q.state().zone(Zone::graveyard(asker.controller)).len() >= 7
}

/// Exile a card at random from your graveyard. You may play that card this turn.
fn exile_and_play(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let cards: Vec<ObjectId> = g.state().zone(Zone::graveyard(player)).to_vec();
    let Some(card) = cantrip_mechanics::library::random_of(g, &cards) else { return Ok(()) };
    let exiled = g.exile(card);
    if is_live(g, exiled) && g.obj(exiled).zone.is(ZoneKind::Exile) {
        g.allow_play(exiled, player, PlayGrant { lands: true, expiry: Expiry::EndOfTurn, ..PlayGrant::CAST });
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When Tersa Lightshatter enters, discard up to two cards, then draw that many cards.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: discard_and_draw,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever Tersa Lightshatter attacks, if there are seven or more cards in your graveyard, exile a card at random from your graveyard. You may play that card this turn.",
            listens: EventMask::ATTACKS,
            matches: cantrip_mechanics::triggers::this_attacks,
            effect: exile_and_play,
            condition: Some(seven_in_graveyard),
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::TERSA_LIGHTSHATTER
};
