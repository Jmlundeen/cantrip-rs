use cantrip_api::*;
use cantrip_mechanics::{double, mana, targets};
use cantrip_oracle as oracle;

/// Until end of turn, double target creature's power and it gains first strike. Doubling a power is +X/+0 for the
/// power the creature has as the spell resolves (CR 701.10a-c; a negative power gets -X/-0), read once and fixed
/// (CR 608.2h).
fn double(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        // "Double target creature's power and it gains first strike": one instruction (CR 608.2c).
        instruction(g, |g| {
            double::power(g, item.object, creature);
            g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::FirstStrike])), Duration::EndOfTurn);
        });
    }
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_one_of(&[Color::Red, Color::White]), "{T}: Add {R} or {W}.")],
    enters: Some(mana::enters_tapped),
    ..oracle::LEGION_STRONGHOLD
};

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], double),
    back: Some(&BACK),
    ..oracle::LEGION_LEADERSHIP
};
