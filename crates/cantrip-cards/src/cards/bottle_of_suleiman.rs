use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

pub static DJINN: CardDef = CardDef {
    types: TypeSet::of(&[CardType::Artifact, CardType::Creature]),
    power: Some(5),
    toughness: Some(5),
    keywords: Keywords::of(&[Keyword::Flying]),
    ..token!(Djinn)
};

/// The flip happens on resolution (CR 705.1), after the Bottle was sacrificed as a cost.
fn flip_coin_if_win_flip(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let won = g.flip_coin(item.controller);
    g.end_sentence();
    if won {
        tokens::create(g, item.controller, &DJINN, 1)?;
    } else {
        g.damage(item.object, Target::Player(item.controller), 5);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            cost("{1}").and(SACRIFICE_THIS),
            flip_coin_if_win_flip,
            "{1}, Sacrifice this artifact: Flip a coin. If you win the flip, create a 5/5 colorless Djinn artifact creature token with flying. If you lose the flip, this artifact deals 5 damage to you.",
        ),
    ],
    ..oracle::BOTTLE_OF_SULEIMAN
};

pub static EXTRA: &[&CardDef] = &[&DJINN];
