use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Kraven's power is equal to the greatest mana value among permanents you control" (CR 604.3, 202.3, layer 7a): a
/// characteristic-defining ability, Kraven himself among the permanents counted when he is on the battlefield. Tokens and
/// lands have mana value 0, so with nothing else it is Kraven's own 3. The controller of each is read through the view (CR 613.1b).
fn power_is_greatest_mana_value_you_control(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let controller = q.controller(source);
    let greatest = q
        .state()
        .battlefield()
        .iter()
        .filter(|&&id| q.controller(id) == controller)
        .map(|&id| q.mana_value(id, &q.chars(id)))
        .max()
        .unwrap_or(0);
    chars.power = Some(greatest as i32);
}

/// Vigilance is a core keyword.
pub static CARD: CardDef = CardDef {
    statics: &[characteristic_defining(
        Layer::PtDefine,
        power_is_greatest_mana_value_you_control,
        "Top of the Food Chain — Kraven's power is equal to the greatest mana value among permanents you control.",
    )
    .lost_pt(0, 4)],
    ..oracle::KRAVEN_PROUD_PREDATOR
};
