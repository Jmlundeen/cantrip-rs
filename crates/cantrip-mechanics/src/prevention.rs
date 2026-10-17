//! Prevention effects a resolving spell or ability makes (CR 615, 611.2): "prevent the next 3 damage that would be dealt to
//! any target this turn" (a shield, CR 615.7) and "prevent all combat damage that would be dealt this turn". The core
//! applies them (`EffectKind::Prevention`): ordered with the replacement effects of the same damage (CR 616.1), never to
//! damage that can't be prevented (CR 615.12), and recorded as they prevent it (`Event::DamagePrevented`). A card lists
//! the definition it uses in its `granted_preventions` and makes the effect with the index it has there.

use cantrip_api::*;

fn any(_: &dyn Query, _: &Asker, _: &DamageEvent) -> bool {
    true
}

fn combat(_: &dyn Query, _: &Asker, d: &DamageEvent) -> bool {
    d.combat
}

/// Damage of any kind from any source, to whom the effect names: "prevent the next 3 damage that would be dealt to any
/// target this turn" (Healing Salve, Healing Grace, Shelter's "prevent all damage").
pub const ANY_DAMAGE: GrantedPreventionDef = GrantedPreventionDef { hint: "prevents the damage", damage: "damage", applies: any };

/// Combat damage only (CR 510.2): "prevent all combat damage that would be dealt this turn" (Fog, Holy Day, Moment's
/// Peace).
pub const COMBAT_DAMAGE: GrantedPreventionDef = GrantedPreventionDef { hint: "prevents all combat damage", damage: "combat damage", applies: combat };

/// Makes the prevention effect the resolving spell or ability's card lists `index`th in its `granted_preventions`, of damage
/// to `to` (`Target::None` for damage to anyone), until end of turn: all of it, or the next N (CR 615.7).
pub fn this_turn(g: &mut dyn Game, item: &StackItem, to: Target, index: u8, shield: Shield) {
    let affected = to.object().unwrap_or(item.object);
    g.add_effect(item.object, affected, EffectKind::Prevention { index, to, shield }, Duration::EndOfTurn);
}

/// "Prevent all combat damage that would be dealt this turn" (CR 615.1): Fog, Holy Day, Moment's Peace. The card lists
/// `COMBAT_DAMAGE` `index`th in its `granted_preventions`.
pub fn all_combat_damage_this_turn(g: &mut dyn Game, item: &StackItem, index: u8) {
    this_turn(g, item, Target::None, index, Shield::All);
}

/// "Prevent the next N damage that would be dealt to [the target in `slot`] this turn" (CR 615.7): a shield on it, if it is
/// still a legal target (CR 608.2b). Healing Salve, Healing Grace. The card lists `ANY_DAMAGE` `index`th in its
/// `granted_preventions`.
pub fn next_damage_to_target(g: &mut dyn Game, item: &StackItem, slot: usize, index: u8, n: u16) {
    if !g.target_legal(item, slot) {
        return;
    }
    this_turn(g, item, item.targets[slot], index, Shield::Next(n));
}
