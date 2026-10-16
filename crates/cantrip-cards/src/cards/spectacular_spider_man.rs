use cantrip_api::*;
use cantrip_oracle as oracle;

/// {1}: this gains flying until end of turn.
fn gain_flying(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if is_live(g, item.object) && g.is_on_battlefield(item.object) {
        g.add_effect(item.object, item.object, EffectKind::AddKeywords(Keywords::of(&[Keyword::Flying])), Duration::EndOfTurn);
    }
    Ok(())
}

/// {1}, Sacrifice this: creatures you control gain hexproof and indestructible until end of turn. The set is the
/// creatures there are as this resolves, this one already sacrificed (CR 611.2c).
fn protect_team(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for creature in g.permanents_of(item.controller) {
        if g.chars(creature).is(CardType::Creature) {
            g.add_effect(
                item.object,
                creature,
                EffectKind::AddKeywords(Keywords::of(&[Keyword::Hexproof, Keyword::Indestructible])),
                Duration::EndOfTurn,
            );
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{1}"), gain_flying, "{1}: Spectacular Spider-Man gains flying until end of turn."),
        ability(cost("{1}").and(SACRIFICE_THIS), protect_team, "{1}, Sacrifice Spectacular Spider-Man: Creatures you control gain hexproof and indestructible until end of turn."),
    ],
    ..oracle::SPECTACULAR_SPIDER_MAN
};
