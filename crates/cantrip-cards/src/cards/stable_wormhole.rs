use cantrip_api::*;
use cantrip_mechanics::triggers::YOU_ATTACK;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// "Whenever you attack, create a tapped Planet land token named New Planet with '{T}: Add one mana of any color.'"
/// (CR 508.1m, 111.1, 111.4): one trigger for each declaration of attackers, whatever their number.
fn create_new_planet(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_by_rule(g, item.controller, &tokens::NEW_PLANET, 1, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOU_ATTACK,
        create_new_planet,
        "Whenever you attack, create a tapped Planet land token named New Planet with \"{T}: Add one mana of any color.\"",
    )],
    ..oracle::STABLE_WORMHOLE
};

pub static EXTRA: &[&CardDef] = &[&tokens::NEW_PLANET];
