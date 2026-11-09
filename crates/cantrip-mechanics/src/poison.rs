//! Poison counters from damage, shared by Pit Scorpion and Serpent Generator's
//! Snake token. This printed damage trigger also applies to noncombat damage;
//! it is distinct from the poisonous keyword (CR 702.70).
use cantrip_api::*;

/// The player dealt damage gets the printed number of poison counters, once
/// for the damage event, independently of its amount (CR 608.2c, 122.1).
fn give<const N: u32>(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(player) = item.subject.player() {
        g.give_poison(player, N);
    }
    Ok(())
}

/// "Whenever this creature deals damage to a player, that player gets N poison
/// counters" (CR 603.2, 608.2c). Pit Scorpion and Serpent Generator's Snake print
/// this with one counter. It is not targeted and does not require combat damage.
pub const fn on_damage_to_player<const N: u32>(text: &'static str) -> AbilityDef {
    triggered(when(EventMask::DAMAGE, crate::triggers::this_deals_damage_to_a_player), give::<N>, text)
}
