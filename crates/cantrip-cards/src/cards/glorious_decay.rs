use cantrip_api::*;
use cantrip_mechanics::{damage_target, destroy_target, exile_target, targets};
use cantrip_oracle as oracle;

/// "Destroy target artifact."
fn destroy_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

/// "Glorious Decay deals 4 damage to target creature with flying." Nothing happens if the target has lost flying or is
/// otherwise no longer legal as the spell resolves (CR 608.2b).
fn damage_flyer(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 4);
    Ok(())
}

/// "Exile target card from a graveyard. Draw a card." The draw follows the exile only while the target is still legal:
/// a spell with an illegal only target does not resolve at all, so nothing is drawn (CR 608.2b, and the ruling).
fn exile_card_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if exile_target(g, item, 0) {
        g.draw(item.controller, 1);
    }
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(destroy_artifact, "Destroy target artifact.").targets(&[targets::ARTIFACT]),
    mode(damage_flyer, "Glorious Decay deals 4 damage to target creature with flying.").targets(&[targets::CREATURE_WITH_FLYING]),
    mode(exile_card_and_draw, "Exile target card from a graveyard. Draw a card.").targets(&[targets::CARD_IN_A_GRAVEYARD]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::GLORIOUS_DECAY };
