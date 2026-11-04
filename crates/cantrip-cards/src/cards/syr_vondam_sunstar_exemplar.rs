use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Whenever another creature you control dies or is put into exile" (CR 700.4, 603.10a): a creature other than this one went from
/// the battlefield to a graveyard or to exile, looked back on as it last was there, under the controller it last had. A creature card
/// put into exile from any other zone does not count (the ruling), nor does one of the other side's.
fn another_creature_you_control_dies_or_is_exiled(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. }
            if *old != me.source
                && from.is(ZoneKind::Battlefield)
                && (to.is(ZoneKind::Graveyard) || to.is(ZoneKind::Exile))
                && q.chars(*old).is(CardType::Creature)
                && q.obj(*old).controller == me.controller =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Put a +1/+1 counter on [this creature] and you gain 1 life" (CR 122.1, 119.3): the counter only if the source is still the
/// object that triggered (CR 400.7), the life whatever became of it.
fn counter_and_gain_a_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::plus_one_counter_on_this(g, item)?;
    effects::gain_life(g, item, 1)
}

/// "When this creature dies or is put into exile while its power is 4 or greater" (CR 700.4, 603.10a): this creature went from
/// the battlefield to a graveyard or to exile, and its power, as it last was on the battlefield, was 4 or more. That is the trigger's
/// own condition, read as it happens: the ability resolves whatever its power became.
fn this_dies_or_is_exiled_with_power_4_or_greater(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. }
            if *old == source
                && from.is(ZoneKind::Battlefield)
                && (to.is(ZoneKind::Graveyard) || to.is(ZoneKind::Exile))
                && q.chars(*old).power.is_some_and(|p| p >= 4) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::ZONE_CHANGE, another_creature_you_control_dies_or_is_exiled),
            counter_and_gain_a_life,
            "Whenever another creature you control dies or is put into exile, put a +1/+1 counter on Syr Vondam and you gain 1 life.",
        ),
        triggered(
            when(EventMask::ZONE_CHANGE, this_dies_or_is_exiled_with_power_4_or_greater),
            effects::destroy_first_target,
            "When Syr Vondam dies or is put into exile while its power is 4 or greater, destroy up to one target nonland permanent.",
        )
        .targets(&[up_to_one(targets::NONLAND_PERMANENT.filter)]),
    ],
    ..oracle::SYR_VONDAM_SUNSTAR_EXEMPLAR
};
