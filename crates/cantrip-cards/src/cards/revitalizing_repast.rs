use cantrip_api::*;
use cantrip_mechanics::{mana, targets};
use cantrip_oracle as oracle;

/// Put a +1/+1 counter on target creature. It gains indestructible until end of turn.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(creature) = item.targets[0] {
        g.add_counters(creature, CounterKind::PlusOne, 1);
        g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::Indestructible])), Duration::EndOfTurn);
    }
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_one_of(&[Color::Black, Color::Green]), "{T}: Add {B} or {G}.")],
    enters: Some(mana::enters_tapped),
    ..oracle::OLD_GROWTH_GROVE
};

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    back: Some(&BACK),
    ..oracle::REVITALIZING_REPAST
};
