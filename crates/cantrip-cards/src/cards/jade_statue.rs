use cantrip_api::*;
use cantrip_oracle as oracle;

/// "{2}: This artifact becomes a 3/6 Golem artifact creature until end of combat. Activate only during combat." (CR 205.1b, 608.2, 611.2a).
fn artifact_becomes_3_6_golem(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let golem = g.words(item.object).subtype(Subtype::Golem);
    let animation = cantrip_mechanics::Animation {
        power: 3,
        toughness: 6,
        colors: None,
        types: TypeSet::of(&[CardType::Artifact]),
        subtypes: &[golem],
        all_creature_types: false,
        replaces_creature_types: true,
        keywords: Keywords::NONE,
        ability: None,
    };
    cantrip_mechanics::animate(g, item.object, item.object, &animation, Duration::EndOfCombat);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                cost("{2}"),
                artifact_becomes_3_6_golem,
                "{2}: This artifact becomes a 3/6 Golem artifact creature until end of combat. Activate only during combat.",
            )
            .condition(cantrip_mechanics::combat::activate_only_during_combat),
    ],
    ..oracle::JADE_STATUE
};
