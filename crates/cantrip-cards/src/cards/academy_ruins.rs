use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "{1}{U}, {T}: Put target artifact card from your graveyard on top of your library." (CR 115.1).
fn put_target_artifact_card_graveyard_target_0(
    q: &dyn Query,
    asker: &Asker,
    t: Target,
) -> bool {
    matches!(
        t, Target::Object(id) if (q.state().obj(id).zone.kind() == ZoneKind::Graveyard)
        && ! q.state().obj(id).token && ! q.state().obj(id).card_copy && ((q.chars(id)
        .is(CardType::Artifact) && (q.state().obj(id).zone.kind() == ZoneKind::Graveyard
        && asker.controller == q.state().obj(id).owner)))
    )
}

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
            .targets(&[target(put_target_artifact_card_graveyard_target_0)]),
    ],
    ..oracle::ACADEMY_RUINS
};
