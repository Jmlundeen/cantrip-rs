use cantrip_api::*;
use cantrip_mechanics::effects::ask_may;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

const ASK: Question = Question::new("PutOnTop", "put it on top of your library and take two damage?", "If you don't, it goes on the bottom", "Top", "Bottom");

/// "Choose target nonland permanent. Its owner may put it on top of their library. If they do, Clash of
/// Elements deals 2 damage to them. If they didn't put the card on top of their library, they put it on the
/// bottom." The owner chooses; yes means top.
fn clash_of_elements(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let owner = g.obj(permanent).owner;
    if ask_may(g, owner, item.object, Purpose::Ask(&ASK))? {
        g.put_into_library(&[permanent], LibraryPlace::Top);
        g.damage(item.object, Target::Player(owner), 2);
    } else {
        g.put_into_library(&[permanent], LibraryPlace::Bottom);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::NONLAND_PERMANENT], clash_of_elements),
    ..oracle::CLASH_OF_ELEMENTS
};
