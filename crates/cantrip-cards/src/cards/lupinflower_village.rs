use cantrip_api::*;
use cantrip_mechanics::library::{look_at_top, Rest};
use cantrip_oracle as oracle;

/// "Look at the top six cards of your library. You may reveal a Bat, Bird, Mouse, or Rabbit card from among them and put it
/// into your hand. Put the rest on the bottom of your library in a random order." The creature types are read through this
/// land's own words (CR 612). A card of any type with one of them qualifies; with none to take the cards are still looked at.
fn look(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let words = g.words(item.object);
    let types = [Subtype::Bat, Subtype::Bird, Subtype::Mouse, Subtype::Rabbit].map(|s| words.subtype(s));
    look_at_top(g, item.controller, item.object, 6, (0, 1), true, |c| types.iter().any(|&t| c.has_subtype(t)), Rest::BottomRandom)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(TAP, add(Mana::W).only_for(ManaRestriction::CREATURE_SPELLS), "{T}: Add {W}. Spend this mana only to cast a creature spell."),
        ability(
            cost("{1}{W}, {T}").and(SACRIFICE_THIS),
            look,
            "{1}{W}, {T}, Sacrifice this land: Look at the top six cards of your library. You may reveal a Bat, Bird, Mouse, or Rabbit card from among them and put it into your hand. Put the rest on the bottom of your library in a random order.",
        ),
    ],
    ..oracle::LUPINFLOWER_VILLAGE
};
