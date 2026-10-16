use cantrip_api::*;
use cantrip_mechanics::miracle;
use cantrip_oracle as oracle;

/// "Nonland cards in your hand have miracle {0}" (CR 702.94a).
fn nonland_in_your_hand(q: &dyn Query, asker: &Asker, card: ObjectId) -> bool {
    miracle::in_your_hand(q, asker, card) && !q.chars(card).is(CardType::Land)
}

pub static CARD: CardDef = CardDef {
    granted_miracles: &[GrantedMiracle { text: "Molecule Man's miracle {0}", has: nonland_in_your_hand, cost: |_, _| Some(ManaCost::ZERO) }],
    ..oracle::MOLECULE_MAN
};
