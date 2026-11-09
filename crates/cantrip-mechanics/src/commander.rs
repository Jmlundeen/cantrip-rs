//! Commander recipes over the core designation and fixed identity queries (CR 903.3, 903.4).
use cantrip_api::*;

/// Arcane Signet, Command Tower, Commander's Sphere and Path of Ancestry:
/// "Add one mana of any color in your commander's color identity." (CR 903.4f).
/// An empty identity still offers the ability, pays its cost and adds nothing (CR 106.5).
pub fn colors(q: &dyn Query, asker: &Asker) -> ColorSet {
    q.commander_color_identity(asker.controller).unwrap_or(ColorSet::NONE)
}

pub const MANA: AbilityDef = mana_ability(
    TAP,
    add_any_color(1),
    "{T}: Add one mana of any color in your commander's color identity.",
)
.colors_from(colors);
