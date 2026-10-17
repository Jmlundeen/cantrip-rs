use cantrip_api::*;
use cantrip_mechanics::{effects, targets, tokens};
use cantrip_oracle as oracle;

/// "Draw a card. Create a Food token." (CR 121.1, 111.10b.)
fn draw_and_food(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::draw(g, item, 1)?;
    tokens::create_food(g, item.controller)
}

static MODAL: ModalDef = modes(&[
    mode(effects::destroy_first_target, "Destroy target creature with flying.").targets(&[targets::CREATURE_WITH_FLYING]),
    mode(effects::destroy_first_target, "Destroy target enchantment.").targets(&[targets::ENCHANTMENT]),
    mode(draw_and_food, "Draw a card. Create a Food token."),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::PAWPATCH_FORMATION };

pub static EXTRA: &[&CardDef] = &[&tokens::FOOD];
