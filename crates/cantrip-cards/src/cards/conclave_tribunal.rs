use cantrip_api::*;
use cantrip_mechanics::{exile_target_until_leaves, targets, triggers::this_enters};
use cantrip_oracle as oracle;

/// "Exile target nonland permanent an opponent controls until this enchantment leaves the battlefield" (CR 610.3).
/// (Convoke is a characteristic of the card: `pay_with`.)
fn exile_until_it_leaves(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target_until_leaves(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When this enchantment enters, exile target nonland permanent an opponent controls until this enchantment leaves the battlefield.",
        listens: EventMask::ZONE_CHANGE,
        matches: this_enters,
        targets: &[targets::NONLAND_PERMANENT_AN_OPPONENT_CONTROLS],
        effect: exile_until_it_leaves,
        ..TriggeredDef::BLANK
    })],
    ..oracle::CONCLAVE_TRIBUNAL
};
