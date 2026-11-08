use cantrip_api::*;
use cantrip_mechanics::{choose, effects, select};
use cantrip_oracle as oracle;

/// "As this land enters, you may reveal a Plains or Swamp card from your hand. If you don't, this land enters tapped. {T}: Add {W} or {B}." (CR 614.1d, 701.20a).
fn add_w_b_enters(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let asker = &Asker::new(entering.object, entering.controller);
    let payer_0 = entering.controller;
    let (paid_0, _, _, _): (bool, Vec<ObjectId>, Vec<ObjectId>, Vec<ObjectId>) = if effects::ask_may(
        g,
        payer_0,
        asker.source,
        Purpose::Optional,
    )? {
        let selected_2_player = entering.controller;
        let selected_2_candidates = {
            let q: &dyn Query = g;
            Some(entering.controller)
                .into_iter()
                .flat_map(|p: PlayerId| q.state().zone(Zone::hand(p)).to_vec())
                .collect::<Vec<ObjectId>>()
                .into_iter()
                .filter(|&id| {
                    !q.state().obj(id).token && !q.state().obj(id).card_copy
                        && (q
                            .chars(id)
                            .has_subtype(q.words(asker.source).subtype(Subtype::Plains))
                            || q
                                .chars(id)
                                .has_subtype(q.words(asker.source).subtype(Subtype::Swamp)))
                })
                .collect::<Vec<ObjectId>>()
        };
        let selected_2 = choose::select(
            g,
            selected_2_player,
            asker.source,
            Purpose::Reveal,
            selected_2_candidates,
            select::Selection::of(select::Count::AsManyAsCan(1), &[]),
        )?;
        let revealer_1 = entering.controller;
        g.reveal(revealer_1, &selected_2);
        let paid_1 = !selected_2.is_empty();
        (paid_1, Vec::new(), Vec::new(), selected_2.clone())
    } else {
        (false, Vec::new(), Vec::new(), Vec::new())
    };
    if !(((paid_0 as i64) != 0)) {
        entering.tapped = true;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(
        TAP,
        add_one_of(&[Color::White, Color::Black]),
        "{T}: Add {W} or {B}.",
    )],
    enters: Some(add_w_b_enters),
    ..oracle::SHINESHADOW_SNARL
};
