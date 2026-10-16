use cantrip_api::*;
use cantrip_mechanics::{damage_target, statics::{have_vigilance, your_creature_tokens}, targets};
use cantrip_oracle as oracle;

/// "Target attacking or blocking creature" (CR 506.4, 508.1, 509.1): a creature on the battlefield that is attacking or
/// blocking now.
fn attacking_or_blocking_creature(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id)
        && q.chars(id).is(CardType::Creature)
        && (q.is_attacking(id) || q.state().blocks.iter().any(|b| b.blocker == id)))
}

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
            .targets(&[target(attacking_or_blocking_creature)])
            .from(ZoneKind::Hand),
    ],
    statics: &[
        static_ability(Layer::PtModify, your_creature_tokens, cantrip_mechanics::statics::plus_pt::<1, 0>, "Creature tokens you control get +1/+0 and have vigilance."),
        static_ability(Layer::Ability, your_creature_tokens, have_vigilance, "Creature tokens you control get +1/+0 and have vigilance."),
    ],
    ..oracle::GIDEONS_MEMORIAL
};
