use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, targets};
use cantrip_oracle as oracle;

/// "As long as this Aura entered this turn, enchanted creature has hexproof" (CR 702.11): every new object records the
/// turn it was made in `controlled_since`, which for an Aura is the turn it entered. A static ability is computed by the
/// layers, which read the state and not the event history, so this is the one "entered this turn" not asked of the
/// history; it would be wrong only for an Aura whose control changed the turn it entered.
fn enchanted_and_entered_this_turn(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    attached_object(view, source, affected, so_far) && view.state().obj(source).controlled_since == view.state().turn.number
}

fn hexproof(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Hexproof);
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE_YOU_CONTROL),
    statics: &[
        static_ability(Layer::Ability, enchanted_and_entered_this_turn, hexproof, "As long as this Aura entered this turn, enchanted creature has hexproof."),
        static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<1, 1>, "Enchanted creature gets +1/+1."),
    ],
    ..oracle::SHARDMAGES_RESCUE
};
