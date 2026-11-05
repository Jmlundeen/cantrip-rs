use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::AN_OPPONENT_DRAWS_A_CARD;
use cantrip_oracle as oracle;

/// The 1/1 blue Faerie with flying it makes (CR 111.10b, 702.9).
pub static FAERIE: CardDef = CardDef {
    colors: Some(ColorSet::of(Color::Blue)),
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(1),
    toughness: Some(1),
    keywords: Keywords::of(&[Keyword::Flying]),
    ..token!(Faerie)
};

/// "during your turn" (CR 102.1, 603.4): an intervening "if", checked as the ability triggers and as it resolves, so a
/// card drawn on an opponent's turn makes nothing even if the turn has changed by then.
fn during_your_turn(q: &dyn Query, me: &Asker) -> bool {
    q.state().turn.active == me.controller
}

fn make_faerie(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &FAERIE, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        AN_OPPONENT_DRAWS_A_CARD,
        make_faerie,
        "Whenever an opponent draws a card during your turn, create a 1/1 blue Faerie creature token with flying.",
    )
    .condition(during_your_turn)],
    ..oracle::TROI_WINGING_IT
};

pub static EXTRA: &[&CardDef] = &[&FAERIE];
