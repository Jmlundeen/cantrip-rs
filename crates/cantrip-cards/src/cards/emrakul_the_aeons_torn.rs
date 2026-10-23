use cantrip_api::*;
use cantrip_mechanics::library;
use cantrip_mechanics::statics::itself;
use cantrip_mechanics::triggers::{annihilator, CAST_THIS_SPELL, PUT_INTO_A_GRAVEYARD_FROM_ANYWHERE};
use cantrip_oracle as oracle;

/// "When you cast this spell, take an extra turn after this one." Triggers from the stack (CR 603.2, 702.85a's shape).
fn extra_turn(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_extra_turns(item.controller, 1);
    Ok(())
}

/// "Its owner shuffles their graveyard into their library."
fn shuffle_graveyard_in(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let owner = g.obj(item.object).owner;
    library::shuffle_graveyard_into_library(g, owner);
    Ok(())
}

/// "Protection from spells that are one or more colors."
fn protection(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.protection_colored_spells = true;
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(CAST_THIS_SPELL, extra_turn, "When you cast this spell, take an extra turn after this one."),
        annihilator::<6>("Annihilator 6 (Whenever this creature attacks, defending player sacrifices six permanents.)"),
        triggered(PUT_INTO_A_GRAVEYARD_FROM_ANYWHERE, shuffle_graveyard_in, "When Emrakul is put into a graveyard from anywhere, its owner shuffles their graveyard into their library."),
    ],
    statics: &[static_ability(Layer::Ability, itself, protection, "Protection from spells that are one or more colors")],
    ..oracle::EMRAKUL_THE_AEONS_TORN
};
