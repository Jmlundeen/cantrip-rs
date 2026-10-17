use cantrip_api::*;
use cantrip_mechanics::effects::plus_one_counter_on_this;
use cantrip_mechanics::library::exile_top_play_this_turn;
use cantrip_oracle as oracle;

/// Exile the top card of your library. You may play that card this turn (CR 305.1, 601.3): a land too.
fn exile_top(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_top_play_this_turn(g, item.controller, 1);
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

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(cantrip_mechanics::triggers::THIS_ATTACKS, exile_top, "Whenever Laelia, the Blade Reforged attacks, exile the top card of your library. You may play that card this turn."),
        triggered(when(EventMask::ZONE_CHANGE, cards_exiled), plus_one_counter_on_this, "Whenever one or more cards are put into exile from your library and/or your graveyard, put a +1/+1 counter on Laelia, the Blade Reforged.")
            .once_per_batch(),
    ],
    ..oracle::LAELIA_THE_BLADE_REFORGED
};
