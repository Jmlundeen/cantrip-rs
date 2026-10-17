use cantrip_api::*;
use cantrip_oracle as oracle;

/// As this creature enters, look at an opponent's hand, then choose any card name (CR 614.12a).
fn enters(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    cantrip_mechanics::hand::look_at_an_opponents_hand_then_choose_a_name(g, entering)
}

/// Spells your opponents cast with the chosen name cost {2} more to cast. Activated abilities of sources with the
/// chosen name cost {2} more to activate unless they're mana abilities (CR 601.2f, 602.2b): the rules ask this only of
/// the abilities that use the stack.
fn tax(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    let Some(chosen) = q.obj(source).marks.chosen_card else { return CostDelta::NONE };
    let (object, spell) = match costed {
        Costed::Spell(id) => (id, true),
        Costed::Ability(id, _) => (id, false),
    };
    if !cantrip_mechanics::library::object_has_name(q, object, chosen) || (spell && player == q.controller(source)) {
        return CostDelta::NONE;
    }
    CostDelta::more(2)
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters),
    cost_changes: &[tax],
    ..oracle::ANOINTED_PEACEKEEPER
};
