use cantrip_api::*;
use cantrip_mechanics::{statics::have_trample, targets};
use cantrip_oracle as oracle;

/// Four or more card types among the cards exiled with this creature (CR 610.3): the cards in exile whose link points
/// back at it, their types counted without regard to how many cards have each.
fn collection(q: &dyn LayerQuery, source: ObjectId) -> bool {
    let mut types = TypeSet::NONE;
    for id in q.exiled_with(q.linked_to(source)) {
        types = TypeSet(types.0 | q.chars(id).types.0);
    }
    types.0.count_ones() >= 4
}

fn itself_with_collection(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    source == affected && collection(q, source)
}

fn plus_four(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|p| p + 4);
    chars.toughness = chars.toughness.map(|t| t + 4);
}

/// Exile target card from a graveyard; it is exiled with this creature (CR 607.2a).
fn exile_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.exile_with(&[card], g.linked(item), false);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{1}"), exile_card, "{1}: Exile target card from a graveyard.")
        .targets(&[targets::CARD_IN_A_GRAVEYARD])],
    statics: &[
        static_ability(Layer::PtModify, itself_with_collection, plus_four, "As long as there are four or more card types among cards exiled with this creature, it gets +4/+4."),
        static_ability(Layer::Ability, itself_with_collection, have_trample, "As long as there are four or more card types among cards exiled with this creature, it has trample."),
    ],
    ..oracle::KEEN_EYED_CURATOR
};
