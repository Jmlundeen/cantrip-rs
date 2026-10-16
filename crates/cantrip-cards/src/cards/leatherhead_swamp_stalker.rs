use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER;
use cantrip_oracle as oracle;

/// "Leatherhead enters with a hexproof counter on her" (CR 122.1b, 614.1c): a keyword counter that grants hexproof for as
/// long as it is there.
fn enters(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.counters[CounterKind::Hexproof as usize] += 1;
    Ok(())
}

/// "A counter" names no kind (CR 118.3), so the choice is made for her controller in their favour: drawbacks first, then
/// counters with no rules meaning, then the ones whose loss costs something, indestructible last of all.
const SHED_ORDER: [CounterKind; 13] = [
    CounterKind::MinusOne,
    CounterKind::Stun,
    CounterKind::Finality,
    CounterKind::Defense,
    CounterKind::Corpse,
    CounterKind::Charge,
    CounterKind::Lore,
    CounterKind::Loyalty,
    CounterKind::Flying,
    CounterKind::Hexproof,
    CounterKind::Lifelink,
    CounterKind::PlusOne,
    CounterKind::Indestructible,
];

fn counter_to_shed(q: &dyn Query, permanent: ObjectId) -> Option<CounterKind> {
    let o = q.obj(permanent);
    SHED_ORDER.into_iter().find(|&k| o.counters(k) > 0).or_else(|| CounterKind::ALL.into_iter().find(|&k| o.counters(k) > 0))
}

/// "You may remove a counter from her. When you do, destroy target artifact or enchantment that player controls." Not
/// offered when she has no counter. The reflexive trigger (CR 603.12) is queued once the counter is off.
fn remove_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let me = item.object;
    if !targets::permanent(g, me) {
        return Ok(());
    }
    let Some(kind) = counter_to_shed(g, me) else { return Ok(()) };
    if !cantrip_mechanics::effects::ask_may(g, item.controller, me, Purpose::Optional)? {
        return Ok(());
    }
    g.add_counters(me, kind, -1);
    let this = g.card_id(&CARD);
    g.reflexive_trigger(me, item.controller, AbilityRef::delayed(this, 0), item.subject);
    Ok(())
}

/// "Target artifact or enchantment that player controls": the player Leatherhead dealt combat damage to is the one she
/// is attacking (CR 506.4, 510.2); the reflexive trigger goes on the stack with her still attacking.
fn artifact_or_enchantment_of_the_attacked_player(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let Target::Object(id) = t else { return false };
    if !targets::permanent(q, id) {
        return false;
    }
    let c = q.chars(id);
    if !(c.is(CardType::Artifact) || c.is(CardType::Enchantment)) {
        return false;
    }
    q.state()
        .attackers
        .iter()
        .any(|a| a.id == asker.source && matches!(a.target, Target::Player(p) if p == q.controller(id)))
}

fn destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters),
    abilities: &[triggered(DEALS_COMBAT_DAMAGE_TO_PLAYER, remove_counter, "Whenever Leatherhead deals combat damage to a player, you may remove a counter from her. When you do, destroy target artifact or enchantment that player controls.")],
    delayed: &[delayed_trigger(destroy, "When you do, destroy target artifact or enchantment that player controls.")
        .targets(&[TargetSpec { min: 1, max: 1, filter: artifact_or_enchantment_of_the_attacked_player }])],
    ..oracle::LEATHERHEAD_SWAMP_STALKER
};
