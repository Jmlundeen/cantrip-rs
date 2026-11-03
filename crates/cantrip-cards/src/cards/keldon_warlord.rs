use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

/// Power and toughness count non-Wall creatures its controller controls in every zone (CR 604.3, 613.4a).
fn power_and_toughness(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let wall = q.words(source).subtype(Subtype::Wall);
    let n = lands::count_you_control(q, source, &|c| c.is(CardType::Creature) && !c.has_subtype(wall));
    chars.power = Some(n);
    chars.toughness = Some(n);
}

pub static CARD: CardDef = CardDef {
    statics: &[characteristic_defining(
        Layer::PtDefine,
        power_and_toughness,
        "Keldon Warlord's power and toughness are each equal to the number of non-Wall creatures you control.",
    ).lost_pt(0, 0)],
    ..oracle::KELDON_WARLORD
};
