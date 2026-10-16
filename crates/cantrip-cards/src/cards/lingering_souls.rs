use cantrip_api::*;
use cantrip_mechanics::flashback;
use cantrip_oracle as oracle;

/// The 1/1 white Spirit creature token with flying it makes.
pub static SPIRIT: CardDef = CardDef {
    name: "Spirit",
    has_cost: false,
    colors: Some(ColorSet::of(Color::White)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Spirit]),
    power: Some(1),
    toughness: Some(1),
    keywords: Keywords::of(&[Keyword::Flying]),
    text: "Flying",
    ..CardDef::BLANK
};

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::tokens::create(g, item.controller, &SPIRIT, 2)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ways: &[flashback::way(cost("{1}{B}"), "Flashback {1}{B}")],
    ..oracle::LINGERING_SOULS
};

pub static EXTRA: &[&CardDef] = &[&SPIRIT];
