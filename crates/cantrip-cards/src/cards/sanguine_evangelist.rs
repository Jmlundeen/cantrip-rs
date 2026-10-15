use cantrip_api::*;
use cantrip_mechanics::battle_cry;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::{this_dies, this_enters};
use cantrip_oracle as oracle;

/// A 1/1 black Bat creature token with flying.
pub static BAT: CardDef = CardDef {
    name: "Bat",
    has_cost: false,
    colors: Some(ColorSet::of(Color::Black)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Bat]),
    power: Some(1),
    toughness: Some(1),
    keywords: Keywords::of(&[Keyword::Flying]),
    ..CardDef::BLANK
};

fn make_bat(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &BAT, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        battle_cry::TRIGGER,
        AbilityDef::Triggered(TriggeredDef {
            text: "When Sanguine Evangelist enters, create a 1/1 black Bat creature token with flying.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: make_bat,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "When Sanguine Evangelist dies, create a 1/1 black Bat creature token with flying.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_dies,
            effect: make_bat,
            look_back: true,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::SANGUINE_EVANGELIST
};

pub static EXTRA: &[&CardDef] = &[&BAT];
