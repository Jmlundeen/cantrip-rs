use cantrip_api::*;
use cantrip_mechanics::{effects, select, triggers};
use cantrip_oracle as oracle;

const NONLAND_PERMANENT: select::Selection = select::Selection::of(select::up_to(1), &[select::NONLAND]).called("nonland permanent");

/// "You may sacrifice a nonland permanent. When you do, put a +1/+1 counter on each creature you control" (CR 701.21a, 603.12):
/// the permanent is chosen as the ability resolves, not as a cost, and may be Felothar itself; the reflexive ability exists only
/// for a permanent actually sacrificed, and counts the creatures its controller has as it resolves.
fn sacrifice_for_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !effects::may_sacrifice(g, item.controller, item.object, NONLAND_PERMANENT)?.is_empty() {
        g.reflexive_trigger(item.object, item.controller, AbilityRef::delayed(g.card_id(&CARD), 0), Target::None);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS_OR_ATTACKS,
        sacrifice_for_counters,
        "Whenever Felothar enters or attacks, you may sacrifice a nonland permanent. When you do, put a +1/+1 counter on each creature you control.",
    )],
    delayed: &[delayed_trigger(
        effects::plus_one_counter_on_each_creature_you_control,
        "When you do, put a +1/+1 counter on each creature you control.",
    )],
    ..oracle::FELOTHAR_DAWN_OF_THE_ABZAN
};
