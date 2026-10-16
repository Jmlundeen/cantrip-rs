use cantrip_api::*;
use cantrip_oracle as oracle;

/// "As this artifact enters, choose a creature type" (CR 614.1c, 614.12a), kept on the permanent for its anthem to read.
fn enters(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let ask = RequestKind::ChooseSubtype { source: entering.object, choices: Subtype::CREATURE_TYPES.to_vec() };
    if let Answer::Subtype(chosen) = g.ask(entering.controller, ask)? {
        entering.chosen_subtype = Some(chosen);
    }
    Ok(())
}

/// "Creatures you control of the chosen type".
fn affects(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let s = q.state();
    s.obj(source).chosen_subtype.is_some_and(|t| so_far.is(CardType::Creature) && so_far.has_subtype(t))
        && s.obj(affected).controller == s.obj(source).controller
}

/// "+2/+2" (CR 613.4c): only something that has power and toughness to modify gets it.
fn apply(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 2);
    chars.toughness = chars.toughness.map(|v| v + 2);
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters),
    statics: &[static_ability(Layer::PtModify, affects, apply, "Creatures you control of the chosen type get +2/+2.")],
    ..oracle::OBELISK_OF_URD
};
