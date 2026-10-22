use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// Mode 1: "Target creature gets +2/+0 and gains first strike until end of turn. Put a +1/+1 counter on it." (CR 611.2,
/// 702.7, 122.1.)
fn charge_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        instruction(g, |g| {
            g.add_effect(item.object, creature, EffectKind::Pump(2, 0), Duration::EndOfTurn);
            g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::FirstStrike])), Duration::EndOfTurn);
        });
        g.end_sentence();
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(
        |g, item| {
            effects::pump_creatures_you_control(g, item, 2, 0);
            Ok(())
        },
        "Creatures you control get +2/+0 until end of turn.",
    ),
    mode(charge_one, "Target creature gets +2/+0 and gains first strike until end of turn. Put a +1/+1 counter on it.")
        .targets(&[targets::CREATURE]),
]);

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&MODAL),
    ..oracle::CHARGE_THE_SANCTUM
};
