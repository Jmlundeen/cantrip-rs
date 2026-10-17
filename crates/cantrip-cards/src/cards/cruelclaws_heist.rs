use cantrip_api::*;
use cantrip_mechanics::{gift, hand::choose_from_revealed_hand_of_kind, targets};
use cantrip_oracle as oracle;

/// "Target opponent reveals their hand. You choose a nonland card from it. Exile that card. If the gift was promised, you
/// may cast that card for as long as it remains exiled, and mana of any type can be spent to cast it." The gift was given
/// by the rules before this runs (CR 702.174j), so the hand revealed includes the card drawn. Looking at the hand is part
/// of the spell, so the choice is asked even with nothing eligible. The permission is to cast, for the card's own costs and
/// timing, for as long as that exiled card stays there (CR 400.7); it is given only to the card exiled (the new object).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = item.targets[0].player() else { return Ok(()) };
    let chosen = choose_from_revealed_hand_of_kind(g, item, opponent, Purpose::Exile, "nonland", |q, c| !q.chars(c).is(CardType::Land))?;
    let exiled = g.exile_all(&chosen);
    if item.gift.is_some() {
        for card in exiled {
            g.allow_play(card, item.controller, PlayGrant { any_type: true, ..PlayGrant::CAST });
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::OPPONENT], effect), gift: Some(gift::CARD), ..oracle::CRUELCLAWS_HEIST };
