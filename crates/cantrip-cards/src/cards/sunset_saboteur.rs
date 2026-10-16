use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::ward_discard;
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on target creature an opponent controls."
fn counter_effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(id) = targets::object(g, item, 0) {
        g.add_counters(id, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ward_discard("Ward\u{2014}Discard a card. (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player discards a card.)"),
        triggered(cantrip_mechanics::triggers::THIS_ATTACKS, counter_effect, "Whenever this creature attacks, put a +1/+1 counter on target creature an opponent controls.")
            .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS]),
    ],
    ..oracle::SUNSET_SABOTEUR
};
