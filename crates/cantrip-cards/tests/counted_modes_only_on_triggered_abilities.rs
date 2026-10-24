//! "Choose up to that many" (`ModalDef::COUNTED`, CR 603.12a) reads a triggered ability's count as it is put on the stack.
//! A spell has no such count, and its modes would read the sentinel as 255 modes; an activated ability is refused where it
//! is built (`modal_ability`, `AbilityDef::modal`). This test holds every spell to a fixed number of modes.

use cantrip_api::*;

#[test]
fn no_modal_spell_chooses_that_many_modes() {
    let table = CardTable::new(cantrip_cards::all());
    let mut modal = 0;
    let mut wrong = Vec::new();
    for card in table.cards.iter() {
        if let Some(m) = card.modal {
            modal += 1;
            if m.max == ModalDef::COUNTED {
                wrong.push(card.name);
            }
        }
    }
    assert!(wrong.is_empty(), "spells with \"that many\" modes, which only a triggered ability counts: {wrong:?}");
    assert!(modal > 0, "no modal spell was found: the test reads nothing");
}
