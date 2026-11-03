use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "{T}: Target creature loses flying until end of turn." (CR 608.2, 611.2a).
fn target_creature_loses_flying(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let objects_0 = targets::object(g, item, 0);
    let kind_0_0_0 = EffectKind::LoseKeywords(Keywords::of(&[Keyword::Flying]));
    instruction(
        g,
        |g| {
            if let Some(object) = objects_0 {
                g.add_effect(item.object, object, kind_0_0_0, Duration::EndOfTurn);
            }
        },
    );
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                TAP,
                target_creature_loses_flying,
                "{T}: Target creature loses flying until end of turn.",
            )
            .targets(&[targets::CREATURE]),
    ],
    ..oracle::RADJAN_SPIRIT
};
