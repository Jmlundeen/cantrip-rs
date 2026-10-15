use cantrip_api::*;
use cantrip_oracle as oracle;

fn charge(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::Charge, 1);
    Ok(())
}

/// "+1/+1 for each charge counter on this artifact" (CR 613.4c).
fn apply(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let bonus = q.state().obj(source).counters(CounterKind::Charge);
    chars.power = chars.power.map(|v| v + bonus);
    chars.toughness = chars.toughness.map(|v| v + bonus);
}

pub static CARD: CardDef = CardDef {
    enters: Some(cantrip_mechanics::chosen_type::enters_choosing_a_creature_type),
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever you cast a spell of the chosen type, put a charge counter on this artifact.",
        listens: EventMask::SPELL_CAST,
        matches: cantrip_mechanics::chosen_type::you_cast_chosen_type,
        effect: charge,
        ..TriggeredDef::BLANK
    })],
    statics: &[StaticDef {
        text: "Creatures you control of the chosen type get +1/+1 for each charge counter on this artifact.",
        layer: Layer::PtModify,
        affects: cantrip_mechanics::chosen_type::creatures_you_control_of_chosen_type,
        apply,
        ..StaticDef::BLANK
    }],
    ..oracle::DOOR_OF_DESTINIES
};
