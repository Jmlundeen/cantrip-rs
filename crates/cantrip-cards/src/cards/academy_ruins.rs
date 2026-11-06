use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "{1}{U}, {T}: Put target artifact card from your graveyard on top of your library." (CR 608.2).
fn put_target_artifact_card_graveyard(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    g.put_into_library(&[target], LibraryPlace::Top);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(
                cost("{1}{U}").and(TAP),
                put_target_artifact_card_graveyard,
                "{1}{U}, {T}: Put target artifact card from your graveyard on top of your library.",
            )
            .targets(&[targets::ARTIFACT_CARD_IN_YOUR_GRAVEYARD]),
    ],
    ..oracle::ACADEMY_RUINS
};
