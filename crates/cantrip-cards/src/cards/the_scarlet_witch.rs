use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Instant and sorcery spells you cast with mana value 4 or greater cost {X} less to cast, where X is The Scarlet Witch's power"
/// (CR 601.2f, 118.7a, 202.3): asked of the spell as it is cast, only its controller's, and read as the total cost is
/// determined - the mana value it has on the stack, an {X} in its cost counting as the X chosen, and her power as it is then.
/// The reduction is generic mana only; a power of zero or less reduces nothing.
fn instants_and_sorceries_cost_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    let Costed::Spell(spell) = costed else { return CostDelta::NONE };
    if player != q.controller(source) {
        return CostDelta::NONE;
    }
    let chars = q.chars(spell);
    if !(chars.is(CardType::Instant) || chars.is(CardType::Sorcery)) || q.mana_value(spell) < 4 {
        return CostDelta::NONE;
    }
    let power = q.chars(source).power.unwrap_or(0).max(0);
    CostDelta::less(power as u32)
}

pub static CARD: CardDef = CardDef { cost_changes: &[instants_and_sorceries_cost_less], ..oracle::THE_SCARLET_WITCH };
