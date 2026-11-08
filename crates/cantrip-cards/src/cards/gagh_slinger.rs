use cantrip_api::*;
use cantrip_mechanics::{effects, lands, triggers};
use cantrip_oracle as oracle;

/// "Klingon spells and Warrior spells you cast cost {1} less to cast." (CR 601.2f, 118.7a): only its controller's spells,
/// judged on the spell's own characteristics as it is cast, once whether it is a Klingon, a Warrior or both, the two
/// subtypes read through the words of this card's text (CR 612). A generic reduction takes only the generic part (118.7c).
fn klingon_or_warrior_spells_cost_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    let Costed::Spell(spell) = costed else { return CostDelta::NONE };
    if player != q.controller(source) {
        return CostDelta::NONE;
    }
    let chars = q.chars(spell);
    if chars.has_subtype(q.words(source).subtype(Subtype::Klingon)) || chars.has_subtype(q.words(source).subtype(Subtype::Warrior)) {
        CostDelta::less(1)
    } else {
        CostDelta::NONE
    }
}

/// "Whenever another Klingon or Warrior you control enters" (CR 603.6a): a permanent other than this one, under this
/// creature's controller's control as it entered, that is a Klingon or a Warrior (a token too), the types read through the
/// words of this card's text (CR 612); once for that permanent, though it is both.
fn another_klingon_or_warrior_enters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    triggers::another_of_subtype_you_control_enters::<{ lands::sub(Subtype::Klingon) }>(q, me, event)
        .or_else(|| triggers::another_of_subtype_you_control_enters::<{ lands::sub(Subtype::Warrior) }>(q, me, event))
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, another_klingon_or_warrior_enters),
        |g, item| {
            effects::damage_each_opponent(g, item, 1);
            Ok(())
        },
        "Whenever another Klingon or Warrior you control enters, this creature deals 1 damage to each opponent.",
    )],
    cost_changes: &[klingon_or_warrior_spells_cost_less],
    ..oracle::GAGH_SLINGER
};
