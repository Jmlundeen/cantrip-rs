use cantrip_api::*;
use cantrip_oracle as oracle;

/// "White spells cost {3} more to cast." (CR 601.2f): any white spell, whoever casts it; "white" is read through Gloom's
/// words (CR 612).
fn white_spells_cost_3_more(q: &dyn Query, source: ObjectId, _: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    let Costed::Spell(spell) = costed else { return CostDelta::NONE };
    if q.chars(spell).colors.has(q.words(source).color(Color::White)) { CostDelta::more(3) } else { CostDelta::NONE }
}

/// "Activated abilities of white enchantments cost {3} more to activate." (CR 601.2f by way of 602.2b): the abilities of
/// white enchantment permanents (CR 109.2), mana abilities included (CR 605.3), whoever activates them; not an ability
/// of an enchantment card activated from another zone. "White" is read through Gloom's words (CR 612).
fn white_enchantment_abilities_cost_3_more(q: &dyn Query, source: ObjectId, _: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    let Costed::Ability(permanent, ..) = costed else { return CostDelta::NONE };
    let chars = q.chars(permanent);
    if q.is_on_battlefield(permanent)
        && chars.colors.has(q.words(source).color(Color::White))
        && chars.is(CardType::Enchantment)
    {
        CostDelta::more(3)
    } else {
        CostDelta::NONE
    }
}

pub static CARD: CardDef = CardDef {
    cost_changes: &[white_spells_cost_3_more, white_enchantment_abilities_cost_3_more],
    ..oracle::GLOOM
};
