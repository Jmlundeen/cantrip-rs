use cantrip_api::*;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// "Target creature gets -X/-X until end of turn. That creature gets an additional -3/-3 until end of turn if you
/// controlled a Faerie as you cast this spell." Whether you did is what the board was as the spell became cast (CR
/// 601.2i), kept in its cast record: a Faerie lost since still counts, one gained since does not (the ruling), and a copy,
/// which was not cast (CR 707.10), never has it.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let x = item.x as i16;
    pump(g, item.object, creature, -x, -x);
    // A sentence ends (CR 608.2c): the state triggers judge what it did before the next (`Game::end_sentence`).
    g.end_sentence();
    if g.history().fact_as_cast(item.object) != 0 {
        pump(g, item.object, creature, -3, -3);
    }
    Ok(())
}

/// "If you controlled a Faerie as you cast this spell": asked once, as the spell becomes cast. "Faerie" is a word of its
/// text (CR 612), and a changeling is every creature type (CR 702.73a).
fn controlled_a_faerie(q: &dyn Query, me: &Asker) -> u32 {
    let faerie = q.words(me.source).subtype(Subtype::Faerie);
    q.permanents_of(me.controller).into_iter().any(|id| q.chars(id).has_subtype(faerie)) as u32
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    cast_fact: Some(controlled_a_faerie),
    ..oracle::FAERIE_FENCING
};
