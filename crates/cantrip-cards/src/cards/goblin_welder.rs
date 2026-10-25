use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target artifact card in that player's graveyard" (CR 601.2c): the graveyard of whoever controls the artifact the first slot
/// chose. Nothing is legal until the artifact has been chosen, and the same fact is read again as the ability resolves.
fn artifact_card_in_that_players_graveyard(q: &dyn Query, _: &Asker, t: Target) -> bool {
    let Target::Object(card) = t else { return false };
    if !targets::card_in_graveyard(q, t, &[CardType::Artifact]) {
        return false;
    }
    let c = q.obj(card);
    // Before the first slot has chosen (is the ability offered at all?) any artifact card in a graveyard will do.
    match q.chosen_targets().first() {
        None => true,
        Some(&Target::Object(artifact)) => q.is_on_battlefield(artifact) && c.zone == Zone::graveyard(q.controller(artifact)),
        Some(_) => false,
    }
}

/// Choose target artifact a player controls and target artifact card in that player's graveyard. If both targets are still legal
/// as this ability resolves, that player simultaneously sacrifices the artifact and returns the artifact card to the battlefield
/// (CR 101.4): under their control.
fn weld(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !g.target_legal(item, 0) || !g.target_legal(item, 1) {
        return Ok(());
    }
    let (Target::Object(artifact), Target::Object(card)) = (item.target(0), item.target(1)) else { return Ok(()) };
    let player = g.controller(artifact);
    g.sacrifice_and_return(player, &[artifact], &[(card, player)])
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP, weld, "{T}: Choose target artifact a player controls and target artifact card in that player's graveyard. If both targets are still legal as this ability resolves, that player simultaneously sacrifices the artifact and returns the artifact card to the battlefield.")
        .targets(&[
            targets::ARTIFACT,
            target(artifact_card_in_that_players_graveyard),
        ])],
    ..oracle::GOBLIN_WELDER
};
