//! "Choose a creature type" permanents (CR 205.3m, 614.12a): the type is chosen as the permanent enters and kept on it, and
//! its abilities say "of the chosen type". Banner of Kinship, Chronicle of Victory and Door of Destinies print the lot.

use cantrip_api::*;

/// "As this enters, choose a creature type" (CR 614.12a): asked of the entering permanent's controller, any creature type,
/// and kept on the permanent as `chosen_subtype`. The type chosen, if one was.
pub fn choose_creature_type(g: &mut dyn Game, entering: &mut Entering) -> Result<Option<Subtype>, Suspend> {
    choose_type_among(g, entering, Subtype::CREATURE_TYPES.to_vec())
}

/// "As this enters, choose [one of these types]" (CR 614.12a): `choose_creature_type` among `choices` only, in the order
/// given (Eclipsed Realms' eight, read through its words).
pub fn choose_type_among(g: &mut dyn Game, entering: &mut Entering, choices: Vec<Subtype>) -> Result<Option<Subtype>, Suspend> {
    let ask = RequestKind::ChooseSubtype { source: entering.object, choices, purpose: WordUse::Chosen };
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

/// "Creatures you control ... The same is true for creature spells you control and creature cards you own that aren't on
/// the battlefield" (CR 611.3a, 613.1d): the `affects` of a static ability declared `.everywhere()`, judged on what the layers
/// before made the object - a creature on the battlefield under the source's controller, a creature spell on the stack they
/// control, and a creature card they own anywhere else that is not the battlefield (a hand, a library, a graveyard, exile,
/// the stack under another player's control: a spell is a card there, CR 112.1, 108.2). A token or a copy of a card off
/// the battlefield is no card; a card outside the game the layers leave alone (CR 400.11c). Arcane Adaptation, Conspiracy
/// and Maskwood Nexus print it.
pub fn your_creatures_spells_and_cards(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    if !so_far.is(CardType::Creature) {
        return false;
    }
    let s = q.state();
    let o = s.obj(affected);
    let you = s.obj(source).controller;
    let your_card = o.owner == you && !o.token && !o.card_copy;
    match o.zone.kind() {
        ZoneKind::Battlefield => o.controller == you,
        ZoneKind::Stack => o.controller == you || your_card,
        _ => your_card,
    }
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
