use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Cosmogoyf's power is equal to the number of cards you own in exile and its toughness is equal to that number plus 1" (CR 604.3,
/// 604.3a): a characteristic-defining ability that works in every zone, so in exile it counts itself (its ruling). A player's exile zone
/// holds the cards they own (CR 400.1, 108.3), whoever controls them or controlled them last; "you" is its controller (the engine keeps
/// the owner as controller off the battlefield and stack, CR 108.4a). What is there but no card is not counted: a token not yet gone
/// (CR 111.7, 704.5d), and a copy of a card, as a prepared permanent's prepare spell waiting there (CR 707.12, 722.3).
fn size(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let state = q.state();
    let zone = state.zone(Zone::exile(state.obj(source).controller));
    let owned = zone.iter().filter(|&&id| !state.obj(id).token && !state.obj(id).card_copy).count() as i16;
    chars.power = Some(owned);
    chars.toughness = Some(owned + 1);
}

pub static CARD: CardDef = CardDef {
    statics: &[characteristic_defining(
        Layer::PtDefine,
        size,
        "Cosmogoyf's power is equal to the number of cards you own in exile and its toughness is equal to that number plus 1.",
    )
    .lost_pt(0, 1)],
    ..oracle::COSMOGOYF
};
