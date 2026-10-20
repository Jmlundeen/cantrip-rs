use cantrip_api::*;
use cantrip_mechanics::{library::find_to_hand, select::{basic_land_cards, up_to, DIFFERENT_NAMES}, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Domain — Fblthp's power is equal to the number of basic land types among lands you control."
fn domain(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    chars.power = Some(cantrip_mechanics::domain::count_in_layers(q, q.state().obj(source).controller) as i16);
}

/// "When Fblthp enters, search your library for up to X basic land cards with different names, reveal them, put them into
/// your hand, then shuffle." (CR 701.23, 701.23b, 701.23f, 201.2b).
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let x = u32::from(g.obj(item.object).x);
    find_to_hand(g, player, item.object, basic_land_cards(up_to(x)).with(DIFFERENT_NAMES), true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, search, "When Fblthp enters, search your library for up to X basic land cards with different names, reveal them, put them into your hand, then shuffle.")],
    statics: &[characteristic_defining(Layer::PtDefine, domain, "Domain — Fblthp's power is equal to the number of basic land types among lands you control.")
        .lost_pt(0, 2)],
    ..oracle::FBLTHP_KNOWS_THE_WAY
};
