use cantrip_api::*;
use cantrip_mechanics::{choose, triggers::this_enters, mana};
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
    g.exile_with(&cards, item.object, false);
    Ok(())
}

/// The colors of the card exiled with this artifact: what its mana ability may add.
fn imprinted_colors(q: &dyn Query, asker: &Asker) -> ColorSet {
    let mut colors = ColorSet::NONE;
    for card in q.exiled_with(asker.source) {
        colors = ColorSet(colors.0 | q.chars(card).colors.0);
    }
    colors
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "{T}: Add one mana of any of the exiled card's colors.",
            cost: Cost { tap: true, ..Cost::NONE },
            mana: Some(mana::add_chosen),
            any_color: true,
            colors_from: Some(imprinted_colors),
            adds_any: 1,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Imprint \u{2014} When this artifact enters, you may exile a nonartifact, nonland card from your hand.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: imprint,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::CHROME_MOX
};
