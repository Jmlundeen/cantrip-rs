use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Tap three untapped artifacts and/or creatures you control" (CR 602.2b, 118.3): any untapped artifact or creature its
/// controller controls, this creature and one that came under their control this turn included (CR 302.6 limits only a
/// creature's own {T} abilities, and this is not {T} but the tapping of permanents as a cost).
fn untapped_artifact_or_creature_you_control(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    let chars = q.chars(id);
    q.controller(id) == asker.controller && !q.obj(id).tapped && (chars.is(CardType::Artifact) || chars.is(CardType::Creature))
}

/// "As long as this creature has three or more counters on it" (CR 122.1): counters of any kind, added up.
fn has_three_or_more_counters(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    source == affected && q.state().obj(source).counter_array().iter().map(|&n| n.max(0) as i32).sum::<i32>() >= 3
}

fn flying_and_vigilance(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Flying).with(Keyword::Vigilance);
}

/// "Put a +1/+1 counter on this creature. Scry 1." The counter goes only on this creature while it is still on the
/// battlefield (CR 400.7); the scry happens either way.
fn grow_and_scry(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    cantrip_mechanics::scry(g, item.controller, item.object, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        tap_untapped(3, untapped_artifact_or_creature_you_control),
        grow_and_scry,
        "Tap three untapped artifacts and/or creatures you control: Put a +1/+1 counter on this creature. Scry 1. Activate only as a sorcery.",
    )
    .sorcery_speed()],
    statics: &[static_ability(
        Layer::Ability,
        has_three_or_more_counters,
        flying_and_vigilance,
        "As long as this creature has three or more counters on it, it has flying and vigilance.",
    )],
    ..oracle::WARDEN_OF_THE_INNER_SKY
};
