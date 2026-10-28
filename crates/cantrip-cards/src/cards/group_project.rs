use cantrip_api::*;
use cantrip_mechanics::{flashback, tokens, untapped_creature_you_control};
use cantrip_oracle as oracle;

/// "Create a 2/2 red and white Spirit creature token."
fn spirit(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::SPIRIT_2_2_RED_WHITE, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], spirit),
    ways: &[flashback::way(
        tap_untapped(3, untapped_creature_you_control),
        "Flashback\u{2014}Tap three untapped creatures you control. (You may cast this card from your graveyard for its flashback cost. Then exile it.)",
    )],
    ..oracle::GROUP_PROJECT
};

pub static EXTRA: &[&CardDef] = &[&tokens::SPIRIT_2_2_RED_WHITE];
