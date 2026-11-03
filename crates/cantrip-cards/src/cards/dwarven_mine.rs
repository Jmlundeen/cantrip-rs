use cantrip_api::*;
use cantrip_mechanics::{lands, tokens, triggers};
use cantrip_oracle as oracle;

pub static DWARF: CardDef = CardDef {
    types: TypeSet::of(&[CardType::Creature]),
    colors: Some(ColorSet::of(Color::Red)),
    power: Some(1),
    toughness: Some(1),
    ..token!(Dwarf)
};

/// "When this land enters untapped, create a 1/1 red Dwarf creature token." (CR 603.2, 111.1).
fn create_1_1_red_dwarf(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &DWARF, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS_UNTAPPED,
            create_1_1_red_dwarf,
            "When this land enters untapped, create a 1/1 red Dwarf creature token.",
        ),
    ],
    enters: Some(lands::enters_tapped_unless_you_control_three_or_more_other::<{ lands::sub(Subtype::Mountain) }>),
    ..oracle::DWARVEN_MINE
};

pub static EXTRA: &[&CardDef] = &[&DWARF];
