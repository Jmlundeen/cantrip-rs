use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::{crew::crew, discard_chosen, tokens::PILOT};
use cantrip_oracle as oracle;

/// Draw two cards, then discard a card. Create a Pilot.
fn loot_and_pilot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 2);
    discard_chosen(g, item.controller, item.object, 1)?;
    tokens::create(g, item.controller, &PILOT, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "{1}, {T}: Draw two cards, then discard a card. Create a 1/1 colorless Pilot creature token with \"This token crews Vehicles as though its power were 2 greater.\"",
            cost: Cost { mana: mana(1, 0, 0, 0, 0, 0), tap: true, ..Cost::NONE },
            effect: loot_and_pilot,
            ..ActivatedDef::BLANK
        }),
        crew(8, "Crew 8 (Tap any number of creatures you control with total power 8 or more: This Vehicle becomes an artifact creature until end of turn.)"),
    ],
    ..oracle::SHORIKAI_GENESIS_ENGINE
};

pub static EXTRA: &[&CardDef] = &[&PILOT];
