use cantrip_api::*;
use cantrip_mechanics::{effects, pump, targets};
use cantrip_oracle as oracle;

/// "Target creature gets +2/+2 and gains flying until end of turn. If it's a Spider, you gain 2 life." The pump is one
/// sentence's continuous effects (CR 611.2a); the Spider is judged as the second sentence resolves, after them, and a spell
/// whose only target has become illegal does nothing at all (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    instruction(g, |g| {
        pump(g, item.object, creature, 2, 2);
        effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Flying);
    });
    g.end_sentence();
    if g.chars(creature).has_subtype(g.words(item.object).subtype(Subtype::Spider)) {
        effects::gain_life(g, item, 2)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE], effect), ..oracle::THWIP };
