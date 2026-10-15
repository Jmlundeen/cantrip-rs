use cantrip_api::*;
use cantrip_mechanics::{boast, damage_target};
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Sacrifice another creature or artifact" (CR 118.3).
fn another_creature_or_artifact(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    let c = q.chars(id);
    id != asker.source && (c.is(CardType::Creature) || c.is(CardType::Artifact))
}

/// The permanent sacrificed to pay for this activation: the sacrifice the history recorded just before the ability was
/// activated (costs are paid, then the ability is put on the stack as activated, CR 602.2).
fn sacrificed_to_pay(q: &dyn Query, item: &StackItem) -> Option<ObjectId> {
    let records = q.history().this_turn();
    let activated = records.iter().rposition(|r| {
        matches!(r.event, Event::AbilityActivated { source, ability, .. } if source == item.object && ability == item.ability)
    })?;
    records[..activated].iter().rev().find_map(|r| match r.event {
        Event::ZoneChange { old, from, reason: ZoneReason::Sacrifice, .. }
            if from.is(ZoneKind::Battlefield) && old != item.object =>
        {
            Some(old)
        }
        _ => None,
    })
}

/// "Broadside Bombardiers deals damage equal to 2 plus the sacrificed permanent's mana value to any target": the
/// mana value it had as it was sacrificed (CR 608.2h).
fn boast_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let value = sacrificed_to_pay(g, item).map_or(0, |id| g.mana_value(id));
    damage_target(g, item, 0, 2 + value as u16);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[boast::ability(
        "Boast \u{2014} Sacrifice another creature or artifact: Broadside Bombardiers deals damage equal to 2 plus the sacrificed permanent's mana value to any target. (Activate only if this creature attacked this turn and only once each turn.)",
        Cost::sacrifice(1, another_creature_or_artifact),
        &[targets::ANY],
        boast_damage,
    )],
    ..oracle::BROADSIDE_BOMBARDIERS
};
