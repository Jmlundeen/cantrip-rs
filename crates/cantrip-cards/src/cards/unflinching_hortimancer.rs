use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "Whenever you gain life, put a +1/+1 counter on this creature" (CR 119.3): one trigger for each life-gain event,
/// however much life it was, and none once the Hortimancer has left the battlefield.
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::plus_one_counter_on_this(g, item)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggers::ward::<1>("Ward {1} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {1}.)"),
        triggered(triggers::YOU_GAIN_LIFE, grow, "Whenever you gain life, put a +1/+1 counter on this creature."),
    ],
    ..oracle::UNFLINCHING_HORTIMANCER
};
