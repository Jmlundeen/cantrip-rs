use cantrip_api::*;
use cantrip_mechanics::statics::{your_artifact_creatures, your_of_subtype};
use cantrip_mechanics::{lands, triggers};
use cantrip_oracle as oracle;

const HERO: u16 = lands::sub(Subtype::Hero);

/// "Artifact creatures and Heroes you control" (CR 613.1f): `your_artifact_creatures` and the Heroes under the controller's control,
/// each as the earlier layers made it, "Hero" read through the words of this card's text (CR 612.2).
fn your_artifact_creatures_and_heroes(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    your_artifact_creatures(q, source, affected, so_far) || your_of_subtype::<HERO>(q, source, affected, so_far)
}

/// "+1/+1 for each Artificer you control and each Artificer card in your graveyard" (CR 613.4c, layer 7c): the Artificers on the
/// battlefield as the earlier layers left them, this one among them, and an Artificer card in the graveyard by the subtypes the card
/// has there; "Artificer" read through the words of this card's text (CR 612.2). One count, taken as characteristics are computed.
fn plus_one_per_artificer_you_control_and_in_your_graveyard(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let state = view.state();
    let you = state.obj(source).controller;
    let artificer = view.words(source).subtype(Subtype::Artificer);
    let on_battlefield = state.battlefield().iter().filter(|&&id| state.obj(id).controller == you && view.chars(id).has_subtype(artificer)).count();
    let in_graveyard = state.zone(Zone::graveyard(you)).iter().filter(|&&id| view.chars(id).has_subtype(artificer)).count();
    let n = (on_battlefield + in_graveyard) as i32;
    chars.power = chars.power.map(|v| v + n);
    chars.toughness = chars.toughness.map(|v| v + n);
}

/// "A deck can have any number of cards named Cid, Timeless Artificer" is a deck-construction rule, not an ability: it is not in any
/// game and no line of code here.
pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::PtModify,
        your_artifact_creatures_and_heroes,
        plus_one_per_artificer_you_control_and_in_your_graveyard,
        "Artifact creatures and Heroes you control get +1/+1 for each Artificer you control and each Artificer card in your graveyard.",
    )],
    abilities: &[triggers::cycling(mana("{W}{U}"), "Cycling {W}{U} ({W}{U}, Discard this card: Draw a card.)")],
    ..oracle::CID_TIMELESS_ARTIFICER
};
