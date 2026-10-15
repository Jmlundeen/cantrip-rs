use cantrip_api::*;
use cantrip_mechanics::library::exile_top_may_play;
use cantrip_oracle as oracle;

/// Exile the top card of your library. You may play that card this turn (CR 305.1, 601.3): a land too.
fn exile_top(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let grant = PlayGrant { lands: true, expiry: Expiry::EndOfTurn, ..PlayGrant::CAST };
    exile_top_may_play(g, item.controller, 1, item.controller, grant);
    Ok(())
}

/// "Whenever one or more cards are put into exile from your library and/or your graveyard" (CR 603.2c): judged by the
/// zone each card left, which one of this ability's controller's own zones.
fn cards_exiled(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { from, to, .. }
            if to.is(ZoneKind::Exile)
                && (from.is(ZoneKind::Library) || from.is(ZoneKind::Graveyard))
                && from.player == q.controller(source) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever Laelia, the Blade Reforged attacks, exile the top card of your library. You may play that card this turn.",
            listens: EventMask::ATTACKS,
            matches: cantrip_mechanics::triggers::this_attacks,
            effect: exile_top,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever one or more cards are put into exile from your library and/or your graveyard, put a +1/+1 counter on Laelia, the Blade Reforged.",
            listens: EventMask::ZONE_CHANGE,
            matches: cards_exiled,
            effect: grow,
            once_per_batch: true,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::LAELIA_THE_BLADE_REFORGED
};
