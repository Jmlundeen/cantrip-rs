use cantrip_api::*;
use cantrip_mechanics::{mana, targets};
use cantrip_oracle as oracle;

fn card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(q, asker, t, &[])
}

/// Return target card from your graveyard to your hand.
fn recover(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.return_to_hand(&[card]);
    }
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add(Mana::G), "{T}: Add {G}.")],
    enters: Some(mana::enters_tapped),
    ..oracle::BALA_GED_SANCTUARY
};

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(card_in_your_graveyard)], recover),
    back: Some(&BACK),
    ..oracle::BALA_GED_RECOVERY
};
