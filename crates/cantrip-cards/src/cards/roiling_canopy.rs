use cantrip_api::*;
use cantrip_mechanics::{mana, pump, targets};
use cantrip_oracle as oracle;

/// How many Forests `controller` controls other than `entered`, as the layers and this land's words have it (CR 305.6,
/// 612.2): a land of any kind may be a Forest.
fn other_forests(q: &dyn Query, source: ObjectId, controller: PlayerId, entered: ObjectId) -> usize {
    let forest = q.words(source).subtype(Subtype::Forest);
    q.state().battlefield().iter().filter(|&&id| id != entered && q.controller(id) == controller && q.chars(id).has_subtype(forest)).count()
}

/// "Whenever a Forest you control enters, if you control at least five other Forests" (CR 603.2, 603.4, 603.6a): the
/// Forest that entered under its controller's control, and the intervening "if" as it enters, "other" than that Forest.
/// The land itself is not a Forest unless something makes it one, in which case it is counted like any other.
fn forest_enters_with_five_others(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield)
                && q.controller(*new) == me.controller
                && q.chars(*new).has_subtype(q.words(me.source).subtype(Subtype::Forest))
                && other_forests(q, me.source, me.controller, *new) >= 5 =>
        {
            Fired::about(Target::Object(*new))
        }
        _ => None,
    }
}

/// "Target creature you control gets +3/+3 until end of turn." The intervening "if" is checked again as the ability
/// resolves (CR 603.4): five Forests other than the one that entered, which may have left. A target no longer legal
/// means the ability does nothing (CR 608.2b).
fn pump_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(entered) = item.subject else { return Ok(()) };
    if other_forests(g, item.object, item.controller, entered) < 5 {
        return Ok(());
    }
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    pump(g, item.object, creature, 3, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        mana_ability(TAP, add(Mana::G), "{T}: Add {G}."),
        triggered(
            when_asked(EventMask::ZONE_CHANGE, forest_enters_with_five_others),
            pump_creature,
            "Whenever a Forest you control enters, if you control at least five other Forests, target creature you control gets +3/+3 until end of turn.",
        )
        .targets(&[targets::CREATURE_YOU_CONTROL]),
    ],
    ..oracle::ROILING_CANOPY
};
