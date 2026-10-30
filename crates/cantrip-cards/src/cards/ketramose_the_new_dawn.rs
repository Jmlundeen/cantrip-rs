use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "There are seven or more cards in exile" (CR 400.1): every card in every player's exile zone, whoever owns it and however
/// it got there, face down ones included (the card's ruling).
fn seven_cards_in_exile(q: &dyn Query) -> bool {
    let s = q.state();
    s.seats().map(|p| s.zone(Zone::exile(p)).len()).sum::<usize>() >= 7
}

/// "Ketramose can't attack ... unless there are seven or more cards in exile" (CR 508.1c): asked of this creature as attackers
/// are declared. Once declared it keeps attacking whatever happens to exile (the card's ruling).
fn attacks_only_with_seven_in_exile(q: &dyn Query, asker: &Asker, attacker: ObjectId, _: Target) -> bool {
    asker.source != attacker || seven_cards_in_exile(q)
}

/// "Ketramose can't ... block unless there are seven or more cards in exile" (CR 509.1b).
fn blocks_only_with_seven_in_exile(q: &dyn Query, source: ObjectId, blocker: ObjectId, _: ObjectId) -> bool {
    source != blocker || seven_cards_in_exile(q)
}

/// "Whenever one or more cards are put into exile from graveyards and/or the battlefield during your turn" (CR 603.2c, 603.10):
/// a card (a token is not a card, CR 108.2, 111.1) moved to exile from any player's graveyard or the
/// battlefield while it is the ability's controller's turn. Hand, library and stack are not mentioned.
fn cards_exiled_on_your_turn(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, from, to, .. }
            if to.is(ZoneKind::Exile) && !q.obj(*new).token
                && (from.is(ZoneKind::Graveyard) || from.is(ZoneKind::Battlefield))
                && q.state().turn.active == q.controller(source) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when(EventMask::ZONE_CHANGE, cards_exiled_on_your_turn),
        effects::draw_a_card_and_lose_1_life,
        "Whenever one or more cards are put into exile from graveyards and/or the battlefield during your turn, you draw a card and lose 1 life.",
    )
    .once_per_batch()],
    attack_rules: &[attacks_only_with_seven_in_exile],
    block_rules: &[blocks_only_with_seven_in_exile],
    ..oracle::KETRAMOSE_THE_NEW_DAWN
};
