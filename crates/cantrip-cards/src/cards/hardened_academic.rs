use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "This creature gains lifelink until end of turn." Nothing if it has left the battlefield by the time the ability
/// resolves (CR 400.7): the effect is added to the object it was when the ability was put on the stack.
fn lifelink(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::gain_lifelink_until_end_of_turn(g, item.object, item.object);
    Ok(())
}

/// "Whenever one or more cards leave your graveyard" (CR 603.2c, 603.10a): a card of the source's controller's graveyard
/// going to any other zone. A token is in no graveyard long enough to leave one (CR 111.7, 704.5d), and the core records
/// no departure for it. The cards that left together are the subjects of the matching events, so the batch triggers it once.
fn cards_leave_your_graveyard(_: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, .. } if *from == Zone::graveyard(me.controller) => {
            Fired::about(Target::Object(*old))
        }
        _ => None,
    }
}

/// "Put a +1/+1 counter on target creature you control." No target left (CR 608.2b) does nothing.
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::effects::plus_one_counter_on_target(g, item)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(discard_cards(1), lifelink, "Discard a card: This creature gains lifelink until end of turn."),
        triggered(
            when_asked(EventMask::ZONE_CHANGE, cards_leave_your_graveyard),
            counter,
            "Whenever one or more cards leave your graveyard, put a +1/+1 counter on target creature you control.",
        )
        .targets(&[targets::CREATURE_YOU_CONTROL])
        .once_per_batch(),
    ],
    ..oracle::HARDENED_ACADEMIC
};
