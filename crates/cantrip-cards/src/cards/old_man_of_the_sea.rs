use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// A creature's power for "power less than or equal to": one that is no longer a creature has none, and counts as 0
/// (the card's ruling).
fn power(q: &dyn Query, id: ObjectId) -> i16 {
    let c = q.chars(id);
    if c.is(CardType::Creature) { c.power.unwrap_or(0) } else { 0 }
}

/// "Target creature with power less than or equal to this creature's power" (CR 115.1), judged as the ability is activated
/// and again as it resolves (CR 608.2b), against this creature's power then, or as it last was if it has left (CR 113.7a).
fn no_more_power(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && q.chars(id).is(CardType::Creature) && power(q, id) <= targets::power_or_last_known(q, asker.source))
}

/// "For as long as this creature remains tapped and that creature's power remains less than or equal to this creature's
/// power" (CR 611.2b): this creature is the effect's source, the same object still on the battlefield and tapped (one that
/// has left is not tapped: the ruling), whoever controls it; and the creature it took has power no greater than its own,
/// whatever else it has become (the rulings).
fn holds(q: &dyn Query, e: &ContinuousEffect) -> bool {
    let me = e.source;
    is_live(q, me) && q.is_on_battlefield(me) && q.obj(me).tapped && power(q, e.affected) <= power(q, me)
}

/// "Gain control of target creature ... for as long as ..." (CR 613.1b): nothing if the target is no longer legal (CR 608.2b),
/// and nothing at all if this creature has untapped or the target has grown before it resolves (CR 611.2b, the ruling).
fn take(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.gain_control(item.object, creature, item.controller, Duration::AsLongAs(g.card_id(&CARD), 0));
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    // "You may choose not to untap this creature during your untap step." (CR 502.3)
    may_not_untap: true,
    abilities: &[ability(cost("{T}"), take, "{T}: Gain control of target creature with power less than or equal to this creature's power for as long as this creature remains tapped and that creature's power remains less than or equal to this creature's power.")
        .targets(&[target(no_more_power)])],
    lasting: &[Lasting { text: "{source} remains tapped and its power remains less than or equal to {source}'s power", holds }],
    ..oracle::OLD_MAN_OF_THE_SEA
};
