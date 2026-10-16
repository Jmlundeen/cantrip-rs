use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

const TYPES: [Subtype; 5] = [Subtype::Plains, Subtype::Island, Subtype::Swamp, Subtype::Mountain, Subtype::Forest];

/// "As this land enters, choose a basic land type. Then you may pay 2 life. If you don't, it enters tapped."
/// The type is chosen first (kept on the permanent, CR 614.12a); the payment is asked only of a player who can make it.
fn enters(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let card = entering.copy.map(|c| c.card).unwrap_or(g.obj(entering.object).card);
    let chosen = choose::one_of(g, entering.controller, entering.object, (0..5).map(|i| Label::Card(card, i)).collect())?;
    entering.chosen_subtype = Some(TYPES[chosen as usize]);

    let paid = g.state().player(entering.controller).life >= 2
        && effects::ask_may(g, entering.controller, entering.object, Purpose::Optional)?;
    if paid {
        g.lose_life(entering.controller, 2);
    } else {
        entering.tapped = true;
    }
    Ok(())
}

fn affects(_: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    source == affected
}

/// "This land is the chosen type" (CR 305.7): replaces its subtypes; the mana ability follows from the subtype (CR 305.6).
fn apply(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    if let Some(chosen) = q.state().obj(source).chosen_subtype {
        chars.subtypes = SubtypeSet::NONE.with(chosen);
    }
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters),
    labels: &["Plains", "Island", "Swamp", "Mountain", "Forest"],
    statics: &[static_ability(Layer::Type, affects, apply, "This land is the chosen type.")],
    ..oracle::MULTIVERSAL_PASSAGE
};
