use cantrip_api::*;
use cantrip_oracle as oracle;

/// "As this artifact enters, choose a creature type" (CR 614.1c, 614.12a), kept on the permanent for its anthem to read.
fn enters(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let ask = RequestKind::ChooseSubtype { source: entering.object, choices: Subtype::CREATURE_TYPES.to_vec(), purpose: WordUse::Chosen };
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

pub static CARD: CardDef = CardDef {
    enters: Some(enters),
    statics: &[static_ability(Layer::PtModify, affects, cantrip_mechanics::statics::plus_pt::<2, 2>, "Creatures you control of the chosen type get +2/+2.")],
    ..oracle::OBELISK_OF_URD
};
