use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// "Investigate X times." (CR 701.16a): X as the spell was cast, 0 for a copy cast without paying its cost (CR 107.3).
fn investigate_x_times(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for _ in 0..item.x {
        tokens::investigate(g, item.controller)?;
    }
    Ok(())
}

/// The Adventure (CR 715): Follow Him.
pub static FOLLOW_HIM: CardDef = CardDef {
    spell: spell(&[], investigate_x_times),
    ..oracle::FOLLOW_HIM
};

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_n(Mana::C, 2).only_for(ManaRestriction::ABILITIES), "{T}: Add {C}{C}. Spend this mana only to activate abilities.")],
    adventure: Some(&FOLLOW_HIM),
    ..oracle::JAMES_WANDERING_DAD
};

pub static EXTRA: &[&CardDef] = &[&FOLLOW_HIM, &tokens::CLUE];
