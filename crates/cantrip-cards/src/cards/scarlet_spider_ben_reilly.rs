use cantrip_api::*;
use cantrip_mechanics::web_slinging;
use cantrip_oracle as oracle;

/// The creature returned to its owner's hand to pay for casting `spell` by web-slinging (CR 702.188a, 601.2h): the one the
/// history shows leaving the battlefield for a hand between the spell's move to the stack (CR 601.2a) and its becoming
/// cast (CR 601.2i), which is when its costs are paid. As it last existed on the battlefield, that object.
fn returned_creature(q: &dyn Query, spell: ObjectId) -> Option<ObjectId> {
    let records = q.history().all();
    let onto_stack = records
        .iter()
        .rposition(|r| matches!(r.event, Event::ZoneChange { new, to, .. } if new == spell && to.is(ZoneKind::Stack)))?;
    records[onto_stack..]
        .iter()
        .take_while(|r| !matches!(r.event, Event::Cast(cast) if cast.spell == spell))
        .find_map(|r| match r.event {
            Event::ZoneChange { old, from, to, .. } if from.is(ZoneKind::Battlefield) && to.is(ZoneKind::Hand) => Some(old),
            _ => None,
        })
}

/// Sensational Save: "If Scarlet Spider was cast using web-slinging, he enters with X +1/+1 counters on him, where X is
/// the mana value of the returned creature." (CR 614.1c, 702.188a) The mana value is the creature's as it last existed on
/// the battlefield, an {X} in its cost counting 0 (ruling 2025-09-19, CR 202.3b).
fn sensational_save(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    if web_slinging::was_cast_using_web_slinging(g, entering.object)
        && let Some(creature) = returned_creature(g, entering.object)
    {
        entering.counters[CounterKind::PlusOne as usize] += g.mana_value(creature).min(i16::MAX as u32) as i16;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    ways: &[web_slinging::way(mana("{R}{G}"), "Web-slinging {R}{G}")],
    enters: Some(sensational_save),
    ..oracle::SCARLET_SPIDER_BEN_REILLY
};
