use cantrip_api::*;
use cantrip_mechanics::effects::plus_one_counter_on_this;
use cantrip_mechanics::sneak;
use cantrip_mechanics::triggers::YOU_GAIN_LIFE;
use cantrip_oracle as oracle;

/// "Sneak {W}" (CR 702.190a): cast for {W} and an unblocked attacker you control returned to its owner's hand, in your
/// declare blockers step; "he enters tapped and attacking" the same player, planeswalker or battle as that creature
/// (CR 702.190b), which the core does for a permanent spell cast this way. "Lifelink" is printed (CR 702.15). "Whenever you
/// gain life, put a +1/+1 counter on Leonardo": "Leonardo" is this object (CR 201.5), once for each life-gaining event
/// (CR 119.9, the rulings).
pub static CARD: CardDef = CardDef {
    ways: &[sneak::way(mana("{W}"), "Sneak {W}")],
    abilities: &[triggered(YOU_GAIN_LIFE, plus_one_counter_on_this, "Whenever you gain life, put a +1/+1 counter on Leonardo.")],
    ..oracle::LEONARDO_CUTTING_EDGE
};
