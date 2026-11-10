use cantrip_api::*;
use cantrip_mechanics::{statics, triggers};
use cantrip_oracle as oracle;

const TEXT: &str = "Historic creatures you control have flying.";

/// "Historic creatures you control" (CR 700.6, 613.1f, layer 6): a creature under the source's controller's control that is
/// historic - an artifact, a legendary permanent, or a Saga - judged on what the earlier layers made it, so a permanent
/// layer 4 turned into an artifact creature gets flying and one that stops being historic loses it.
fn historic_creatures_you_control(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    statics::your_creatures(q, source, affected, so_far) && triggers::is_historic_chars(so_far)
}

// Flying is the core's keyword (CR 702.9), granted with `statics::have_flying`: the card prints none of its own.
pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::Ability, historic_creatures_you_control, statics::have_flying, TEXT)],
    ..oracle::ON_THE_SHOULDERS_OF_GIANTS
};
