use cantrip_api::*;
use cantrip_mechanics::{library, targets};
use cantrip_oracle as oracle;

/// "Target artifact card in the targeted player's graveyard" (CR 115.1): an artifact card in the graveyard of the player chosen
/// as the first target, once that is chosen.
fn artifact_card_in_targeted_graveyard(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(
        t, Target::Object(id) if q.state().obj(id).zone.kind() == ZoneKind::Graveyard
        && !q.state().obj(id).token && !q.state().obj(id).card_copy && q.chars(id).is(CardType::Artifact)
        && q.chosen_targets().first().and_then(|t| t.player()).is_none_or(|p| p == q.state().obj(id).owner)
    )
}

/// "Put any number of target artifact cards from target player's graveyard on top of their library in any order." (CR 608.2b,
/// 401.4): every card that is still a legal target goes, its owner (the targeted player) putting them in the order they choose.
/// A player who is no longer a legal target supplies no information (CR 608.2b).
fn put_any_number_target_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = targets::player(g, item, 0) else { return Ok(()) };
    let cards = targets::legal_objects(g, item);
    let order = library::ask_library_order(g, player, item.object, &cards, DestinationKind::LibraryTop)?;
    g.put_into_library(&order, LibraryPlace::Top);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(
        &[
            targets::PLAYER,
            target(artifact_card_in_targeted_graveyard).between(0, TargetSpec::ANY_NUMBER),
        ],
        put_any_number_target_artifact,
    ),
    ..oracle::DRAFNAS_RESTORATION
};
