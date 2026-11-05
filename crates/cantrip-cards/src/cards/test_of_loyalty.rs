use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

fn exile_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.exile(card);
        g.draw(item.controller, 1);
    }
    Ok(())
}

/// The first mode targets any graveyard card and draws; the second returns a creature card in your graveyard.
static MODES: ModalDef = modes(&[
    mode(exile_and_draw, "Plain, Simple Garak — Exile target card from a graveyard. Draw a card.")
        .targets(&[targets::CARD_IN_A_GRAVEYARD]),
    mode(effects::return_first_target_to_hand, "Of the Obsidian Order — Return target creature card from your graveyard to your hand.")
        .targets(&[targets::CREATURE_CARD_IN_YOUR_GRAVEYARD]),
]).choose(1, 1);

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&MODES),
    ..oracle::TEST_OF_LOYALTY
};
