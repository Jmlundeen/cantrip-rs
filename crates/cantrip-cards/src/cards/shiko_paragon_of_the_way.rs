use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{copy, targets};
use cantrip_oracle as oracle;

/// "Target nonland card with mana value 3 or less from your graveyard" (CR 115.1, 202.3): a card in the ability controller's own
/// graveyard that is not a land card, its mana value read as the game reads it in a graveyard (X is 0, CR 202.3e).
fn nonland_card_with_mana_value_3_or_less_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id)
        if targets::card_in_your_graveyard_with_mana_value_at_most(q, asker, t, &[], 3) && !q.chars(id).is(CardType::Land))
}

/// "Exile target nonland card with mana value 3 or less from your graveyard. Copy it, then you may cast the copy without paying
/// its mana cost." The target is exiled (a new object, CR 400.7) and "it" is the card in exile, copied there (CR 707.12): the
/// copy is cast while the ability resolves, whatever its timing (CR 608.2g), if it has a legal target and the controller says
/// so; a permanent spell's copy becomes a token as it resolves (CR 707.10f). With the target gone, nothing is exiled or copied
/// (CR 608.2b).
fn exile_and_copy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    let exiled = g.exile(card);
    copy::may_cast_a_copy_without_paying(g, item.controller, exiled)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, exile_and_copy, "When Shiko enters, exile target nonland card with mana value 3 or less from your graveyard. Copy it, then you may cast the copy without paying its mana cost.")
        .targets(&[target(nonland_card_with_mana_value_3_or_less_in_your_graveyard)])],
    ..oracle::SHIKO_PARAGON_OF_THE_WAY
};
