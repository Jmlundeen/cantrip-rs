use cantrip_api::*;
use cantrip_mechanics::{hand::choose_from_revealed_hand, targets, triggers::this_enters};
use cantrip_oracle as oracle;

/// "Target opponent reveals their hand and you choose a nonland card from it. Exile that card." Looking at the hand is
/// part of the ability, so the choice is asked even with nothing eligible. The card is remembered as exiled with this
/// creature (CR 607.2a) and is not tied to its staying on the battlefield: the second ability brings it back.
fn reveal_and_exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = item.targets[0].player() else { return Ok(()) };
    let chosen = choose_from_revealed_hand(g, item, opponent, Purpose::Exile, |q, c| !q.chars(c).is(CardType::Land))?;
    if !chosen.is_empty() {
        g.exile_with(&chosen, item.object, false);
    }
    Ok(())
}

/// "Return the exiled card to its owner's hand."
fn return_exiled(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let exiled = g.exiled_with(item.object);
    g.return_to_hand(&exiled);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When this creature enters, target opponent reveals their hand and you choose a nonland card from it. Exile that card.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            targets: &[targets::OPPONENT],
            effect: reveal_and_exile,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "When this creature leaves the battlefield, return the exiled card to its owner's hand.",
            listens: EventMask::ZONE_CHANGE,
            matches: cantrip_mechanics::triggers::this_leaves,
            effect: return_exiled,
            look_back: true,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::TIDEHOLLOW_SCULLER
};
