use cantrip_api::*;
use cantrip_mechanics::chosen_type::{enters_choosing_a_creature_type, your_creatures_spells_and_cards};
use cantrip_oracle as oracle;

/// "Creatures you control are the chosen type. The same is true for creature spells you control and creature cards you own
/// that aren't on the battlefield." A static ability that works on objects in every zone (CR 611.3a), a type-changing effect
/// in layer 4 (CR 613.1d) that sets the creature type to the one chosen as it entered (CR 614.12a): every other creature type
/// is lost (the ruling; CR 205.1a), card types and other subtypes stay. A creature type confers no ability (the ruling: a
/// Wall can still attack), and the text of other cards is not changed (the ruling). Its reach - what is a creature - can
/// change as other layer-4 effects apply, so it waits for Opalescence however new that is (CR 613.8a). No type chosen (it
/// entered with its abilities lost, CR 614.12), it changes nothing.
fn set_chosen_type(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    if let Some(chosen) = q.state().chosen_subtype(q.linked_to(source)) {
        let kept = chars.subtypes.iter().filter(|t| !t.is_creature_type()).fold(SubtypeSet::NONE, SubtypeSet::with);
        chars.subtypes = kept.with(chosen);
    }
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters_choosing_a_creature_type),
    statics: &[static_ability(
        Layer::Type,
        your_creatures_spells_and_cards,
        set_chosen_type,
        "Creatures you control are the chosen type. The same is true for creature spells you control and creature cards you own that aren't on the battlefield.",
    )
    .everywhere()],
    ..oracle::CONSPIRACY
};
