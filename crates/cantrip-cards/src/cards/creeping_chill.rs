use cantrip_api::*;
use cantrip_mechanics::triggers::PUT_INTO_YOUR_GRAVEYARD_FROM_YOUR_LIBRARY;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

const ASK: Question = Question::new("ExileCreepingChill", "exile Creeping Chill to deal 3 damage to each opponent and gain 3 life?", "It was put into your graveyard from your library", "Exile it", "Don't");

/// "Creeping Chill deals 3 damage to each opponent and you gain 3 life."
fn drain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::damage_each_opponent_and_gain_life(g, item, 3)
}

/// "When Creeping Chill is put into your graveyard from your library, you may exile it. If you do, Creeping Chill deals 3
/// damage to each opponent and you gain 3 life." (CR 608.2c, 400.7): the card is exiled only if it is still the object that
/// reached the graveyard; the drain happens only if it was exiled.
fn exile_to_drain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !effects::ask_may_this_card(g, item, Purpose::Ask(&ASK))? {
        return Ok(());
    }
    let exiled = g.exile(item.object);
    // "If you do": the card counts as exiled only if it reached exile.
    g.obj(exiled).zone.is(ZoneKind::Exile).then(|| drain(g, item)).unwrap_or(Ok(()))
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], drain),
    abilities: &[triggered(
        PUT_INTO_YOUR_GRAVEYARD_FROM_YOUR_LIBRARY,
        exile_to_drain,
        "When Creeping Chill is put into your graveyard from your library, you may exile it. If you do, Creeping Chill deals 3 damage to each opponent and you gain 3 life.",
    )],
    ..oracle::CREEPING_CHILL
};
