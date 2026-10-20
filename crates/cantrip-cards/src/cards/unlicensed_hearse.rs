use cantrip_api::*;
use cantrip_mechanics::{crew::crew, targets};
use cantrip_oracle as oracle;

/// "Exile up to two target cards from a single graveyard." The cards still in that graveyard are exiled together (CR
/// 608.2b: a target that became illegal is left alone), each exiled with this Vehicle (CR 607.2a), where its power and
/// toughness count them.
fn exile_cards(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cards: Vec<ObjectId> = (0..2).filter_map(|slot| targets::object(g, item, slot)).collect();
    g.exile_with(&cards, item.object, false);
    Ok(())
}

/// "Unlicensed Hearse's power and toughness are each equal to the number of cards exiled with it" (CR 604.3, 607.2a): the
/// cards in exile whose link points back at it, a characteristic-defining ability in every zone. A Vehicle that left and
/// came back is a new object (CR 400.7) with none.
fn exiled_with_it(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let s = q.state();
    let n = s.seats().flat_map(|p| s.zone(Zone::exile(p)).iter().copied()).filter(|&id| s.obj(id).exiled.is_some_and(|l| l.by == source)).count();
    chars.power = Some(n as i16);
    chars.toughness = Some(n as i16);
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(TAP, exile_cards, "{T}: Exile up to two target cards from a single graveyard.").targets(&[
            targets::CARD_IN_A_GRAVEYARD.optional().labelled("First card to exile (up to two, from a single graveyard)"),
            targets::ANOTHER_CARD_IN_THE_SAME_GRAVEYARD.optional().labelled("Second card to exile, from the same graveyard"),
        ]),
        crew(2, "Crew 2 (Tap any number of creatures you control with total power 2 or more: This Vehicle becomes an artifact creature until end of turn.)"),
    ],
    statics: &[characteristic_defining(Layer::PtDefine, exiled_with_it, "Unlicensed Hearse's power and toughness are each equal to the number of cards exiled with it.").lost_pt(0, 0)],
    ..oracle::UNLICENSED_HEARSE
};
