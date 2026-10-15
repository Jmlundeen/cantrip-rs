//! "Choose a creature type" permanents (CR 205.3m, 614.12a): the type is chosen as the permanent enters and kept on it, and
//! its abilities say "of the chosen type". Banner of Kinship, Chronicle of Victory and Door of Destinies print the lot.

use cantrip_api::*;

/// "As this enters, choose a creature type" (CR 614.12a): asked of the entering permanent's controller, any creature type,
/// and kept on the permanent as `chosen_subtype`. The type chosen, if one was.
pub fn choose_creature_type(g: &mut dyn Game, entering: &mut Entering) -> Result<Option<Subtype>, Suspend> {
    let ask = RequestKind::ChooseSubtype { source: entering.object, choices: Subtype::CREATURE_TYPES.to_vec() };
    let Answer::Subtype(chosen) = g.ask(entering.controller, ask)? else { return Ok(None) };
    entering.chosen_subtype = Some(chosen);
    Ok(Some(chosen))
}

/// `choose_creature_type` as a card's `enters`. Chronicle of Victory and Door of Destinies print exactly this.
pub fn enters_choosing_a_creature_type(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    choose_creature_type(g, entering)?;
    Ok(())
}

/// "Creatures you control of the chosen type" (CR 205.3m): what a static ability of the permanent affects, a creature that
/// is the chosen type, under the permanent's controller. Banner of Kinship, Chronicle of Victory and Door of Destinies
/// use it as `StaticDef::affects`.
pub fn creatures_you_control_of_chosen_type(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let s = q.state();
    s.obj(source).chosen_subtype.is_some_and(|t| so_far.is(CardType::Creature) && so_far.has_subtype(t))
        && s.obj(affected).controller == s.obj(source).controller
}

/// "Whenever you cast a spell of the chosen type" (CR 603.2): the spell's own subtypes on the stack (CR 205.3). Chronicle
/// of Victory and Door of Destinies print it.
pub fn you_cast_chosen_type(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. } if *player == q.controller(source) => {
            let chosen = q.obj(source).chosen_subtype?;
            q.chars(*spell).has_subtype(chosen).then_some(Fired::default())
        }
        _ => None,
    }
}
