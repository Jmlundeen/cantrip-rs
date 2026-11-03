use cantrip_api::*;
use cantrip_oracle as oracle;

/// This characteristic-defining ability works in every zone (CR 604.3) and counts current creature names (CR 201.2a).
fn plague_ratss_power_toughness_equal_apply_0(
    q: &dyn LayerQuery,
    _: ObjectId,
    chars: &mut Chars,
) {
    let count = q.state().battlefield().iter().copied().filter(|&id| {
        q.chars(id).is(CardType::Creature)
            && cantrip_mechanics::library::object_names_in_layers(q, id).contains(&"Plague Rats")
    }).count() as i16;
    chars.power = Some(count);
    chars.toughness = Some(count);
}

pub static CARD: CardDef = CardDef {
    power: Some(0),
    toughness: Some(0),
    statics: &[
        characteristic_defining(
            Layer::PtDefine,
            plague_ratss_power_toughness_equal_apply_0,
            "Plague Rats's power and toughness are each equal to the number of creatures named Plague Rats on the battlefield.",
        ),
    ],
    ..oracle::PLAGUE_RATS
};
