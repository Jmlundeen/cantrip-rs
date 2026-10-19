use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, history::entered_this_turn, targets};
use cantrip_oracle as oracle;

/// "As long as this Aura entered this turn, enchanted creature has hexproof" (CR 702.11, 611.3a): asked of the event
/// history whenever the layers apply, so it holds from the moment the Aura is on the battlefield until the turn ends, and
/// an Aura that left and came back is a new object that entered when it came back (CR 400.7).
fn enchanted_and_entered_this_turn(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    attached_object(view, source, affected, so_far) && entered_this_turn(view.history(), source)
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE_YOU_CONTROL),
    statics: &[
        static_ability(Layer::Ability, enchanted_and_entered_this_turn, cantrip_mechanics::statics::have_hexproof, "As long as this Aura entered this turn, enchanted creature has hexproof."),
        static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<1, 1>, "Enchanted creature gets +1/+1."),
    ],
    ..oracle::SHARDMAGES_RESCUE
};
