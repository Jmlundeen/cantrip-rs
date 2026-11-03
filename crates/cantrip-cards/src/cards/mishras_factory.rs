use cantrip_api::*;
use cantrip_mechanics::{animate, pump, targets, Animation};
use cantrip_oracle as oracle;

/// "{1}: This land becomes a 2/2 Assembly-Worker artifact creature until end of turn. It's still a land." (CR 608.2, 611.2a).
fn land_becomes_2_2_assembly(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let subtype = g.words(item.object).subtype(Subtype::AssemblyWorker);
    let animation = Animation {
        power: 2,
        toughness: 2,
        colors: None,
        types: TypeSet::of(&[CardType::Artifact]),
        subtypes: &[subtype],
        all_creature_types: false,
        replaces_creature_types: false,
        keywords: Keywords::NONE,
        ability: None,
    };
    animate(g, item.object, item.object, &animation, Duration::EndOfTurn);
    Ok(())
}

/// "{T}: Target Assembly-Worker creature gets +1/+1 until end of turn." (CR 115.1).
fn target_assembly_worker_creature_gets_target_0(
    q: &dyn Query,
    asker: &Asker,
    t: Target,
) -> bool {
    matches!(
        t, Target::Object(id) if cantrip_mechanics::targets::permanent(q, id) && ((q
        .chars(id).has_subtype(q.words(asker.source).subtype(Subtype::AssemblyWorker)) &&
        q.chars(id).is(CardType::Creature)))
    )
}

/// "{T}: Target Assembly-Worker creature gets +1/+1 until end of turn." (CR 608.2, 611.2a).
fn target_assembly_worker_creature_gets(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    pump(g, item.object, target, 1, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(
            cost("{1}"),
            land_becomes_2_2_assembly,
            "{1}: This land becomes a 2/2 Assembly-Worker artifact creature until end of turn. It's still a land.",
        ),
        ability(
                TAP,
                target_assembly_worker_creature_gets,
                "{T}: Target Assembly-Worker creature gets +1/+1 until end of turn.",
            )
            .targets(&[target(target_assembly_worker_creature_gets_target_0)]),
    ],
    ..oracle::MISHRAS_FACTORY
};
