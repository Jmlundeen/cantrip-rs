use cantrip_api::*;
use cantrip_mechanics::{fight, kicker, targets};
use cantrip_oracle as oracle;

/// "Target creature you control deals damage equal to its power to target creature an opponent controls. If this spell was
/// kicked, the creature you control deals twice that much damage instead." (CR 120.3, 609.7, 608.2b, 702.33d): one-sided, the
/// creature being the source of the damage, so its abilities (lifelink, deathtouch) and colors count. Its power is read as the
/// spell resolves (CR 608.2c) and the damage is that or twice that, never both; a creature with no power deals none (CR 120.8).
/// If either target is illegal then, no damage is dealt.
fn deal_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let (Some(creature), Some(victim)) = (targets::object(g, item, 0), targets::object(g, item, 1)) {
        let amount = fight::damage_equal_to_power(g, creature) * if item.kicked > 0 { 2 } else { 1 };
        g.damage(creature, Target::Object(victim), amount);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL, targets::CREATURE_AN_OPPONENT_CONTROLS], deal_damage),
    // "Kicker\u{2014}Return a land you control to its owner's hand." (CR 702.33a, 601.2b, 601.2h, 701.21): a kicker whose cost is
    // not mana, announced before the targets and paid with the other costs; paying it kicks the spell once (CR 702.33d).
    optional_cost: Some(kicker::non_mana(return_to_hand(1, targets::filters::land))),
    labels: &["Cast without kicking", "Kicker\u{2014}Return a land you control to its owner's hand"],
    ..oracle::CHOCOBO_KICK
};
