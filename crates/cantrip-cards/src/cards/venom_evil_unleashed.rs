use cantrip_api::*;
use cantrip_mechanics::effects::gain_until_end_of_turn;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "{2}{B}, Exile this card from your graveyard: Put two +1/+1 counters on target creature. It gains deathtouch until end of turn."
/// (CR 122.1, 702.2, 611.2a.) The cost exiles the card, so the ability goes on working from the stack on its own; a target no
/// longer legal gets neither (CR 608.2b).
fn counters_and_deathtouch(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(creature, CounterKind::PlusOne, 2);
    g.end_sentence();
    gain_until_end_of_turn(g, item.object, creature, Keyword::Deathtouch);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{2}{B}").and(EXILE_THIS),
        counters_and_deathtouch,
        "{2}{B}, Exile this card from your graveyard: Put two +1/+1 counters on target creature. It gains deathtouch until end of turn. Activate only as a sorcery.",
    )
    .targets(&[targets::CREATURE])
    .from(ZoneKind::Graveyard)
    .sorcery_speed()],
    ..oracle::VENOM_EVIL_UNLEASHED
};
