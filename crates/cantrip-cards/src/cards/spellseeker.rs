use cantrip_api::*;
use cantrip_mechanics::library::{Find, search_to_hand};
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

/// "You may search your library for an instant or sorcery card with mana value 2 or less, reveal it, put it into your
/// hand, then shuffle." Declining the search ("you may") is choosing no card.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_hand(g, item.controller, item.object, Find::ONE.revealed(), |q, c| {
        let chars = q.chars(c);
        (chars.is(CardType::Instant) || chars.is(CardType::Sorcery)) && q.mana_value(c) <= 2
    })?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When Spellseeker enters, you may search your library for an instant or sorcery card with mana value 2 or less, reveal it, put it into your hand, then shuffle.",
        listens: EventMask::ZONE_CHANGE,
        matches: this_enters,
        effect: search,
        ..TriggeredDef::BLANK
    })],
    ..oracle::SPELLSEEKER
};
