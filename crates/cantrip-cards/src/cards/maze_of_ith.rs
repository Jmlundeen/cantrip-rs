use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "... all combat damage that would be dealt to and dealt by that creature" (CR 615.1, 510.2): combat damage whose source
/// or recipient is the creature the effect is about. Once that creature has left the battlefield it is a new object
/// (CR 400.7), which the effect is not about.
fn combat_damage_to_and_by(_: &dyn Query, _: &Asker, creature: ObjectId, d: &DamageEvent) -> bool {
    d.combat && (d.source == creature || d.target == Target::Object(creature))
}

const TO_AND_BY: GrantedPreventionDef = GrantedPreventionDef {
    hint: "prevents all combat damage dealt to and dealt by that creature",
    damage: "combat damage",
    of_it: Some("to and dealt by it"),
    applies: combat_damage_to_and_by,
};

/// "{T}: Untap target attacking creature. Prevent all combat damage that would be dealt to and dealt by that creature this
/// turn." Nothing if the target is no longer an attacking creature (CR 608.2b). It stays in combat, still blocked or
/// unblocked (the rulings): the damage it would deal and be dealt is prevented as it is dealt, after it is assigned
/// (CR 510.1-2), so a trampler still assigns lethal damage to its blockers. Activated after combat damage (an attacker of its
/// controller's), it only untaps it. The prevention lasts until the turn ends (CR 615.4, 514.2), and none applies where
/// damage can't be prevented (CR 615.12, the core).
fn untap_and_prevent(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.untap(creature);
        g.add_effect(item.object, creature, EffectKind::Prevention { card: item.ability.card, index: 0, to: Target::None, shield: Shield::All }, Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP, untap_and_prevent, "{T}: Untap target attacking creature. Prevent all combat damage that would be dealt to and dealt by that creature this turn.")
        .targets(&[targets::ATTACKING_CREATURE])],
    granted_preventions: &[TO_AND_BY],
    ..oracle::MAZE_OF_ITH
};
