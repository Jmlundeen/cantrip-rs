use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Instant and enchantment spells you cast cost {2} less to cast." (CR 604.1).
fn instant_enchantment_spells_cast_cost_rule(
    q: &dyn Query,
    source: ObjectId,
    player: PlayerId,
    costed: Costed,
    _: &[Target],
) -> CostDelta {
    let asker = &Asker::new(source, q.controller(source));
    let Costed::Spell(spell) = costed else { return CostDelta::NONE };
    if player == asker.controller
        && (q.chars(spell).is(CardType::Instant)
            || q.chars(spell).is(CardType::Enchantment))
    {
        CostDelta::less(2)
    } else {
        CostDelta::NONE
    }
}

pub static CARD: CardDef = CardDef {
    cost_changes: &[instant_enchantment_spells_cast_cost_rule],
    ..oracle::MANA_MATRIX
};
