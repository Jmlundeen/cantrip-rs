use cantrip_api::*;
use cantrip_mechanics::effects::{plus_one_counter_on_this, put_its_counters_on_target};
use cantrip_mechanics::statics::{have_flying_and_vigilance, itself};
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::{THIS_DIES, YOU_GAIN_LIFE};
use cantrip_oracle as oracle;

/// "As long as you've lost life this turn": whether this creature's controller has lost any life this turn (CR 119.3), however
/// much they have gained besides (the ruling). Asked of the event history each time the layers apply (CR 611.3a), so the
/// abilities come the moment the life is lost and go as the next turn begins.
fn you_lost_life_this_turn(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    itself(view, source, affected, so_far) && view.history().life_lost(view.state().obj(source).controller) > 0
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(YOU_GAIN_LIFE, plus_one_counter_on_this,"Whenever you gain life, put a +1/+1 counter on this creature."),
        triggered(THIS_DIES, put_its_counters_on_target, "When this creature dies, put its counters on target creature you control.")
            .targets(&[targets::CREATURE_YOU_CONTROL]),
    ],
    statics: &[static_ability(
        Layer::Ability,
        you_lost_life_this_turn,
        have_flying_and_vigilance,
        "As long as you've lost life this turn, this creature has flying and vigilance.",
    )],
    ..oracle::ESSENCE_CHANNELER
};
