use cantrip_api::*;
use cantrip_mechanics::{channel, damage_target, targets};
use cantrip_oracle as oracle;

/// "It deals 4 damage to target attacking or blocking creature." The card is the source, as it last existed in the hand
/// it was discarded from (CR 113.7a, 608.2h); nothing happens if the target is no longer legal (CR 608.2b).
fn four_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 4);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::W), "{T}: Add {W}."),
        ability(cost("{2}{W}").and(DISCARD_THIS), four_damage, "Channel \u{2014} {2}{W}, Discard this card: It deals 4 damage to target attacking or blocking creature. This ability costs {1} less to activate for each legendary creature you control.")
            .targets(&[targets::ATTACKING_OR_BLOCKING_CREATURE])
            .from(ZoneKind::Hand),
    ],
    own_cost_changes: &[channel::cheaper_per_legendary_creature],
    ..oracle::EIGANJO_SEAT_OF_THE_EMPIRE
};
