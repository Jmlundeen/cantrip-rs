use cantrip_api::*;
use cantrip_mechanics::attach;
use cantrip_oracle as oracle;

/// "Enchant Wall" (CR 702.5a, 303.4a).
fn enchanted_wall_can_attack_as_enchant(
    q: &dyn Query,
    asker: &Asker,
    t: Target,
) -> bool {
    cantrip_mechanics::targets::permanent_of_any_subtype(q, asker, t, &[Subtype::Wall])
}

pub static CARD: CardDef = CardDef {
    enchant: Some(target(enchanted_wall_can_attack_as_enchant)),
    // "Enchanted Wall can attack as though it didn't have defender." (CR 702.3b): the permanent it is attached to.
    attacks_despite_defender: &[attach::attached_to_it],
    ..oracle::ANIMATE_WALL
};
