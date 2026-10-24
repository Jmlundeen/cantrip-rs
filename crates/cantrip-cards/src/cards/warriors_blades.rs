use cantrip_api::*;
use cantrip_mechanics::{attach, effects, statics, targets, triggers};
use cantrip_oracle as oracle;

/// "This ability costs {1} less to activate for each +1/+1 counter on the creature it targets."
/// The announced target determines the equip cost (CR 601.2f, 602.2b); only generic mana is reduced (CR 118.7a).
fn less_per_counter_on_target(q: &dyn Query, _: &Asker, _: Costed, targets: &[Target]) -> CostDelta {
    let counters = targets.iter().filter_map(|t| t.object())
        .map(|creature| q.state().obj(creature).counters(CounterKind::PlusOne).clamp(0, 255) as u8).sum();
    CostDelta::less(counters)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            |g, item| effects::damage_target_and_gain_life(g, item, 3, 3),
            "When this Equipment enters, it deals 3 damage to any target and you gain 3 life.",
        ).targets(&[targets::ANY]),
        attach::equip(mana("{3}"), "Equip {3}. This ability costs {1} less to activate for each +1/+1 counter on the creature it targets.")
            .cost_change(less_per_counter_on_target),
    ],
    statics: &[static_ability(
        Layer::PtModify, attach::attached_object, statics::plus_pt::<2, 1>, "Equipped creature gets +2/+1.",
    )],
    ..oracle::WARRIORS_BLADES
};
