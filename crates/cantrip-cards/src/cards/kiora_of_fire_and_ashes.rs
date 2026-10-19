use cantrip_api::*;
use cantrip_mechanics::{tokens, triggers};
use cantrip_oracle as oracle;

fn make_dragon(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::DRAGON_5_5, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            make_dragon,
            "When Kiora enters, create a 5/5 red Dragon creature token with flying.",
        ),
        ability(cost("{8}"), make_dragon, "{8}: Create a 5/5 red Dragon creature token with flying."),
    ],
    ..oracle::KIORA_OF_FIRE_AND_ASHES
};

pub static EXTRA: &[&CardDef] = &[&tokens::DRAGON_5_5];
