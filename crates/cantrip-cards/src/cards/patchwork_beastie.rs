use cantrip_api::*;
use cantrip_mechanics::{delirium, effects, triggers::YOUR_UPKEEP};
use cantrip_oracle as oracle;

/// "Delirium - This creature can't attack ... unless there are four or more card types among cards in your graveyard"
/// (CR 508.1c, 702.138a): asked of this creature as attackers are declared, "your" its controller's.
fn attacks_only_with_delirium(q: &dyn Query, source: ObjectId, attacker: ObjectId) -> bool {
    source != attacker || delirium(q, q.obj(source).controller)
}

/// "... can't ... block unless there are four or more card types among cards in your graveyard" (CR 509.1b).
fn blocks_only_with_delirium(q: &dyn Query, source: ObjectId, blocker: ObjectId, _: ObjectId) -> bool {
    source != blocker || delirium(q, q.obj(source).controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOUR_UPKEEP,
        effects::may_mill_a_card,
        "At the beginning of your upkeep, you may mill a card. (You may put the top card of your library into your graveyard.)",
    )],
    attack_rules: &[attacks_only_with_delirium],
    block_rules: &[blocks_only_with_delirium],
    ..oracle::PATCHWORK_BEASTIE
};
