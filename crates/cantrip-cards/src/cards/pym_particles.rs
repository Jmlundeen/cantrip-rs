use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target creature gains vigilance until end of turn and can't be blocked this turn. Draw a card." If the single
/// target is illegal on resolution the whole spell is countered (CR 608.2b) and nothing is drawn.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_effect(
        item.object,
        creature,
        EffectKind::AddKeywords(Keywords::of(&[Keyword::Vigilance, Keyword::CantBeBlocked])),
        Duration::EndOfTurn,
    );
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    ..oracle::PYM_PARTICLES
};
