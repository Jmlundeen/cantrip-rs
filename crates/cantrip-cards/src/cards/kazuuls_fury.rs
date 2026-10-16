use cantrip_api::*;
use cantrip_mechanics::{damage_target, mana, targets};
use cantrip_oracle as oracle;

/// The power of what the spell's additional cost sacrificed (CR 601.2h), as it last existed (CR 608.2h): the sacrifice is
/// the last one in the history before the spell was cast.
fn sacrificed_power(q: &dyn Query, spell: ObjectId) -> i32 {
    let records = q.history().all();
    let Some(cast) = records.iter().rposition(|r| matches!(r.event, Event::Cast(c) if c.spell == spell)) else { return 0 };
    records[..cast]
        .iter()
        .rev()
        .find_map(|r| match r.event {
            Event::ZoneChange { old, from, reason: ZoneReason::Sacrifice, .. } if from.is(ZoneKind::Battlefield) => Some(old),
            _ => None,
        })
        .map_or(0, |creature| q.chars(creature).power.unwrap_or(0) as i32)
}

/// Kazuul's Fury deals damage equal to the sacrificed creature's power to any target; a power of 0 or less deals none.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let power = sacrificed_power(g, item.object);
    if power > 0 {
        damage_target(g, item, 0, power.min(u16::MAX as i32) as u16);
    }
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add(Mana::R), "{T}: Add {R}.")],
    enters: Some(mana::enters_tapped),
    ..oracle::KAZUULS_CLIFFS
};

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::ANY], effect }),
    // As an additional cost to cast this spell, sacrifice a creature (CR 118.8).
    additional_cost: Some(Cost::sacrifice(1, targets::filters::creature)),
    back: Some(&BACK),
    ..oracle::KAZUULS_FURY
};
