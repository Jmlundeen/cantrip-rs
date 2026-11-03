use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Prevent combat damage dealt by the creature the effect is about (CR 615.1).
fn combat_damage_dealt_by(_: &dyn Query, _: &Asker, creature: ObjectId, damage: &DamageEvent) -> bool {
    damage.combat && damage.source == creature
}

/// The prevention lasts until end of turn and applies to damage dealt to any recipient (CR 615.1, 611.2a).
fn prevent_combat_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_effect(item.object, creature, EffectKind::Prevention { card: item.ability.card, index: 0, to: Target::None, shield: Shield::All }, Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{2}").and(TAP), prevent_combat_damage, "{2}, {T}: Prevent all combat damage that would be dealt by target creature this turn.")
        .targets(&[targets::CREATURE])],
    granted_preventions: &[GrantedPreventionDef {
        hint: "prevents all combat damage dealt by that creature",
        damage: "combat damage",
        of_it: Some("dealt by it"),
        applies: combat_damage_dealt_by,
        then: None,
        text: "Prevent all combat damage that would be dealt by target creature this turn.",
    }],
    ..oracle::HORN_OF_DEAFENING
};
