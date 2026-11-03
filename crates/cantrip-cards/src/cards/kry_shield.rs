use cantrip_api::*;
use cantrip_mechanics::{prevention, targets};
use cantrip_oracle as oracle;

/// Prevent all damage dealt by the creature this turn, then add its mana value to its toughness (CR 615.1, 608.2h).
fn protect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_effect(
        item.object,
        creature,
        EffectKind::Prevention { card: item.ability.card, index: 0, to: Target::None, shield: Shield::All },
        Duration::EndOfTurn,
    );
    g.end_sentence();
    let toughness = g.mana_value(creature) as i32;
    g.add_effect(item.object, creature, EffectKind::Pump(0, toughness), Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{2}").and(TAP), protect,
        "{2}, {T}: Prevent all damage that would be dealt this turn by target creature you control. That creature gets +0/+X until end of turn, where X is its mana value.")
        .targets(&[targets::CREATURE_YOU_CONTROL])],
    granted_preventions: &[prevention::DAMAGE_DEALT_BY.text("Prevent all damage that would be dealt this turn by target creature you control. That creature gets +0/+X until end of turn, where X is its mana value.")],
    ..oracle::KRY_SHIELD
};
