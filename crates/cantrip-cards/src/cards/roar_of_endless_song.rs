use cantrip_api::*;
use cantrip_mechanics::{double, tokens};
use cantrip_oracle as oracle;

/// "Create a 5/5 green Elephant creature token" (CR 111.1).
fn create_an_elephant(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::ELEPHANT_5_5, 1)?;
    Ok(())
}

/// "Double the power and toughness of each creature you control until end of turn": the creatures are those the controller
/// has as the chapter resolves, and each one's power and toughness is read once and fixed (CR 701.10a-b, 608.2h).
fn double_each_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<ObjectId> = g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    // "Double the power and toughness of each creature you control": one instruction (CR 608.2c).
    instruction(g, |g| {
        for creature in creatures {
            double::power_and_toughness(g, item.object, creature);
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        chapter(1, create_an_elephant, "I, II \u{2014} Create a 5/5 green Elephant creature token."),
        chapter(2, create_an_elephant, "I, II \u{2014} Create a 5/5 green Elephant creature token."),
        chapter(3, double_each_creature, "III \u{2014} Double the power and toughness of each creature you control until end of turn."),
    ],
    ..oracle::ROAR_OF_ENDLESS_SONG
};

pub static EXTRA: &[&CardDef] = &[&tokens::ELEPHANT_5_5];
