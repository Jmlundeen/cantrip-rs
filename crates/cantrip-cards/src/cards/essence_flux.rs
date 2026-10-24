use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Exile target creature you control, then return that card to the battlefield under its owner's control. If it's a
/// Spirit, put a +1/+1 counter on it." The creature returns as a new object (CR 400.7); a token exiled this way ceases to
/// exist and does not return (CR 111.7). Whether it is a Spirit is read after it has returned, as it is now (ruling
/// 2016-04-08), through the words of Essence Flux's own text (CR 612).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let returned = cantrip_mechanics::exile_and_return_objects(g, &[creature])?;
    g.end_sentence();
    for card in returned {
        if g.is_on_battlefield(card) && g.chars(card).has_subtype(g.words(item.object).subtype(Subtype::Spirit)) {
            g.add_counters(card, CounterKind::PlusOne, 1);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE_YOU_CONTROL], effect), ..oracle::ESSENCE_FLUX };
