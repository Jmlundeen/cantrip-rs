use cantrip_api::*;
use cantrip_mechanics::{tokens, triggers};
use cantrip_oracle as oracle;

/// "You can't sacrifice Treasure tokens." (CR 101.2, 701.21a): a player sacrifices only a permanent they control, so the
/// Treasures this stops are its own controller's; an opponent sacrificing their own Treasure pays that cost as usual.
fn no_treasures(q: &dyn Query, asker: &Asker, object: ObjectId) -> bool {
    let it = q.obj(object);
    it.token && it.controller == asker.controller && q.chars(object).has_subtype(Subtype::Treasure)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::EACH_UPKEEP,
        tokens::create_a_treasure,
        "At the beginning of each upkeep, create a Treasure token.",
    )],
    sacrifice_bans: &[no_treasures],
    ..oracle::QUARK_HUMBLE_PROPRIETOR
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
