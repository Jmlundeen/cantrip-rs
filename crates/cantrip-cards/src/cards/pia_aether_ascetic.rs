use cantrip_api::*;
use cantrip_mechanics::{effects, library};
use cantrip_oracle as oracle;

fn discard_and_tutor(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if effects::may_discard_a_card(g, item)? {
        library::search_to_hand(g, item.controller, item.object, library::Find::ONE.revealed(),
            |q, card| q.chars(card).is(CardType::Enchantment))?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(cantrip_mechanics::triggers::THIS_ENTERS, discard_and_tutor,
        "When Pia enters, you may discard a card. If you do, search your library for an enchantment card, reveal it, put it into your hand, then shuffle.")],
    ..oracle::PIA_AETHER_ASCETIC
};
