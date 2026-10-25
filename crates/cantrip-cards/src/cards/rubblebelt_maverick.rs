use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on target creature."
fn counter_on_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::effects::plus_one_counter_on_target(g, item)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, |g, item| effects::surveil(g, item, 2), "When this creature enters, surveil 2."),
        ability(
            cost("{G}").and(EXILE_THIS),
            counter_on_creature,
            "{G}, Exile this card from your graveyard: Put a +1/+1 counter on target creature. Activate only as a sorcery.",
        )
        .targets(&[targets::CREATURE])
        .from(ZoneKind::Graveyard)
        .sorcery_speed(),
    ],
    ..oracle::RUBBLEBELT_MAVERICK
};
