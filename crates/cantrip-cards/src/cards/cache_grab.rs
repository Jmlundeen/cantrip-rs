use cantrip_api::*;
use cantrip_mechanics::{choose, targets, tokens};
use cantrip_oracle as oracle;

/// "Mill four cards. You may put a permanent card from among the cards milled this way into your hand. If you control a Squirrel
/// or returned a Squirrel card to your hand this way, create a Food token." The cards milled (CR 701.17) are all shown, with only
/// the permanent cards (CR 110.4a) to choose from; the one chosen is put into its owner's hand from where milling left it, which a
/// replacement effect may have made exile (CR 701.17c). "Squirrel" is the creature type of this card's text, read through its words
/// (CR 612): a permanent you control with it now, the creature types being whatever the layers give it (a changeling is one), or
/// the card put into your hand, which has the types it has there. Nothing is asked when no permanent card was milled.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let milled = g.mill(you, 4);
    let permanents: Vec<ObjectId> = milled.iter().copied().filter(|&card| targets::is_permanent_card(&g.chars(card))).collect();
    let chosen = choose::cards_from_kind(g, you, item.object, Purpose::PutIntoHand, "permanent", milled, permanents, 1, true)?;
    let returned = g.return_to_hand(&chosen);
    let squirrel = g.words(item.object).subtype(Subtype::Squirrel);
    let controls_a_squirrel = g.permanents_of(you).into_iter().any(|id| g.chars(id).has_subtype(squirrel));
    if controls_a_squirrel || returned.iter().any(|&card| g.chars(card).has_subtype(squirrel)) {
        tokens::create_food(g, you)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::CACHE_GRAB };

pub static EXTRA: &[&CardDef] = &[&tokens::FOOD];
