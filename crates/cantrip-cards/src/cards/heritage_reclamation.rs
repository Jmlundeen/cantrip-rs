use cantrip_api::*;
use cantrip_mechanics::{destroy_target, exile_target, targets};
use cantrip_oracle as oracle;

/// "Destroy target artifact." / "Destroy target enchantment." (CR 701.8): nothing if the target is no longer legal
/// (CR 608.2b).
fn destroy_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

/// "Exile up to one target card from a graveyard. Draw a card." The card is drawn whether or not a target was chosen; if
/// the one chosen became illegal the spell does not resolve at all (CR 608.2b), the core's.
fn exile_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    g.draw(item.controller, 1);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(destroy_it, "Destroy target artifact.").targets(&[targets::ARTIFACT]),
    mode(destroy_it, "Destroy target enchantment.").targets(&[targets::ENCHANTMENT]),
    mode(exile_and_draw, "Exile up to one target card from a graveyard. Draw a card.")
        .targets(&[targets::CARD_IN_A_GRAVEYARD.optional()]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::HERITAGE_RECLAMATION };
