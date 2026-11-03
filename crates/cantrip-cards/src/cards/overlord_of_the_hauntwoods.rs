use cantrip_api::*;
use cantrip_mechanics::{impending, tokens, triggers};
use cantrip_oracle as oracle;

pub static EVERYWHERE: CardDef = CardDef {
    name: "Everywhere",
    types: TypeSet::of(&[CardType::Land]),
    subtypes: SubtypeSet::of(
        &[
            Subtype::Plains,
            Subtype::Island,
            Subtype::Swamp,
            Subtype::Mountain,
            Subtype::Forest,
        ],
    ),
    ..CardDef::BLANK
};

/// "Whenever this permanent enters or attacks, create a tapped colorless land token named Everywhere that is every basic land type." (CR 608.2, 111.1).
fn create_tapped_colorless_land_token(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    tokens::create_by_rule(g, item.controller, &EVERYWHERE, 1, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        impending::REMOVE_A_TIME_COUNTER,
        triggered(
            triggers::THIS_ENTERS_OR_ATTACKS,
            create_tapped_colorless_land_token,
            "Whenever this permanent enters or attacks, create a tapped colorless land token named Everywhere that is every basic land type.",
        ),
    ],
    statics: &[impending::NOT_A_CREATURE],
    enters: Some(impending::enters::<4>),
    ways: &[impending::way(mana("{1}{G}{G}"), "Impending 4—{1}{G}{G}")],
    ..oracle::OVERLORD_OF_THE_HAUNTWOODS
};

pub static EXTRA: &[&CardDef] = &[&EVERYWHERE];
