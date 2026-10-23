use cantrip_api::*;
use cantrip_mechanics::{effects, targets, tokens};
use cantrip_oracle as oracle;

/// "Return up to one target permanent card from your graveyard to your hand. Create a Clue token." (CR 400.3, 111.10f.)
/// With no target chosen the Clue is still created; if the one chosen became illegal the spell does not resolve at all
/// (CR 608.2b), the core's. Creating a Clue is not investigating (the ruling).
fn return_and_clue(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::return_first_target_to_hand(g, item)?;
    g.end_sentence();
    tokens::create(g, item.controller, &tokens::CLUE, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PERMANENT_CARD_IN_YOUR_GRAVEYARD.optional()], return_and_clue),
    ..oracle::TRUE_ANCESTRY
};

pub static EXTRA: &[&CardDef] = &[&tokens::CLUE];
