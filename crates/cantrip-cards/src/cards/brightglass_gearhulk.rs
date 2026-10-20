use cantrip_api::*;
use cantrip_mechanics::{triggers::THIS_ENTERS, library::{ask_may_search, search_to_hand, Find}};
use cantrip_oracle as oracle;

const ASK: Question = Question::new("SearchLibrary", "search your library for up to two cards with mana value 1 or less?", "Artifact, creature and/or enchantment cards, revealed and put into your hand, then shuffle", "Search", "Don't search");

/// You may search your library for up to two artifact, creature, and/or enchantment cards with mana value 1 or less,
/// reveal them, put them into your hand, then shuffle (CR 701.23). The "may" is asked first; declining skips the
/// search and the shuffle.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    if !ask_may_search(g, player, item.object, &ASK)? {
        return Ok(());
    }
    search_to_hand(g, player, item.object, Find::ONE.revealed().up_to(2), |q, c| {
        let chars = q.chars(c);
        (chars.is(CardType::Artifact) || chars.is(CardType::Creature) || chars.is(CardType::Enchantment))
            && q.card(chars.card).cost.mana_value() <= 1
    })?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, search, "When Brightglass Gearhulk enters, you may search your library for up to two artifact, creature, and/or enchantment cards with mana value 1 or less, reveal them, put them into your hand, then shuffle.")],
    ..oracle::BRIGHTGLASS_GEARHULK
};
