use cantrip_api::*;
use cantrip_mechanics::{exile_target_until_leaves, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Exile target nonland permanent an opponent controls until this artifact leaves the battlefield" (CR 610.3): nothing is
/// exiled if the artifact has already left by the time the trigger resolves (CR 610.3b), and a token exiled does not return.
fn exile_until_it_leaves(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target_until_leaves(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::W), "{T}: Add {W}."),
        triggered(THIS_ENTERS, exile_until_it_leaves, "When this artifact enters, exile target nonland permanent an opponent controls until this artifact leaves the battlefield.")
            .targets(&[targets::NONLAND_PERMANENT_AN_OPPONENT_CONTROLS]),
    ],
    ..oracle::WHITE_AURACITE
};
