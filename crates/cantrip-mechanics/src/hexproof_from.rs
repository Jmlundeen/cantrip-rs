//! Hexproof from [quality] (CR 702.11d-h): "This permanent can't be the target of [quality] spells your opponents
//! control or abilities your opponents control from [quality] sources." The core judges it as a target is chosen
//! (`Chars::hexproof_from`); printed on a card it is `CardDef::hexproof_from` (Knight of Grace's "hexproof from black",
//! Sphinx of the Guildpact's "hexproof from monocolored"). This is the static ability that gives it to another object.

use cantrip_api::*;

/// "... has hexproof from [quality]" (CR 702.11d, layer 6) as a static ability's `StaticDef::apply`, the qualities being
/// `HexproofFrom` bits: Dragonfire Blade's "Equipped creature ... has hexproof from monocolored"
/// (`gives::<{ HexproofFrom::MONOCOLORED.0 }>`).
pub fn gives<const QUALITIES: u32>(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.hexproof_from = chars.hexproof_from.and(HexproofFrom(QUALITIES));
}
