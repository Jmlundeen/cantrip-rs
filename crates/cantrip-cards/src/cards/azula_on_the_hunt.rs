use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_mechanics::{firebending, tokens};
use cantrip_oracle as oracle;

/// "You lose 1 life and create a Clue token." (CR 119.3, 111.10f.) Creating a Clue is not investigating (the ruling), so nothing that
/// triggers on investigating sees it.
fn lose_life_and_create_a_clue(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.lose_life(item.controller, 1);
    tokens::create(g, item.controller, &tokens::CLUE, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        firebending::trigger::<2>("Firebending 2 (Whenever this creature attacks, add {R}{R}. This mana lasts until end of combat.)"),
        triggered(
            THIS_ATTACKS,
            lose_life_and_create_a_clue,
            "Whenever Azula attacks, you lose 1 life and create a Clue token. (It's an artifact with \"{2}, Sacrifice this token: Draw a card.\")",
        ),
    ],
    ..oracle::AZULA_ON_THE_HUNT
};

pub static EXTRA: &[&CardDef] = &[&tokens::CLUE];
