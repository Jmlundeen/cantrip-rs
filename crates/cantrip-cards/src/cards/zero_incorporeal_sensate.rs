use cantrip_api::*;
use cantrip_mechanics::{hand, select, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When Zero enters, look at target opponent's hand. You may exile a nonland card from it. For as long as that card
/// remains exiled, its owner may play it. A spell cast this way costs {2} more to cast." (CR 603.6a, 406.2, 601.2f): the
/// card is chosen as the trigger resolves and may be declined (CR 608.2d), and only a nonland card may be exiled. The
/// permission and the {2} surcharge outlast Zero himself, so both live on the exiled card (CR 610.3, 118.9a).
fn look_and_exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = item.target(0).player() else { return Ok(()) };
    if let Some(card) = hand::look_and_may_choose(g, item, opponent, Purpose::Exile, select::NONLAND_CARD)? {
        hand::exile_owner_may_play_for_two_more(g, card);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, look_and_exile, "When Zero enters, look at target opponent's hand. You may exile a nonland card from it. For as long as that card remains exiled, its owner may play it. A spell cast this way costs {2} more to cast.")
        .targets(&[targets::OPPONENT])],
    ..oracle::ZERO_INCORPOREAL_SENSATE
};
