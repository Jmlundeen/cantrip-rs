use cantrip_api::*;
use cantrip_mechanics::{tokens, triggers::EACH_UPKEEP};
use cantrip_oracle as oracle;

/// "A 3/3 green Forest Tentacle land creature token" (CR 111.3, 111.4, 205.4c).
/// Forest gives it its intrinsic mana ability (CR 305.6); it is not a basic land.
pub static FOREST_TENTACLE: CardDef = CardDef {
    colors: Some(ColorSet::of(Color::Green)),
    types: TypeSet::of(&[CardType::Land, CardType::Creature]),
    power: Some(3),
    toughness: Some(3),
    ..token!(Forest Tentacle)
};

/// "You create a 3/3 green Forest Tentacle land creature token" (CR 701.7a): the ability's controller,
/// irrespective of whose upkeep began, creates one token.
fn create_tentacle(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &FOREST_TENTACLE, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(EACH_UPKEEP, create_tentacle, "At the beginning of each player's upkeep, you create a 3/3 green Forest Tentacle land creature token.")],
    ..oracle::VERDANT_KRAKEN
};

pub static EXTRA: &[&CardDef] = &[&FOREST_TENTACLE];
