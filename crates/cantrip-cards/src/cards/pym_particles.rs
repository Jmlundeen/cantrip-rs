use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target creature gains vigilance until end of turn and can't be blocked this turn. Draw a card." If the single
/// target is illegal on resolution the whole spell is countered (CR 608.2b) and nothing is drawn.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    // One instruction (`Game::one_instruction`). "And can't be blocked this turn" has no "gains": an effect on the creature,
    // not an ability, so it outlasts the creature losing its abilities (CR 113.1a, 611.2c).
    instruction(g, |g| {
        g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::Vigilance])), Duration::EndOfTurn);
        g.add_effect(item.object, creature, EffectKind::CantBeBlocked, Duration::EndOfTurn);
    });
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    ..oracle::PYM_PARTICLES
};
