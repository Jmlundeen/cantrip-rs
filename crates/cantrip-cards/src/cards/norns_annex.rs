//! Norn's Annex. Its mana cost's Phyrexian symbols are announced as it is cast (CR 601.2b), as any spell's are.

use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    attack_costs: &[annex],
    ..oracle::NORNS_ANNEX
};

/// "Creatures can't attack you or planeswalkers you control unless their controller pays {W/P} for each of those
/// creatures." (CR 508.1d, 508.1h): each attacker's {W/P} is paid with {W} or 2 life, chosen for each (CR 107.4f; the
/// rulings), as the attackers are declared.
fn annex(q: &dyn Query, asker: &Asker, _: ObjectId, target: Target) -> Option<ManaCost> {
    let mut phyrexian = [0; 5];
    phyrexian[Color::White as usize] = 1;
    statics::attacks_you_or_your_planeswalker(q, asker, target).then_some(ManaCost { phyrexian, ..ManaCost::ZERO })
}
