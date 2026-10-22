use cantrip_api::*;
use cantrip_mechanics::replacements::DAMAGE_TO_YOU_AND_YOUR_OTHER_PERMANENTS_IS_DEALT_TO_THIS;
use cantrip_mechanics::triggers::{THIS_DIES, ward};
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// "Damage isn't removed from this creature during cleanup steps" (CR 514.2): this permanent, and no other. A permanent
/// that left and came back is a new object (CR 400.7) that keeps nothing; regeneration still removes all of it (ruling).
fn this_creature(_: &dyn Query, asker: &Asker, object: ObjectId) -> bool {
    object == asker.source
}

/// "When this creature dies, exile it and create ten tapped Treasure tokens." The card it became in the graveyard is
/// exiled if it is still there (CR 400.7); the Treasures are made either way, under the ability's controller (CR 111.2).
fn exile_it_and_make_treasures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = g.found_after_move(item.object) {
        g.exile(card);
    }
    let treasure = g.card_id(&tokens::TREASURE);
    g.create_tokens(treasure, item.controller, 10, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ward::<3>("Ward {3} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {3}.)"),
        triggered(THIS_DIES, exile_it_and_make_treasures, "When this creature dies, exile it and create ten tapped Treasure tokens."),
    ],
    replacements: &[DAMAGE_TO_YOU_AND_YOUR_OTHER_PERMANENTS_IS_DEALT_TO_THIS],
    damage_stays: &[this_creature],
    ..oracle::ANCIENT_ADAMANTOISE
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
