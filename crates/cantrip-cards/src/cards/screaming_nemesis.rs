use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Whenever this creature is dealt damage" (CR 120.2, 603.2c): one trigger for the damage it is dealt at once, even if
/// it is lethal, carrying that much (`sums_per_subject` adds up the events of one combat that hurt it from several sources).
fn dealt_damage(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { target: Target::Object(hurt), amount, .. } if *hurt == source && *amount > 0 => {
            Some(Fired { subject: Target::None, data: [*amount as u32, 0] })
        }
        _ => None,
    }
}

/// "Any other target" (CR 115.1): any target but this creature itself.
fn any_other_target(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    t != Target::Object(asker.source) && (targets::ANY.filter)(q, asker, t)
}

/// It deals that much damage to any other target. If a player is dealt damage this way, they can't gain life for the rest of
/// the game (CR 119.7): only a player the damage actually reached, not one it was prevented for.
fn retaliate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let dealt = g.damage(item.object, item.targets[0], item.data[0] as u16);
    if dealt > 0
        && let Target::Player(p) = item.targets[0]
    {
        g.add_effect(item.object, item.object, EffectKind::CantGainLife(p), Duration::Indefinite);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when(EventMask::DAMAGE, dealt_damage), retaliate, "Whenever this creature is dealt damage, it deals that much damage to any other target. If a player is dealt damage this way, they can't gain life for the rest of the game.")
        .targets(&[TargetSpec { min: 1, max: 1, filter: any_other_target }])
        .sums_per_subject()],
    ..oracle::SCREAMING_NEMESIS
};
