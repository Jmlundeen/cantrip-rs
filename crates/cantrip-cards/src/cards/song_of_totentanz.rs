use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// The 1/1 black Rat creature token with "This token can't block." (CR 111.1, 509.1b).
pub static RAT: CardDef = CardDef {
    name: "Rat",
    has_cost: false,
    colors: Some(ColorSet::of(Color::Black)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Rat]),
    power: Some(1),
    toughness: Some(1),
    keywords: Keywords::of(&[Keyword::CantBlock]),
    text: "This token can't block.",
    ..CardDef::BLANK
};

/// "Create X 1/1 black Rat creature tokens with 'This token can't block.' Creatures you control gain haste until end of
/// turn." The tokens come first, as one instruction (CR 101.4), so they are among the creatures you control as the
/// spell resolves; the creatures that gain haste are those there then, and ones you begin to control later in the turn
/// do not (CR 611.2c). X is the value announced (0 makes no tokens, yet the haste still happens).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &RAT, item.x as u32)?;
    let creatures: Vec<ObjectId> =
        g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    for creature in creatures {
        g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::Haste])), Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::SONG_OF_TOTENTANZ };

pub static EXTRA: &[&CardDef] = &[&RAT];
