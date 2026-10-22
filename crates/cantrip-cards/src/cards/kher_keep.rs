use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// The 0/1 red Kobold creature token named Kobolds of Kher Keep it makes (CR 111.4: a token's name is the one the
/// effect gives it, so it is not named "Kobold").
pub static KOBOLDS_OF_KHER_KEEP: CardDef = CardDef {
    name: "Kobolds of Kher Keep",
    has_cost: false,
    colors: Some(ColorSet::of(Color::Red)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Kobold]),
    power: Some(0),
    toughness: Some(1),
    ..CardDef::BLANK
};

fn make_kobold(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &KOBOLDS_OF_KHER_KEEP, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(cost("{1}{R}, {T}"), make_kobold, "{1}{R}, {T}: Create a 0/1 red Kobold creature token named Kobolds of Kher Keep."),
    ],
    ..oracle::KHER_KEEP
};

pub static EXTRA: &[&CardDef] = &[&KOBOLDS_OF_KHER_KEEP];
