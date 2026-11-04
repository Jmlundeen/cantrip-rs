//! Riot (CR 702.136a), a static ability: "You may have this permanent enter with an additional +1/+1 counter on it. If you
//! don't, it gains haste." It is a core keyword (`CardDef::riot`, counted in `Chars::riot`): each instance is one of the
//! replacements that modify how the permanent enters, ordered with the others (CR 614.12a, 616.1, 702.136b, `ops::apply_riot`)
//! and judged on the permanent as it would exist on the battlefield, so riot granted to it by another permanent counts. A
//! printed riot needs no card file.
//!
//! What a card prints is the grant: "[Permanents] have riot", `have_riot` as a layer 6 static's `apply`.

use cantrip_api::*;

/// "... have riot" (CR 613.1f, layer 6): one more instance of riot. Each instance is chosen separately (CR 702.136b), so a
/// creature granted riot twice, or printing it and granted it, makes two choices. Spider-Punk ("Other Spiders you control
/// have riot") and Rhythm of the Wild ("Nontoken creatures you control have riot") print it.
pub fn have_riot(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.riot += 1;
}
