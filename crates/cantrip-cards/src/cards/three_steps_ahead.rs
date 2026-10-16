use cantrip_api::*;
use cantrip_mechanics::spree;
use cantrip_mechanics::{counter_target, discard_chosen, targets};
use cantrip_oracle as oracle;

/// Counter target spell.
fn counter_spell(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target(g, item, 0);
    Ok(())
}

/// Draw two cards, then discard a card.
fn loot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 2);
    discard_chosen(g, item.controller, item.object, 1)?;
    Ok(())
}

const MODES: &[ModeDef] = &[
    mode(counter_spell, "+ {1}{U} — Counter target spell.")
        .targets(&[targets::SPELL])
        .cost(mana("{1}{U}")),
    mode(cantrip_mechanics::copy::create_token_copy_of_target, "+ {3} — Create a token that's a copy of target artifact or creature you control.")
        .targets(&[targets::ARTIFACT_OR_CREATURE_YOU_CONTROL])
        .cost(mana("{3}")),
    mode(loot, "+ {2} — Draw two cards, then discard a card.")
        .cost(mana("{2}")),
];

static MODAL: ModalDef = spree::modal(MODES);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::THREE_STEPS_AHEAD };
