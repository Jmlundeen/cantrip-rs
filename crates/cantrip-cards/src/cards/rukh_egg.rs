use cantrip_api::*;
use cantrip_mechanics::{delayed, tokens, triggers};
use cantrip_oracle as oracle;

pub static BIRD: CardDef = CardDef {
    types: TypeSet::of(&[CardType::Creature]),
    colors: Some(ColorSet::of(Color::Red)),
    power: Some(4),
    toughness: Some(4),
    keywords: Keywords::of(&[Keyword::Flying]),
    ..token!(Bird)
};

/// "When this creature dies, create a 4/4 red Bird creature token with flying at the beginning of the next end step." (CR 608.2).
fn delayed_1(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &BIRD, 1)?;
    Ok(())
}

/// "When this creature dies, create a 4/4 red Bird creature token with flying at the beginning of the next end step." (CR 608.2).
fn create_4_4_red_bird(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    delayed::at_next_end_step(g, item, 0, Target::None);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_DIES,
            create_4_4_red_bird,
            "When this creature dies, create a 4/4 red Bird creature token with flying at the beginning of the next end step.",
        ),
    ],
    delayed: &[
        delayed_trigger(
            delayed_1,
            "When this creature dies, create a 4/4 red Bird creature token with flying at the beginning of the next end step.",
        ),
    ],
    ..oracle::RUKH_EGG
};

pub static EXTRA: &[&CardDef] = &[&BIRD];
