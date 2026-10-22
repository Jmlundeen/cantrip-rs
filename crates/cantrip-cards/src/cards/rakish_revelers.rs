use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{land_until_cast, tokens};
use cantrip_oracle as oracle;

/// The 1/1 green and white Citizen creature token (CR 111.4).
pub static CITIZEN: CardDef = CardDef {
    name: "Citizen Token",
    token_name: Some(&[Subtype::Citizen]),
    has_cost: false,
    // White (bit 0) and green (bit 4).
    colors: Some(ColorSet(0b10001)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Citizen]),
    power: Some(1),
    toughness: Some(1),
    ..CardDef::BLANK
};

/// "When this creature enters, create a 1/1 green and white Citizen creature token."
fn citizen(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &CITIZEN, 1)?;
    Ok(())
}

pub static EXTRA: &[&CardDef] = &[&CITIZEN];

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, citizen, "When this creature enters, create a 1/1 green and white Citizen creature token."),
        land_until_cast::ability::<0>(
            cost("{2}"),
            "{2}, Exile this card from your hand: Target land gains \"{T}: Add {R}, {G}, or {W}\" until this card is cast from exile. You may cast this card for as long as it remains exiled.",
        ),
    ],
    delayed: &[mana_ability(TAP, add_one_of(&[Color::Red, Color::Green, Color::White]), "{T}: Add {R}, {G}, or {W}.")],
    ..oracle::RAKISH_REVELERS
};
