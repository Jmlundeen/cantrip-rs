use cantrip_api::*;
use cantrip_mechanics::{tokens, triggers};
use cantrip_oracle as oracle;

/// "Whenever you cast an artifact spell, put a charge counter on this artifact." (CR 603.2): the spell is on the stack
/// and the artifact is already on the battlefield.
fn add_charge(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::Charge, 1);
    Ok(())
}

/// "{4}, {T}: Create a 1/1 colorless Borg artifact creature token for each charge counter on this artifact. Activate
/// only as a sorcery." (CR 602.2, 122.1): the count is read as the ability resolves.
fn tokens_per_charge_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let count = g.obj(item.object).counters(CounterKind::Charge).max(0) as u32;
    tokens::create(g, item.controller, &tokens::BORG, count)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::ARTIFACT_SPELL_YOU_CAST,
            add_charge,
            "Whenever you cast an artifact spell, put a charge counter on this artifact.",
        ),
        ability(
            cost("{4}, {T}"),
            tokens_per_charge_counter,
            "{4}, {T}: Create a 1/1 colorless Borg artifact creature token for each charge counter on this artifact. Activate only as a sorcery.",
        )
        .sorcery_speed(),
    ],
    ..oracle::MATURATION_CHAMBER
};

pub static EXTRA: &[&CardDef] = &[&tokens::BORG];
