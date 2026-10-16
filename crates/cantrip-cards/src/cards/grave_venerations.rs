use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// Each opponent loses 1 life and you gain 1 life.
fn drain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.each_opponent_loses_life(item.controller, 1);
    g.gain_life(item.controller, 1);
    Ok(())
}

/// "If you're the monarch" (CR 603.4): asked as the ability would trigger and again as it resolves.
fn you_are_monarch(q: &dyn Query, asker: &Asker) -> bool {
    q.state().player(asker.controller).monarch
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, cantrip_mechanics::effects::become_monarch, "When this enchantment enters, you become the monarch."),
        triggered(cantrip_mechanics::triggers::CREATURE_YOU_CONTROL_DIES, drain, "Whenever a creature you control dies, each opponent loses 1 life and you gain 1 life.")
            .look_back(),
        triggered(cantrip_mechanics::triggers::YOUR_END_STEP, cantrip_mechanics::effects::return_first_target_to_hand, "At the beginning of your end step, if you're the monarch, return up to one target creature card from your graveyard to your hand.")
            .targets(&[cantrip_mechanics::targets::CREATURE_CARD_IN_YOUR_GRAVEYARD.between(0, 1)])
            .condition(you_are_monarch),
    ],
    ..oracle::GRAVE_VENERATIONS
};
