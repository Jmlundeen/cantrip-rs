use cantrip_api::*;
use cantrip_mechanics::{hand, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// When this creature enters, look at target opponent's hand. You may exile a nonland card from it. For as long as that
/// card remains exiled, its owner may play it. A spell cast this way costs {2} more to cast (CR 601.2f).
fn look_and_exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = item.target(0).player() else { return Ok(()) };
    if let Some(card) = hand::look_and_may_choose(g, item, opponent, Purpose::Exile, "nonland", |q, c| !q.chars(c).is(CardType::Land))? {
        hand::exile_owner_may_play_for_two_more(g, card);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, look_and_exile, "When this creature enters, look at target opponent's hand. You may exile a nonland card from it. For as long as that card remains exiled, its owner may play it. A spell cast this way costs {2} more to cast.")
        .targets(&[targets::OPPONENT])],
    ..oracle::ELITE_SPELLBINDER
};
