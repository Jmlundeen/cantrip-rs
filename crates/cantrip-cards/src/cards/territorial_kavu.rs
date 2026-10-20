use cantrip_api::*;
use cantrip_mechanics::statics::itself;
use cantrip_mechanics::{discard_chosen, targets};
use cantrip_oracle as oracle;

/// Domain (CR 604.3, 305.6): power and toughness each equal the number of basic land types among lands its controller
/// controls (its owner, when it is not on the battlefield), a characteristic-defining ability in every zone.
fn domain(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let n = cantrip_mechanics::domain::count_in_layers(q, q.state().obj(source).controller) as i16;
    chars.power = Some(n);
    chars.toughness = Some(n);
}

/// Mode 0: discard a card, and draw a card if you do. The discard is asked of any hand that is not empty.
fn discard_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !discard_chosen(g, item.controller, item.object, 1)?.is_empty() {
        g.draw(item.controller, 1);
    }
    Ok(())
}

/// Mode 1: exile up to one target card from a graveyard.
fn exile_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if item.target_count > 0 && g.target_legal(item, 0) {
        cantrip_mechanics::exile_target(g, item, 0);
    }
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(discard_and_draw, "Discard a card. If you do, draw a card."),
    mode(exile_card, "Exile up to one target card from a graveyard.")
        .targets(&[targets::CARD_IN_A_GRAVEYARD.optional()]),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(cantrip_mechanics::triggers::THIS_ATTACKS, &MODAL, "Whenever this creature attacks, choose one \u{2014} Discard a card. If you do, draw a card.; or Exile up to one target card from a graveyard.")],
    statics: &[static_ability(Layer::PtDefine, itself, domain, "Domain \u{2014} Territorial Kavu's power and toughness are each equal to the number of basic land types among lands you control.")
        .own()
        .lost_pt(0, 0)],
    ..oracle::TERRITORIAL_KAVU
};
