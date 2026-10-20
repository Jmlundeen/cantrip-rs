use cantrip_api::*;
use cantrip_mechanics::{damage_target, statics::{have_vigilance, your_creature_tokens}, targets};
use cantrip_oracle as oracle;

/// "It deals 4 damage to target attacking or blocking creature." The card is the source, as it last existed in the hand
/// it was discarded from (CR 113.7a, 608.2h); nothing happens if the target is no longer legal (CR 608.2b).
fn four_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 4);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_any_color(1).only_for(ManaRestriction::PLANESWALKER_SPELLS), "{T}: Add one mana of any color. Spend this mana only to cast a planeswalker spell."),
        ability(cost("{1}{W}").and(DISCARD_THIS), four_damage, "{1}{W}, Discard this card: It deals 4 damage to target attacking or blocking creature.")
            .targets(&[targets::ATTACKING_OR_BLOCKING_CREATURE])
            .from(ZoneKind::Hand),
    ],
    statics: &[
        static_ability(Layer::PtModify, your_creature_tokens, cantrip_mechanics::statics::plus_pt::<1, 0>, "Creature tokens you control get +1/+0 and have vigilance."),
        static_ability(Layer::Ability, your_creature_tokens, have_vigilance, "Creature tokens you control get +1/+0 and have vigilance."),
    ],
    ..oracle::GIDEONS_MEMORIAL
};
