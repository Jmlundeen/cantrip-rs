use cantrip_api::*;
use cantrip_mechanics::impending;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// The 2/1 white Insect creature token with flying it makes.
pub static INSECT: CardDef = CardDef {
    name: "Insect",
    has_cost: false,
    colors: Some(ColorSet::of(Color::White)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Insect]),
    power: Some(2),
    toughness: Some(1),
    keywords: Keywords::of(&[Keyword::Flying]),
    ..CardDef::BLANK
};

pub static EXTRA: &[&CardDef] = &[&INSECT];

/// Create two 2/1 white Insect creature tokens with flying.
fn two_insects(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &INSECT, 2)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    ways: &[impending::way(mana(2, 2, 0, 0, 0, 0), "Impending 4\u{2014}{2}{W}{W}")],
    enters: Some(impending::enters::<4>),
    statics: &[impending::NOT_A_CREATURE],
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever this permanent enters, create two 2/1 white Insect creature tokens with flying.",
            listens: EventMask::ZONE_CHANGE,
            matches: triggers::this_enters,
            effect: two_insects,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever this permanent attacks, create two 2/1 white Insect creature tokens with flying.",
            listens: EventMask::ATTACKS,
            matches: triggers::this_attacks,
            effect: two_insects,
            ..TriggeredDef::BLANK
        }),
        impending::REMOVE_A_TIME_COUNTER,
    ],
    ..oracle::OVERLORD_OF_THE_MISTMOORS
};
