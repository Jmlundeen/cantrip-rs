use cantrip_api::*;
use cantrip_mechanics::history;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "At the beginning of combat on your turn" while this card is in its owner's graveyard (CR 113.6k): the only zone the
/// ability functions in. It does not trigger from anywhere else.
fn beginning_of_your_combat_in_graveyard(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    if q.obj(me.source).zone.is(ZoneKind::Graveyard) { triggers::beginning_of_your_combat(q, me, event) } else { None }
}

/// "If two or more creatures died this turn" (CR 603.4): an intervening if, asked as it triggers and again as it resolves.
fn two_creatures_died_this_turn(q: &dyn Query, _: &Asker) -> bool {
    history::creatures_died_this_turn(q, None) >= 2
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::STEP_BEGAN, beginning_of_your_combat_in_graveyard),
        cantrip_mechanics::effects::return_this_card_to_battlefield,
        "At the beginning of combat on your turn, if two or more creatures died this turn, return this card from your graveyard to the battlefield.",
    )
    .condition(two_creatures_died_this_turn)
    .anywhere()],
    ..oracle::DARKLIGHT_PHOENIX
};
