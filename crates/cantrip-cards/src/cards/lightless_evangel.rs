use cantrip_api::*;
use cantrip_mechanics::effects::plus_one_counter_on_this;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Whenever you sacrifice another creature or artifact" (CR 701.21a): a creature or an artifact of yours that a sacrifice moved
/// from the battlefield, read as it was when it left (CR 603.10a), so a Treasure or a Lander counts. One trigger for each
/// permanent, however many are sacrificed together. Not this creature: it is "another".
fn you_sacrifice_another_creature_or_artifact(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let sacrificed = triggers::sacrificed_by(q, me.controller, event)?;
    let chars = q.chars(sacrificed);
    if sacrificed != me.source && (chars.is(CardType::Creature) || chars.is(CardType::Artifact)) { Fired::PLAIN } else { None }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, you_sacrifice_another_creature_or_artifact),
        plus_one_counter_on_this,
        "Whenever you sacrifice another creature or artifact, put a +1/+1 counter on this creature.",
    )
    .on_sacrifice()],
    ..oracle::LIGHTLESS_EVANGEL
};
