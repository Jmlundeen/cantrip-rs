use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Another target creature you control" (CR 115.1): a creature other than this one that you control.
fn another_creature_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if id != asker.source && q.is_on_battlefield(id)
        && q.chars(id).is(CardType::Creature) && q.controller(id) == asker.controller)
}

/// "For as long as Lwaxana Troi remains tapped" (CR 611.2b): this creature is the effect's source, the same object still on the
/// battlefield and tapped (one that has left is not tapped: the ruling), whoever controls it.
fn tapped_here(q: &dyn Query, e: &ContinuousEffect) -> bool {
    let me = e.source;
    is_live(q, me) && q.is_on_battlefield(me) && q.obj(me).tapped
}


/// "Choose another target creature you control. For as long as Lwaxana Troi remains tapped, ..." (CR 611.2b): records the
/// chosen creature for as long as this creature remains tapped; nothing if the target is no longer legal (CR 608.2b).
fn choose_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_effect(item.object, creature, EffectKind::TriggersAgainWhileSourceTapped, Duration::AsLongAs(g.card_id(&CARD), 0));
    }
    Ok(())
}

/// "If a triggered ability of that creature triggers, that ability triggers an additional time" (CR 603.2d) as a
/// `CardDef::additional_triggers` entry: a triggered ability of the chosen creature, while Lwaxana Troi is still tapped.
fn chosen_creature_triggers_again(q: &dyn Query, asker: &Asker, _: &Event, triggering: &Triggering) -> bool {
    triggering.permanent
        && q.is_on_battlefield(asker.source)
        && q.obj(asker.source).tapped
        && q.state().effects.iter().any(|e| {
            e.source == asker.source && e.affected == triggering.source && e.kind == EffectKind::TriggersAgainWhileSourceTapped
        })
}

pub static CARD: CardDef = CardDef {
    // "You may choose not to untap Lwaxana Troi during your untap step." (CR 502.3)
    may_not_untap: true,
    abilities: &[ability(cost("{2}, {T}"), choose_creature, "{2}, {T}: Choose another target creature you control. For as long as Lwaxana Troi remains tapped, if a triggered ability of that creature triggers, that ability triggers an additional time.")
        .targets(&[target(another_creature_you_control)])],
    lasting: &[Lasting { text: "{source} remains tapped", holds: tapped_here }],
    additional_triggers: &[chosen_creature_triggers_again],
    ..oracle::LWAXANA_TROI_CHARMING_GUEST
};
