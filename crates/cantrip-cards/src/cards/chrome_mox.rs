use cantrip_api::*;
use cantrip_mechanics::{choose, mana, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// Imprint (CR 607.2a): "you may exile a nonartifact, nonland card from your hand", exiled with this artifact. Nothing
/// is asked when nothing in the hand qualifies; their own hand is not revealed.
fn imprint(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let hand = g.state().zone(Zone::hand(item.controller)).to_vec();
    let eligible: Vec<ObjectId> = hand
        .into_iter()
        .filter(|&c| {
            let chars = g.chars(c);
            !chars.is(CardType::Artifact) && !chars.is(CardType::Land)
        })
        .collect();
    let cards = choose::cards(g, item.controller, item.object, Purpose::Exile, eligible, 1, true)?;
    g.exile_with(&cards, g.linked(item), false);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any of the exiled card's colors.")
            .colors_from(mana::colors_of_cards_exiled_with_this),
        triggered(THIS_ENTERS, imprint, "Imprint \u{2014} When this artifact enters, you may exile a nonartifact, nonland card from your hand."),
    ],
    ..oracle::CHROME_MOX
};
