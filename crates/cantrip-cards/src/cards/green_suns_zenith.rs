use cantrip_api::*;
use cantrip_mechanics::library::{search_to_battlefield, Find};
use cantrip_oracle as oracle;

/// Search your library for a green creature card with mana value X or less, put it onto the battlefield, then shuffle.
/// Shuffle Green Sun's Zenith into its owner's library. The last instruction moves the spell itself off the stack
/// (CR 608.2m), so it goes nowhere more as it finishes resolving.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let x = item.x as u32;
    let green = g.words(item.object).color(Color::Green);
    search_to_battlefield(g, player, item.object, Find::ONE, false, |q, c| {
        let chars = q.chars(c);
        chars.is(CardType::Creature) && chars.colors.has(green) && q.mana_value(c) <= x
    })?;
    g.put_into_library(&[item.object], LibraryPlace::ShuffledIn);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: Some(SpellDef { targets: &[], effect }), ..oracle::GREEN_SUNS_ZENITH };
