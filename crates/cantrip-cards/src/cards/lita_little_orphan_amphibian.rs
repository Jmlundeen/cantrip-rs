use cantrip_api::*;
use cantrip_mechanics::history;
use cantrip_mechanics::effects::{plus_one_counter_on_this, scry};
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::ANOTHER_CREATURE_YOU_CONTROL_ENTERS;
use cantrip_oracle as oracle;

static MODAL: ModalDef = modes(&[
    mode(|g, item| plus_one_counter_on_this(g, item), "Put a +1/+1 counter on Lita."),
    mode(|g, item| tokens::create_food(g, item.controller), "Create a Food token."),
    mode(|g, item| scry(g, item, 1), "Scry 1."),
])
    .allowed(history::mode_not_chosen_this_turn);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(
        ANOTHER_CREATURE_YOU_CONTROL_ENTERS,
        &MODAL,
        "Alliance \u{2014} Whenever another creature you control enters, choose one that hasn't been chosen this turn \u{2014} Put a +1/+1 counter on Lita; or create a Food token; or scry 1.",
    )],
    ..oracle::LITA_LITTLE_ORPHAN_AMPHIBIAN
};

pub static EXTRA: &[&CardDef] = &[&tokens::FOOD];
