use cantrip_api::*;
use cantrip_mechanics::choose_color;
use cantrip_oracle as oracle;

/// "As this creature enters, choose a color" (CR 614.12a): kept on the permanent.
fn enters(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let color = choose_color(g, entering.controller, entering.object)?;
    entering.chosen_color = Some(color);
    Ok(())
}

/// Permanents, spells on the stack (an ability is not an object here, a spell always has card types), and cards
/// in every other zone: not a token, not a copy of a card (CR 108.2, 722.3c).
fn affects(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    if q.state().obj(source).chosen_color.is_none() {
        return false;
    }
    let obj = q.state().obj(affected);
    match obj.zone.kind() {
        ZoneKind::Battlefield => true,
        ZoneKind::Stack => so_far.types != TypeSet::NONE,
        ZoneKind::Gone => false,
        _ => !obj.token && !obj.card_copy,
    }
}

/// "In addition to their other colors" (CR 105.3, 613.1e).
fn apply(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    if let Some(color) = q.state().obj(source).chosen_color {
        chars.colors = chars.colors.with(color);
    }
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters),
    statics: &[static_ability(Layer::Color, affects, apply, "All cards that aren't on the battlefield, spells, and permanents are the chosen color in addition to their other colors.")
        .everywhere()],
    ..oracle::PAINTERS_SERVANT
};
