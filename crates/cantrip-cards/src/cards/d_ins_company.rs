use cantrip_api::*;
use cantrip_mechanics::library::{look_at_top, Rest};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "This creature has lifelink as long as you control another Dwarf" (CR 702.15, 613.1f): another permanent its controller controls
/// with the creature type Dwarf, read through the layers and through the words of this card's text (CR 612). Its own
/// type does not count, and an opponent's Dwarf is not one "you control".
fn has_lifelink_with_another_dwarf(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    if affected != source {
        return false;
    }
    let state = view.state();
    let you = state.obj(source).controller;
    let dwarf = view.words(source).subtype(Subtype::Dwarf);
    state.battlefield().iter().any(|&id| id != source && state.obj(id).controller == you && view.chars(id).has_subtype(dwarf))
}

fn lifelink(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Lifelink);
}

/// "When this creature enters, look at the top four cards of your library. You may reveal a Dwarf or Equipment card from among
/// them and put it into your hand. Put the rest on the bottom of your library in a random order." The creature type is read
/// through this card's words (CR 612); Equipment is an artifact type, which no text-changing effect touches. With fewer
/// than four cards in the library it looks at what there is (CR 609.3), and with none to take it still shows the cards.
fn look(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let dwarf = g.words(item.object).subtype(Subtype::Dwarf);
    look_at_top(
        g,
        item.controller,
        item.object,
        4,
        (0, 1),
        true,
        |c| c.has_subtype(dwarf) || c.has_subtype(Subtype::Equipment),
        Rest::BottomRandom,
    )?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        look,
        "When this creature enters, look at the top four cards of your library. You may reveal a Dwarf or Equipment card from among them and put it into your hand. Put the rest on the bottom of your library in a random order.",
    )],
    statics: &[static_ability(Layer::Ability, has_lifelink_with_another_dwarf, lifelink, "This creature has lifelink as long as you control another Dwarf.")],
    ..oracle::D_INS_COMPANY
};
