use cantrip_api::*;
use cantrip_mechanics::{creatures_have_total_power_at_least, effects, triggers};
use cantrip_oracle as oracle;

/// "Whenever you attack while creatures you control have total power 8 or greater" (CR 508.3d, 603.2): a condition of the
/// trigger event, judged as attackers are declared and not again as the ability resolves (CR 603.4, `only_if`).
fn eight_power(q: &dyn Query, asker: &Asker) -> bool {
    creatures_have_total_power_at_least(q, asker.controller, 8)
}

/// "Put a +1/+1 counter on this creature. Untap it."
fn counter_and_untap(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::plus_one_counter_on_this(g, item)?;
    g.untap(item.object);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOU_ATTACK,
        counter_and_untap,
        "Whenever you attack while creatures you control have total power 8 or greater, put a +1/+1 counter on this creature. Untap it.",
    )
    .only_if(eight_power)],
    ..oracle::TACTICAL_OFFICER
};
