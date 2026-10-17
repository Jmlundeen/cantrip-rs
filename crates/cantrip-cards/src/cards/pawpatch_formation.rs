use cantrip_api::*;
use cantrip_mechanics::{destroy_target, effects, targets, tokens};
use cantrip_oracle as oracle;

/// "Destroy target creature with flying" / "Destroy target enchantment" (CR 701.8): the verb judges regeneration and
/// indestructible; a target that is gone or no longer legal is not destroyed (CR 608.2b).
fn destroy_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

/// "Draw a card. Create a Food token." (CR 121.1, 111.10b.)
fn draw_and_food(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::draw(g, item, 1)?;
    tokens::create_food(g, item.controller)
}

static MODAL: ModalDef = modes(&[
    mode(destroy_it, "Destroy target creature with flying.").targets(&[targets::CREATURE_WITH_FLYING]),
    mode(destroy_it, "Destroy target enchantment.").targets(&[targets::ENCHANTMENT]),
    mode(draw_and_food, "Draw a card. Create a Food token."),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::PAWPATCH_FORMATION };
