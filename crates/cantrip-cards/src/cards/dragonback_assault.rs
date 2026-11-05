use cantrip_api::*;
use cantrip_mechanics::{effects, tokens, triggers};
use cantrip_oracle as oracle;

/// A 4/4 red Dragon creature token with flying (CR 111.4, 205.3m).
pub static DRAGON_4_4: CardDef = CardDef {
    colors: Some(ColorSet::of(Color::Red)),
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(4),
    toughness: Some(4),
    keywords: Keywords::of(&[Keyword::Flying]),
    text: "Flying",
    ..token!(Dragon)
};

/// "It deals 3 damage to each creature and each planeswalker" (CR 120.3): all together, the enchantment itself the source.
fn damage_each_creature_and_planeswalker(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::damage_each_creature_and_planeswalker(g, item, 3);
    Ok(())
}

/// "Create a 4/4 red Dragon creature token with flying" (CR 111.1, 701.7).
fn make_dragon(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &DRAGON_4_4, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    // Landfall is an ability word (CR 207.2c): the trigger is an ordinary one, whenever a land enters under your control.
    abilities: &[
        triggered(triggers::THIS_ENTERS, damage_each_creature_and_planeswalker, "When this enchantment enters, it deals 3 damage to each creature and each planeswalker."),
        triggered(triggers::LANDFALL, make_dragon, "Landfall \u{2014} Whenever a land you control enters, create a 4/4 red Dragon creature token with flying."),
    ],
    ..oracle::DRAGONBACK_ASSAULT
};

pub static EXTRA: &[&CardDef] = &[&DRAGON_4_4];
