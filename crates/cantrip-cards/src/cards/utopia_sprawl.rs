use cantrip_api::*;
use cantrip_mechanics::{choose_color_for, targets};
use cantrip_oracle as oracle;

/// "Enchant Forest" (CR 702.5a, 303.4a): a permanent that is a Forest now, Forest read as this Aura's text has the word
/// (CR 612.2). An Aura on a permanent that stops being a Forest - a nonbasic Forest under Blood Moon - is put into its
/// owner's graveyard (CR 303.4c, 704.5m).
fn forest(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let forest = q.words(asker.source).subtype(Subtype::Forest);
    matches!(t, Target::Object(id) if targets::permanent(q, id) && q.chars(id).has_subtype(forest))
}

/// "As this Aura enters, choose a color." (CR 614.12a), kept on the permanent (CR 607.2d).
fn enters(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.chosen_color = Some(choose_color_for(g, entering.controller, entering.object, ColorUse::Benefit)?);
    Ok(())
}

/// "Whenever enchanted Forest is tapped for mana" (CR 106.12, 106.12a: a mana ability of it with {T} in its cost resolved and
/// produced mana, which `Event::TappedForMana` is only then followed by triggers for), about that Forest.
fn enchanted_forest_tapped_for_mana(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match *event {
        Event::TappedForMana { permanent, .. } if q.obj(source).attached_to == Some(permanent) => Fired::about(Target::Object(permanent)),
        _ => None,
    }
}

/// "... its controller adds an additional one mana of the chosen color." A triggered mana ability (CR 605.1b), resolving at
/// once (CR 605.4a): the Forest's controller adds it, whoever controls the Aura. The color was chosen as the Aura entered;
/// "the chosen color" is no color word (CR 612.2). An Aura that entered without choosing (none does) adds nothing.
fn add_chosen_color(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let (Some(color), Target::Object(forest)) = (g.obj(item.object).chosen_color, item.subject) else { return Ok(()) };
    let controller = g.controller(forest);
    g.add_mana(controller, color.mana(), 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enchant: Some(target(forest)),
    enters: Some(enters),
    abilities: &[triggered(
        when(EventMask::TAPPED, enchanted_forest_tapped_for_mana),
        add_chosen_color,
        "Whenever enchanted Forest is tapped for mana, its controller adds an additional one mana of the chosen color.",
    )
    .mana_ability()],
    ..oracle::UTOPIA_SPRAWL
};
